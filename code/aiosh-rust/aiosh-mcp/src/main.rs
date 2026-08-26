//! AIOS MCP server — Model Context Protocol tool surface (Rust rewrite).
//!
//! ADR-0035 §D-2 binding: MCP is the only tool-call protocol AIOS
//! exposes to external models. Implements a minimal stdio JSON-RPC
//! server with `initialize`, `tools/list`, and `tools/call`, routing
//! every tool through the classifier → PEP → audit gate.
//!
//! Tools:
//!   aios.fs.read, aios.process.list, aios.audit.tail, aios.audit.verify,
//!   aios.audit.rotate [grant], aios.audit.segments, aios.audit.seen,
//!   aios.pentest.nmap/nikto/sqlmap/tshark/aircrack-ng [C-1]

use aiosh_core::audit::{active_constitution_rev, AuditRing, OpenOptions};
use aiosh_core::dispatch;
use aiosh_core::pentest;
use aiosh_core::pep::PepStore;
use aiosh_core::retention;
use aiosh_core::task_service;
use serde_json::{json, Value};
use std::io::Write;

const SCHEMA_VERSION: &str = "2025-06-18";

struct Server {
    ring: AuditRing,
    pep: PepStore,
    constitution_rev: String,
}

impl Server {
    fn open() -> Self {
        let ring = AuditRing::open(OpenOptions::default()).expect("open audit db");
        ring.prepare_for_write().expect("prepare schemas");
        let pep_path = ring.path().to_string();
        let pep = if pep_path == ":memory:" {
            PepStore::new(rusqlite::Connection::open_in_memory().unwrap()).expect("open pep store")
        } else {
            PepStore::new(rusqlite::Connection::open(&pep_path).expect("open pep db")).expect("open pep store")
        };
        let constitution_rev = active_constitution_rev(None);
        Server { ring, pep, constitution_rev }
    }

    fn tool_manifest(&self) -> Vec<Value> {
        let mut tools = vec![
            json!({"name": "aios.fs.read", "description": "Read a UTF-8 text file (path PEP gated)", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "grant_id": {"type": "string"}}, "required": ["path"]}}),
            json!({"name": "aios.process.list", "description": "List running processes (read-only)", "inputSchema": {"type": "object"}}),
            json!({"name": "aios.audit.tail", "description": "Tail the audit ring", "inputSchema": {"type": "object", "properties": {"n": {"type": "integer"}}}}),
            json!({"name": "aios.audit.verify", "description": "Verify the audit-ring hash chain (full=True replays archived segments)", "inputSchema": {"type": "object", "properties": {"full": {"type": "boolean"}}}}),
            json!({"name": "aios.audit.rotate", "description": "Seal live rows into an archived segment [grant]", "inputSchema": {"type": "object", "properties": {"keep_rows": {"type": "integer"}, "grant_id": {"type": "string"}}}}),
            json!({"name": "aios.audit.segments", "description": "List archived rotation checkpoints", "inputSchema": {"type": "object"}}),
            json!({"name": "aios.audit.seen", "description": "Bloom-backed was-this-hash-ever-logged query", "inputSchema": {"type": "object", "properties": {"hash": {"type": "string"}, "exact": {"type": "boolean"}}}}),
        ];
        for (name, desc) in [
            ("aios.pentest.nmap", "TCP recon (top-100 ports) [C-1]"),
            ("aios.pentest.nikto", "web-misconfig scan (safe tuning) [C-1]"),
            ("aios.pentest.sqlmap", "SQL injection (level=1 risk=1) [C-1]"),
            ("aios.pentest.tshark", "pcap read (no live capture) [C-1]"),
            ("aios.pentest.aircrack-ng", "offline dictionary crack [C-1]"),
        ] {
            tools.push(json!({
                "name": name,
                "description": desc,
                "inputSchema": {"type": "object"},
            }));
        }
        tools.push(json!({
            "name": "aios.task",
            "description": "Task Ledger Control: query or advance the AIOS master task ledger. Read-only: status, check, metrics, validate. Consequential (PEP grant required): done, block, unblock, skip, rebuild.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": [
                        "status", "check", "validate", "done", "block",
                        "unblock", "skip", "rebuild", "metrics"]},
                    "task_id": {"type": "integer", "minimum": 1},
                    "note": {"type": "string", "minLength": 1, "maxLength": 4096},
                    "reason": {"type": "string", "minLength": 1, "maxLength": 4096},
                    "evidence": {"type": "array", "items": {"type": "string"}, "maxItems": 16},
                    "grant_id": {"type": "string"}
                },
                "required": ["action"],
                "additionalProperties": false
            }
        }));
        tools
    }

    fn call_tool(&mut self, tool: &str, arguments: &Value) -> Value {
        let grant_id = arguments.get("grant_id").and_then(|v| v.as_str());
        match tool {
            "aios.fs.read" => {
                let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let abs_path = std::fs::canonicalize(path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.to_string());
                let f = || -> Result<Value, String> {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                    let safe_roots = vec!["/tmp".to_string(), format!("{}/.aios", home)];
                    if !safe_roots
                        .iter()
                        .any(|r| abs_path == *r || abs_path.starts_with(&format!("{}/", r)))
                    {
                        return Err(format!("path '{}' outside safe roots", abs_path));
                    }
                    let data = std::fs::read_to_string(&abs_path).map_err(|e| e.to_string())?;
                    let truncated = data.len() > 16384;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.fs.read",
                        "path": abs_path,
                        "bytes": data.len(),
                        "truncated": truncated,
                        "content": data.chars().take(16384).collect::<String>(),
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.fs.read", &format!("fs.read {}", abs_path),
                    &json!({"path": abs_path}), Some(&abs_path), grant_id, true,
                    dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.process.list" => {
                let f = || -> Result<Value, String> {
                    let mut procs: Vec<Value> = vec![];
                    if let Ok(entries) = std::fs::read_dir("/proc") {
                        for entry in entries.flatten() {
                            let name = entry.file_name();
                            let name = name.to_string_lossy().to_string();
                            if !name.chars().all(|c| c.is_ascii_digit()) {
                                continue;
                            }
                            let comm = std::fs::read_to_string(format!("/proc/{}/comm", name))
                                .unwrap_or_default();
                            let comm = comm.trim().to_string();
                            if !comm.is_empty() {
                                procs.push(json!({"pid": name.parse::<i64>().unwrap_or(0), "name": comm}));
                            }
                        }
                    }
                    procs.sort_by_key(|p| p["pid"].as_i64().unwrap_or(0));
                    procs.truncate(256);
                    Ok(json!({"ok": true, "tool": "aios.process.list",
                              "count": procs.len(), "processes": procs}))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.process.list", "process.list", &json!({}), None, None, false,
                    dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.audit.tail" => {
                let n = arguments.get("n").and_then(|v| v.as_i64()).unwrap_or(10);
                let rows = self.ring.tail(n).map_err(|e| e.to_string());
                let f = || -> Result<Value, String> {
                    let rows = rows.as_ref().map_err(|e| e.clone())?;
                    let rows_json: Vec<Value> = rows.iter().map(row_to_json).collect();
                    Ok(json!({"ok": true, "tool": "aios.audit.tail",
                              "count": rows_json.len(), "rows": rows_json}))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.audit.tail", &format!("audit.tail {}", n), &json!({"n": n}),
                    None, None, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.audit.verify" => {
                let full = arguments.get("full").and_then(|v| v.as_bool()).unwrap_or(false);
                let live_verify = if full {
                    None
                } else if tool == "aios.ci" {
                    let action = arguments.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let file = arguments.get("file").and_then(|v| v.as_str());
                    let result = server.call_ci(action, file);
                    let is_error = result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                    json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{"type": "text", "text": result.to_string()}],
                            "structuredContent": {"result": result},
                            "isError": is_error,
                        }
                    })
                } else {
                    self.ring.verify().map_err(|e| e.to_string()).ok()
                };
                let db_path = self.ring.path().to_string();
                let f = || -> Result<Value, String> {
                    let result = if full {
                        let conn = if db_path == ":memory:" {
                            rusqlite::Connection::open_in_memory().unwrap()
                        } else if tool == "aios.ci" {
                    let action = arguments.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let file = arguments.get("file").and_then(|v| v.as_str());
                    let result = server.call_ci(action, file);
                    let is_error = result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                    json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{"type": "text", "text": result.to_string()}],
                            "structuredContent": {"result": result},
                            "isError": is_error,
                        }
                    })
                } else {
                            rusqlite::Connection::open(&db_path).unwrap()
                        };
                        let res = retention::verify_full(&conn, None).map_err(|e| e.to_string())?;
                        json!({
                            "ok": res.ok, "mode": "full",
                            "checked": res.checked,
                            "segments": res.segments,
                            "archive_checked": res.archive_checked,
                            "live_checked": res.live_checked,
                            "anchor": res.anchor,
                            "broken_at": res.broken_at,
                            "broken_segment": res.broken_segment,
                            "error": res.error,
                        })
                    } else if tool == "aios.ci" {
                    let action = arguments.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let file = arguments.get("file").and_then(|v| v.as_str());
                    let result = server.call_ci(action, file);
                    let is_error = result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                    json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{"type": "text", "text": result.to_string()}],
                            "structuredContent": {"result": result},
                            "isError": is_error,
                        }
                    })
                } else {
                        let res = live_verify.as_ref().ok_or("audit.verify failed")?;
                        json!({
                            "ok": res.ok, "mode": "live",
                            "checked": res.checked,
                            "segments": res.segments,
                            "anchor": res.anchor,
                            "broken_at": res.broken_at,
                        })
                    };
                    let ok = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                    let mut out = result.clone();
                    out["tool"] = json!("aios.audit.verify");
                    out["ok_"] = json!(ok);
                    Ok(out)
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.audit.verify", &format!("audit.verify full={}", full),
                    &json!({"full": full}), None, None, false,
                    dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.audit.rotate" => {
                let keep_rows = arguments.get("keep_rows").and_then(|v| v.as_i64()).unwrap_or(0);
                let verdict = dispatch::dispatch(
                    &mut self.ring, &self.pep,
                    "audit.rotate", "audit.rotate", &json!({"keep_rows": keep_rows}),
                    None, grant_id, true, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR,
                );
                if !verdict.ok {
                    return verdict.to_json();
                }
                let db_path = self.ring.path().to_string();
                let conn = if db_path == ":memory:" {
                    rusqlite::Connection::open_in_memory().unwrap()
                } else if tool == "aios.ci" {
                    let action = arguments.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let file = arguments.get("file").and_then(|v| v.as_str());
                    let result = server.call_ci(action, file);
                    let is_error = result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                    json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{"type": "text", "text": result.to_string()}],
                            "structuredContent": {"result": result},
                            "isError": is_error,
                        }
                    })
                } else {
                    rusqlite::Connection::open(&db_path).unwrap()
                };
                match retention::rotate(
                    &conn,
                    &mut self.ring,
                    retention::RotateOptions {
                        keep_rows,
                        actor: "agent".into(),
                        actor_id: dispatch::DEFAULT_ACTOR_ID.into(),
                        grant_token: grant_id.map(|s| s.into()),
                        constitution_rev: Some(active_constitution_rev(None)),
                        ..Default::default()
                    },
                ) {
                    Ok(res) => {
                        if res.ok {
                            let mut out = res.to_json();
                            out["tool"] = json!("audit.rotate");
                            out["classifier_policy_revision"] = json!(verdict.policy_revision);
                            out
                        } else if tool == "aios.ci" {
                    let action = arguments.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let file = arguments.get("file").and_then(|v| v.as_str());
                    let result = server.call_ci(action, file);
                    let is_error = result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                    json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{"type": "text", "text": result.to_string()}],
                            "structuredContent": {"result": result},
                            "isError": is_error,
                        }
                    })
                } else {
                            let mut out = res.to_json();
                            out["tool"] = json!("audit.rotate");
                            out["gate"] = json!("retention");
                            out["classifier_policy_revision"] = json!(verdict.policy_revision);
                            out
                        }
                    }
                    Err(e) => {
                        let row = dispatch::commit(
                            &mut self.ring, "audit.rotate", "audit.rotate",
                            &json!({"keep_rows": keep_rows}), None, grant_id,
                            "error", Some(&e.to_string()),
                            dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, &verdict,
                        );
                        json!({"ok": false, "tool": "audit.rotate",
                               "error": e.to_string(), "audit_id": row.id})
                    }
                }
            }
            "aios.audit.segments" => {
                let db_path = self.ring.path().to_string();
                let f = || -> Result<Value, String> {
                    let conn = if db_path == ":memory:" {
                        rusqlite::Connection::open_in_memory().unwrap()
                    } else if tool == "aios.ci" {
                    let action = arguments.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let file = arguments.get("file").and_then(|v| v.as_str());
                    let result = server.call_ci(action, file);
                    let is_error = result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                    json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{"type": "text", "text": result.to_string()}],
                            "structuredContent": {"result": result},
                            "isError": is_error,
                        }
                    })
                } else {
                        rusqlite::Connection::open(&db_path).unwrap()
                    };
                    let segs = retention::list_segments(&conn).map_err(|e| e.to_string())?;
                    let segs_json: Vec<Value> = segs
                        .iter()
                        .map(|s| {
                            json!({
                                "segment_id": s.segment_id,
                                "closed_at": s.closed_at,
                                "first_row_id": s.first_row_id,
                                "last_row_id": s.last_row_id,
                                "row_count": s.row_count,
                                "genesis_prev_hash": s.genesis_prev_hash,
                                "head_hash": s.head_hash,
                                "archive_path": s.archive_path,
                                "archive_sha256": s.archive_sha256,
                                "bloom_m_bits": s.bloom_m_bits,
                                "bloom_k": s.bloom_k,
                                "bloom_hex": s.bloom_hex,
                            })
                        })
                        .collect();
                    Ok(json!({"ok": true, "tool": "aios.audit.segments",
                              "count": segs_json.len(), "segments": segs_json}))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.audit.segments", "audit.segments", &json!({}), None, None, false,
                    dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.audit.seen" => {
                let hash = arguments.get("hash").and_then(|v| v.as_str()).unwrap_or("");
                let exact = arguments.get("exact").and_then(|v| v.as_bool()).unwrap_or(false);
                let db_path = self.ring.path().to_string();
                let f = || -> Result<Value, String> {
                    let conn = if db_path == ":memory:" {
                        rusqlite::Connection::open_in_memory().unwrap()
                    } else if tool == "aios.ci" {
                    let action = arguments.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let file = arguments.get("file").and_then(|v| v.as_str());
                    let result = server.call_ci(action, file);
                    let is_error = result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                    json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{"type": "text", "text": result.to_string()}],
                            "structuredContent": {"result": result},
                            "isError": is_error,
                        }
                    })
                } else {
                        rusqlite::Connection::open(&db_path).unwrap()
                    };
                    let res = retention::seen(&conn, hash, exact, None).map_err(|e| e.to_string())?;
                    Ok(json!({"ok": true, "tool": "aios.audit.seen",
                              "found": res.found, "id": res.id,
                              "segments": res.segments, "note": res.note}))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.audit.seen", &format!("audit.seen {}", hash),
                    &json!({"hash": hash, "exact": exact}), None, None, false,
                    dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.pentest.nmap" => {
                let target = arguments.get("target").and_then(|v| v.as_str()).unwrap_or("");
                let timeout = arguments.get("timeout_s").and_then(|v| v.as_u64()).unwrap_or(60);
                pentest::pentest_nmap(&mut self.pentest_ctx(), target, grant_id, timeout)
            }
            "aios.pentest.nikto" => {
                let target = arguments.get("target").and_then(|v| v.as_str()).unwrap_or("");
                let timeout = arguments.get("timeout_s").and_then(|v| v.as_u64()).unwrap_or(90);
                pentest::pentest_nikto(&mut self.pentest_ctx(), target, grant_id, timeout)
            }
            "aios.pentest.sqlmap" => {
                let url = arguments.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let level = arguments.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
                let risk = arguments.get("risk").and_then(|v| v.as_i64()).unwrap_or(1);
                let timeout = arguments.get("timeout_s").and_then(|v| v.as_u64()).unwrap_or(300);
                pentest::pentest_sqlmap(&mut self.pentest_ctx(), url, grant_id, level, risk, timeout)
            }
            "aios.pentest.tshark" => {
                let pcap = arguments.get("pcap_path").and_then(|v| v.as_str()).unwrap_or("");
                let filter = arguments.get("display_filter").and_then(|v| v.as_str());
                let timeout = arguments.get("timeout_s").and_then(|v| v.as_u64()).unwrap_or(30);
                pentest::pentest_tshark(&mut self.pentest_ctx(), pcap, filter, grant_id, timeout)
            }
            "aios.pentest.aircrack-ng" => {
                let capture = arguments.get("capture_path").and_then(|v| v.as_str()).unwrap_or("");
                let wordlist = arguments.get("wordlist_path").and_then(|v| v.as_str()).unwrap_or("");
                let timeout = arguments.get("timeout_s").and_then(|v| v.as_u64()).unwrap_or(120);
                pentest::pentest_aircrack_ng(&mut self.pentest_ctx(), capture, wordlist, grant_id, timeout)
            }
            _ => json!({"ok": false, "error": format!("unknown tool: {}", tool)}),
        }
    }

    fn pentest_ctx(&mut self) -> pentest::RunToolCtx<'_> {
        pentest::RunToolCtx {
            ring: &mut self.ring,
            pep: &self.pep,
            constitution_rev: &self.constitution_rev,
            actor_id: "agent:mcp@aiosh-mcp",
        }
    }

    /// `aios.task` — gate + execute per spec T-00022 §4. Exactly one
    /// audit row for every outcome (ok / refused / error).

    fn call_ci(&mut self, action: &str, file_arg: Option<&str>) -> Value {
        let file_path = file_arg
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                std::env::var("AIOSH_CI_RESULTS").unwrap_or_else(|_| "/tmp/aiosh-ci-results.json".to_string())
            });

        let path = std::path::Path::new(&file_path);
        let summary = match aiosh_core::ci::load_summary_with_retry(path, 3) {
            Ok(s) => s,
            Err(e) => {
                let verdict = dispatch::dispatch(
                    &mut self.ring, &self.pep,
                    "aios.ci", "ci.error", &json!({"action": action, "file": file_path}),
                    None, None, false,
                    "agent:mcp@aiosh-mcp", "agent:mcp",
                );
                dispatch::commit(
                    &mut self.ring, "aios.ci", "ci.error",
                    &json!({"action": action, "file": file_path}), None,
                    None, "error", Some(&e),
                    "agent:mcp@aiosh-mcp", "agent:mcp", &verdict,
                );
                return json!({"ok": false, "error": format!("ci-service: {}", e)});
            }
        };

        match action {
            "show" => {
                let report = aiosh_core::ci::human_report(&summary);
                json!({"ok": true, "result": report})
            }
            "failures" => {
                let failures: Vec<_> = summary.results.iter().filter(|r| r.status != "pass").collect();
                let mut report = String::new();
                if failures.is_empty() {
                    report.push_str("no failed suites\n");
                } else {
                    for r in failures {
                        let rc = r.exit_code.map_or("-".to_string(), |c| c.to_string());
                        report.push_str(&format!("[FAIL] {} {} ({} ms) exit={} log={}\n", r.index, r.suite, r.duration_ms, rc, r.log_path));
                    }
                }
                json!({"ok": true, "result": report})
            }
            "check" => {
                let passed = summary.all_pass;
                let msg = if passed {
                    format!("ci-check: PASS ({}/{} suites)", summary.passed, summary.total)
                } else {
                    format!("ci-check: FAIL ({}/{} suites, {} failed)", summary.passed, summary.total, summary.failed)
                };
                
                let verdict = dispatch::dispatch(
                    &mut self.ring, &self.pep,
                    "aios.ci", "ci.check", &json!({"action": action, "file": file_path}),
                    None, None, false,
                    "agent:mcp@aiosh-mcp", "agent:mcp",
                );
                dispatch::commit(
                    &mut self.ring, "aios.ci", "ci.check",
                    &json!({"action": action, "file": file_path}), None,
                    None, if passed { "success" } else { "failure" }, Some(&msg),
                    "agent:mcp@aiosh-mcp", "agent:mcp", &verdict,
                );

                json!({"ok": true, "result": msg})
            }
            _ => {
                json!({"ok": false, "error": "unknown action"})
            }
        }
    }

    fn call_task(&mut self, args: &task_service::TaskArgsOwned) -> Value {
        // T-00084: metrics needs THIS server's ring facts, so it is
        // composed here rather than through call.execute() (which is
        // ring-less by design). Same gate + one honest commit row.
        if args.action == task_service::TaskAction::Metrics {
            let verdict = dispatch::dispatch(
                &mut self.ring, &self.pep,
                "aios.task", "task.metrics", &json!({"action": "metrics"}),
                None, args.grant_id.as_deref(), false,
                "agent:mcp@aiosh-mcp", "agent:mcp",
            );
            if !verdict.ok {
                return verdict.to_json();
            }
            // T-00085: parity with Python pre-gate validate (and CLI) —
            // metrics is a read-only composer and takes NO task_id.
            // Refusal still earns exactly one honest audit row (SPEC §8).
            if args.task_id.is_some() {
                let detail = "action 'metrics' does not take 'task_id'";
                let row = dispatch::commit(
                    &mut self.ring, "aios.task", "task.metrics",
                    &json!({"action": "metrics"}), None,
                    args.grant_id.as_deref(), "refused", Some(detail),
                    "agent:mcp@aiosh-mcp", "agent:mcp", &verdict,
                );
                return json!({"ok": false, "action": "metrics",
                              "error": detail, "audit_id": row.id});
            }
            let tasks = match aiosh_core::ledger::paths()
                .and_then(|p| aiosh_core::ledger::load_state(&p.state, &p.events))
            {
                Ok(t) => t,
                Err(e) => {
                    let row = dispatch::commit(
                        &mut self.ring, "aios.task", "task.metrics",
                        &json!({"action": "metrics"}), None,
                        args.grant_id.as_deref(), "error", Some(&e),
                        "agent:mcp@aiosh-mcp", "agent:mcp", &verdict,
                    );
                    return json!({"ok": false, "action": "metrics",
                                  "error": e, "audit_id": row.id});
                }
            };
            let verify = self.ring.verify().map(|v| v.ok).unwrap_or(false);
            // T-00088 hardening: O(1) COUNT(*) instead of loading every
            // live row into memory via tail(i64::MAX).
            let rows = self.ring.count().unwrap_or(0) as usize;
            let head = self.ring.tail(1).unwrap_or_default();
            let head_prefix = head
                .first()
                .map(|r| r.hash.chars().take(12).collect::<String>())
                .unwrap_or_default();
            let cfg = match aiosh_core::ledger_config::LedgerConfig::from_env() {
                Ok(c) => c,
                Err(e) => {
                    let row = dispatch::commit(
                        &mut self.ring, "aios.task", "task.metrics",
                        &json!({"action": "metrics"}), None,
                        args.grant_id.as_deref(), "error", Some(&e),
                        "agent:mcp@aiosh-mcp", "agent:mcp", &verdict,
                    );
                    return json!({"ok": false, "action": "metrics",
                                  "error": e, "audit_id": row.id});
                }
            };
            // Envelope parity with Python: bare payload nests under
            // "data" (spec T-00082 stable-key contract).
            let mut out = match task_service::TaskCall::build_metrics(
                tasks, rows, verify, &head_prefix, &cfg,
            ) {
                Ok(data) => json!({"ok": true, "action": "metrics", "data": data}),
                Err(e) => json!({"ok": false, "action": "metrics", "error": e}),
            };
            let row = dispatch::commit(
                &mut self.ring, "aios.task", "task.metrics",
                &json!({"action": "metrics"}), None,
                args.grant_id.as_deref(), "ok", None,
                "agent:mcp@aiosh-mcp", "agent:mcp", &verdict,
            );
            out["audit_id"] = json!(row.id);
            out["classifier_policy_revision"] = json!(verdict.policy_revision);
            return out;
        }
        let call = args.call();
        let args_json = json!({
            "action": call.action.as_str(),
            "task_id": call.task_id,
            "note": call.note,
            "reason": call.reason,
            "evidence": call.evidence,
        });
        dispatch::recorded_call(
            &mut self.ring,
            &self.pep,
            "aios.task",
            &format!("task.{}", call.action.as_str()),
            &args_json,
            None,
            args.grant_id.as_deref(),
            call.action.requires_grant(),
            "agent:mcp@aiosh-mcp",
            "agent:mcp",
            move || {
                let action = call.action;
                call.validate()?; // T-00054: conditional presence/caps (single source)
                let v = call.execute()?;
                // Bare payloads get the standard envelope (spec §3.2);
                // mutations already carry ok:true and gain `action`.
                let out = match action {
                    task_service::TaskAction::Status
                    | task_service::TaskAction::Check
                    | task_service::TaskAction::Rebuild => {
                        json!({"ok": true, "action": action.as_str(), "data": v})
                    }
                    _ => {
                        let mut m = v;
                        m["action"] = json!(action.as_str());
                        m
                    }
                };
                Ok(out)
            },
        )
    }
}

fn row_to_json(r: &aiosh_core::types::AuditRow) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("id".into(), json!(r.id));
    m.insert("ts".into(), json!(r.ts));
    m.insert("actor".into(), json!(r.actor));
    m.insert("actor_id".into(), json!(r.actor_id));
    m.insert("tool".into(), json!(r.tool));
    m.insert("command".into(), json!(r.command));
    m.insert("args".into(), r.args.clone());
    m.insert("target".into(), json!(r.target));
    m.insert("outcome".into(), json!(r.outcome));
    m.insert("outcome_detail".into(), json!(r.outcome_detail));
    m.insert("constitution_rev".into(), json!(r.constitution_rev));
    m.insert("grant_token".into(), json!(r.grant_token));
    m.insert("c_flags".into(), r.c_flags.to_json());
    if let Some(p) = &r.policy_revision {
        m.insert("policy_revision".into(), json!(p));
    }
    if let Some(ids) = &r.classify_rule_ids {
        m.insert("classify_rule_ids".into(), json!(ids));
    }
    if let Some(ev) = &r.classify_evidence {
        m.insert("classify_evidence".into(), ev.clone());
    }
    if let Some(v) = &r.classify_overall_verdict {
        m.insert("classify_overall_verdict".into(), json!(v));
    }
    if let Some(v) = &r.classify_verdict_reason {
        m.insert("classify_verdict_reason".into(), json!(v));
    }
    m.insert("prev_hash".into(), json!(r.prev_hash));
    m.insert("hash".into(), json!(r.hash));
    Value::Object(m)
}

/// Hardening (T-00028): bound request lines so a hostile client cannot
/// balloon server memory with a single giant JSON line. Largest
/// legitimate request is ~70 KiB (4096-byte note + 16×4096 evidence);
/// 1 MiB leaves ample headroom.
const MAX_LINE_BYTES: usize = 1024 * 1024;

enum Line {
    Ok(Vec<u8>),
    TooLong,
    Eof,
}

fn read_line_capped<R: std::io::BufRead>(r: &mut R, cap: usize) -> std::io::Result<Line> {
    let mut buf: Vec<u8> = Vec::with_capacity(512);
    loop {
        let available = r.fill_buf()?;
        if available.is_empty() {
            return Ok(if buf.is_empty() {
                Line::Eof
            } else if buf.len() > cap {
                Line::TooLong
            } else {
                Line::Ok(buf)
            });
        }
        if let Some(pos) = available.iter().position(|&b| b == b'\n') {
            buf.extend_from_slice(&available[..pos]);
            r.consume(pos + 1);
            return Ok(if buf.len() > cap { Line::TooLong } else { Line::Ok(buf) });
        }
        buf.extend_from_slice(available);
        let len = available.len();
        r.consume(len);
        if buf.len() > cap {
            // Over cap: drain through the newline to preserve framing
            // for subsequent requests, without storing any more bytes.
            loop {
                let av = r.fill_buf()?;
                if av.is_empty() {
                    return Ok(Line::TooLong);
                }
                match av.iter().position(|&b| b == b'\n') {
                    Some(pos) => {
                        r.consume(pos + 1);
                        return Ok(Line::TooLong);
                    }
                    None => {
                        let l = av.len();
                        r.consume(l);
                    }
                }
            }
        }
    }
}

fn main() {
    let mut server = Server::open();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut reader = stdin.lock();
    loop {
        let line = match read_line_capped(&mut reader, MAX_LINE_BYTES) {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = match line {
            Line::Eof => break,
            Line::TooLong => {
                let _ = writeln!(
                    out,
                    "{}",
                    json!({"jsonrpc": "2.0", "id": null,
                           "error": {"code": -32700,
                                     "message": format!("request line exceeds {} bytes", MAX_LINE_BYTES)}})
                );
                let _ = out.flush();
                continue;
            }
            Line::Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        };
        let trimmed = trimmed.trim();
        if trimmed.is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let _ = writeln!(
                    out,
                    "{}",
                    json!({"jsonrpc": "2.0", "id": null,
                           "error": {"code": -32700, "message": format!("parse error: {}", e)}})
                );
                let _ = out.flush();
                continue;
            }
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(json!({}));
        let response = match method {
            "initialize" => {
                let protocol_version = params
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or(SCHEMA_VERSION)
                    .to_string();
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": protocol_version,
                        "capabilities": {"tools": {"listChanged": false}},
                        "serverInfo": {"name": "aiosh-mcp", "version": "0.1.0"},
                    }
                })
            }
            "notifications/initialized" => continue, // no response
            "ping" => json!({"jsonrpc": "2.0", "id": id, "result": {}}),
            "tools/list" => {
                let tools = server.tool_manifest();
                json!({"jsonrpc": "2.0", "id": id,
                       "result": {"tools": tools}})
            }
            "tools/call" => {
                let tool = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                if tool == "aios.task" {
                    // Schema violations are protocol errors (-32602);
                    // semantic refusals flow through the gate as
                    // isError:true results (spec §3.3).
                    let response =
                        match aiosh_core::task_service::parse_args(&arguments) {
                            Err(msg) => json!({
                                "jsonrpc": "2.0", "id": id,
                                "error": {"code": -32602, "message": msg}
                            }),
                            Ok(parsed) => {
                                let result = server.call_task(&parsed);
                                let is_error =
                                    result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                                json!({
                                    "jsonrpc": "2.0", "id": id,
                                    "result": {
                                        "content": [{"type": "text", "text": result.to_string()}],
                                        "structuredContent": {"result": result},
                                        "isError": is_error,
                                    }
                                })
                            }
                        };
                    let _ = writeln!(out, "{}", response);
                    let _ = out.flush();
                    continue;
                }
                let result = server.call_tool(tool, &arguments);
                let is_error = result.get("ok").and_then(|v| v.as_bool()) == Some(false);
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{"type": "text", "text": result.to_string()}],
                        "structuredContent": {"result": result},
                        "isError": is_error,
                    }
                })
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("method not found: {}", method)},
            }),
        };
        let _ = writeln!(out, "{}", response);
        let _ = out.flush();
    }
}
