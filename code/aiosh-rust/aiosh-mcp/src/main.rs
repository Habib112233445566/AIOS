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
        tools.push(json!({
            "name": "aios.release.validate",
            "description": "Verify a generated release ISO using its expected hash.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "artifact_path": {"type": "string"},
                    "expected_hash": {"type": "string"}
                },
                "required": ["artifact_path", "expected_hash"]
            }
        }));
        tools.push(json!({
            "name": "aios.backup.validate",
            "description": "Verify the structural integrity of a backup ZIP archive.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "backup_path": {"type": "string"}
                },
                "required": ["backup_path"]
            }
        }));
        tools.push(json!({
            "name": "aios.backup.restore",
            "description": "Extract a backup ZIP archive into a target directory securely. Requires PEP grant.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "backup_path": {"type": "string"},
                    "target_dir": {"type": "string"},
                    "grant_id": {"type": "string"}
                },
                "required": ["backup_path", "target_dir"]
            }
        }));
        tools.push(json!({
            "name": "aios.toolchain.config.get",
            "description": "Get the currently active Dependency & Toolchain Pinning config.",
            "inputSchema": { "type": "object" }
        }));
        tools.push(json!({
            "name": "aios.toolchain.check",
            "description": "Enforce the host toolchain against the manifest configuration.",
            "inputSchema": { "type": "object" }
        }));
        tools.push(json!({
            "name": "aios.doc.index.get",
            "description": "Get the active documentation index catalog.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string" }
                }
            }
        }));
        tools.push(json!({
            "name": "aios.doc.check",
            "description": "Validate markdown link integrity across indexed documentation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string" }
                }
            }
        }));
        tools.push(json!({
            "name": "aios.doc.search",
            "description": "Search indexed documentation entries by keyword.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "repo_path": { "type": "string" }
                },
                "required": ["query"]
            }
        }));
        tools.push(json!({
            "name": "aios.evidence.verify",
            "description": "Verify task evidence files against an evidence manifest.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "manifest_path": { "type": "string" },
                    "repo_path": { "type": "string" }
                }
            }
        }));
        tools.push(json!({
            "name": "aios.evidence.hash",
            "description": "Compute SHA-256 hash of a file on disk.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": { "type": "string" }
                },
                "required": ["file_path"]
            }
        }));
        tools.push(json!({
            "name": "aios.evidence.scan",
            "description": "Scan and discover evidence files in docs/tasks/evidence/.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string" },
                    "task_id": { "type": "integer" }
                }
            }
        }));
        tools.push(json!({
            "name": "aios.repo.health",
            "description": "Assess repository health, Git working tree cleanliness, file bounds, and security governance policies.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Target repository root directory (default: current directory .)." }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.secrets.scan",
            "description": "Scan workspace or specific file for exposed API keys, private keys, and credentials without revealing raw secrets.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Workspace root directory (defaults to .)" },
                    "file_path": { "type": "string", "description": "Specific file path to scan in isolation" },
                    "max_bytes": { "type": "integer", "description": "Maximum file size in bytes to scan (default: 16777216)" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.secrets.check",
            "description": "Fast boolean cleanliness check verifying that no exposed credentials exist in the target workspace.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Workspace root directory (defaults to .)" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.triage.list",
            "description": "List regression triage records with optional status or severity filtering.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "status": { "type": "string", "description": "Filter by status (untriaged, triaged, fix_pending, resolved, wont_fix)" },
                    "severity": { "type": "string", "description": "Filter by severity (blocker, critical, major, minor)" },
                    "store_path": { "type": "string", "description": "Path to triage_store.json" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.triage.show",
            "description": "Show detailed metadata and repro steps for a specific regression by TRG ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Triage record identifier (e.g. TRG-6a1b2c3d)" },
                    "store_path": { "type": "string", "description": "Path to triage_store.json" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.triage.record",
            "description": "Record a test regression finding into the triage store.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "test_target": { "type": "string", "description": "Target test identifier" },
                    "suite_name": { "type": "string", "description": "Name of test suite" },
                    "error_message": { "type": "string", "description": "Error message or panic payload" },
                    "repro_command": { "type": "string", "description": "Command to reproduce failure" },
                    "severity": { "type": "string", "description": "Severity level (blocker, critical, major, minor)" },
                    "store_path": { "type": "string", "description": "Path to triage_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["test_target", "error_message"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.triage.resolve",
            "description": "Mark a regression triage item as resolved with resolution notes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Triage record identifier" },
                    "notes": { "type": "string", "description": "Resolution description and fix notes" },
                    "store_path": { "type": "string", "description": "Path to triage_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["id", "notes"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.triage.check",
            "description": "Check whether any open blocker or critical regressions exist in the triage store.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "store_path": { "type": "string", "description": "Path to triage_store.json" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.handoff.list",
            "description": "List tracked agent handoffs with optional status or active filtering.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "active": { "type": "boolean", "description": "Filter to only active (Pending/Accepted) handoffs" },
                    "status": { "type": "string", "description": "Filter by specific status (pending, accepted, rejected, completed, cancelled, expired)" },
                    "store_path": { "type": "string", "description": "Path to handoff_store.json" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.handoff.show",
            "description": "Show detailed metadata, payload, and status for a specific handoff by HND ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff identifier (e.g. HND-a1b2c3d4)" },
                    "store_path": { "type": "string", "description": "Path to handoff_store.json" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.handoff.initiate",
            "description": "Initiate and enqueue a new handoff between sender and receiver agents.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sender": { "type": "string", "description": "Identifier of the sender agent" },
                    "receiver": { "type": "string", "description": "Identifier of the receiver agent" },
                    "summary": { "type": "string", "description": "Context summary of the handoff" },
                    "task_id": { "type": "integer", "description": "Optional associated task ID" },
                    "payload": { "type": "string", "description": "Optional JSON payload string" },
                    "priority": { "type": "string", "description": "Priority (low, normal, high, urgent)" },
                    "store_path": { "type": "string", "description": "Path to handoff_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["sender", "receiver", "summary"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.handoff.accept",
            "description": "Accept a pending handoff request.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff identifier" },
                    "notes": { "type": "string", "description": "Optional acceptance notes" },
                    "store_path": { "type": "string", "description": "Path to handoff_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.handoff.reject",
            "description": "Reject a pending handoff request.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff identifier" },
                    "notes": { "type": "string", "description": "Optional rejection notes" },
                    "store_path": { "type": "string", "description": "Path to handoff_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.handoff.complete",
            "description": "Mark an accepted handoff as completed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff identifier" },
                    "notes": { "type": "string", "description": "Optional completion notes" },
                    "store_path": { "type": "string", "description": "Path to handoff_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.handoff.cancel",
            "description": "Cancel a pending or accepted handoff request.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff identifier" },
                    "notes": { "type": "string", "description": "Optional cancellation notes" },
                    "store_path": { "type": "string", "description": "Path to handoff_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.distro.list",
            "description": "List registered Linux distribution profiles.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "store_path": { "type": "string", "description": "Optional path to custom distro_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.distro.show",
            "description": "Get detailed distribution profile by ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Distribution profile ID" },
                    "store_path": { "type": "string", "description": "Optional path to custom distro_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.distro.evaluate",
            "description": "Evaluate distribution profiles against AIOS criteria (binary compatibility, footprint, security).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Optional profile ID to evaluate single profile, or omit to evaluate all" },
                    "store_path": { "type": "string", "description": "Optional path to custom distro_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.distro.recommend",
            "description": "Get the recommended Linux distribution profile for AIOS base system.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "store_path": { "type": "string", "description": "Optional path to custom distro_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.distro.policy",
            "description": "Evaluate Linux distribution profiles against AIOS security policy standards.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Optional profile ID to check, or omit to check all registered profiles" },
                    "store_path": { "type": "string", "description": "Optional path to custom distro_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.distro.stats",
            "description": "Get observability and telemetry metrics report for registered Linux distribution profiles.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "store_path": { "type": "string", "description": "Optional path to custom distro_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.distro.check",
            "description": "Validate health and structural integrity of the Linux distribution store.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "store_path": { "type": "string", "description": "Optional path to custom distro_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.image.list",
            "description": "List registered Linux base image manifests with optional format or distro filters.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "format": { "type": "string", "description": "Optional format filter (raw, qcow2, iso, tarball)" },
                    "distro_id": { "type": "string", "description": "Optional distro identifier filter" },
                    "store_path": { "type": "string", "description": "Optional path to custom image_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.image.get",
            "description": "Retrieve detailed base image manifest by ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Image target identifier" },
                    "store_path": { "type": "string", "description": "Optional path to custom image_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.image.plan",
            "description": "Generate reproducible 4-stage build execution plan for base image by ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Image target identifier" },
                    "store_path": { "type": "string", "description": "Optional path to custom image_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.image.config",
            "description": "Get active configuration settings for Linux base image building subsystem.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "config_path": { "type": "string", "description": "Optional path to custom image_config.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools.push(json!({
            "name": "aios.image.policy",
            "description": "Evaluate base image manifests against security policy rules and return compliance verdict.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Optional image target identifier to check a single image" },
                    "store_path": { "type": "string", "description": "Optional path to custom image_store.json" },
                    "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
                },
                "additionalProperties": false
            }
        }));
        tools
    }

    fn call_tool(&mut self, tool: &str, arguments: &Value) -> Value {
        let grant_id = arguments.get("grant_id").and_then(|v| v.as_str());
        match tool {
            "aios.handoff.list" => {
                let active_only = arguments.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                let status_opt = arguments.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/handoff_store.json")
                    .to_string();

                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let (store, _) = aiosh_core::handoff_service::HandoffStore::load_or_recover(path);
                    let records = if active_only {
                        store.list_active()
                    } else {
                        store.list_all()
                    };
                    let filtered: Vec<_> = records.into_iter().filter(|r| {
                        if let Some(ref st) = status_opt {
                            let st_str = format!("{:?}", r.status).to_lowercase();
                            if st_str != st.to_lowercase() {
                                return false;
                            }
                        }
                        true
                    }).collect();
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.handoff.list",
                        "count": filtered.len(),
                        "records": filtered
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.handoff.list", "List handoff records", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.handoff.show" => {
                let id = arguments.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/handoff_store.json")
                    .to_string();

                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let (store, _) = aiosh_core::handoff_service::HandoffStore::load_or_recover(path);
                    let rec = store.get_by_id(&id_for_closure).ok_or_else(|| format!("Record {} not found", id_for_closure))?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.handoff.show",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.handoff.show", &format!("Show handoff record {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.handoff.initiate" => {
                let sender = arguments.get("sender").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let receiver = arguments.get("receiver").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let summary = arguments.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let task_id = arguments.get("task_id").and_then(|v| v.as_u64()).map(|t| t as u32);
                let payload = arguments.get("payload").and_then(|v| v.as_str()).unwrap_or("{}").to_string();
                let priority_str = arguments.get("priority").and_then(|v| v.as_str()).unwrap_or("normal").to_string();
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/handoff_store.json")
                    .to_string();

                let f = move || -> Result<Value, String> {
                    if sender.is_empty() || receiver.is_empty() || summary.is_empty() {
                        return Err("Missing required fields (sender, receiver, summary)".into());
                    }
                    let path = std::path::Path::new(&store_path_str);
                    let (mut store, _) = aiosh_core::handoff_service::HandoffStore::load_or_recover(path);
                    let priority = match priority_str.as_str() {
                        "low" => aiosh_core::handoff::HandoffPriority::Low,
                        "high" => aiosh_core::handoff::HandoffPriority::High,
                        "urgent" => aiosh_core::handoff::HandoffPriority::Urgent,
                        _ => aiosh_core::handoff::HandoffPriority::Normal,
                    };
                    let rec = store.initiate_handoff(&sender, &receiver, task_id, &summary, &payload, priority);
                    store.save_to_path(path)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.handoff.initiate",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.handoff.initiate", "Initiate handoff request", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.handoff.accept" => {
                let id = arguments.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let notes = arguments.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/handoff_store.json")
                    .to_string();

                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let (mut store, _) = aiosh_core::handoff_service::HandoffStore::load_or_recover(path);
                    let rec = store.accept_handoff(&id_for_closure, notes.as_deref())?;
                    store.save_to_path(path)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.handoff.accept",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.handoff.accept", &format!("Accept handoff {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.handoff.reject" => {
                let id = arguments.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let notes = arguments.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/handoff_store.json")
                    .to_string();

                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let (mut store, _) = aiosh_core::handoff_service::HandoffStore::load_or_recover(path);
                    let rec = store.reject_handoff(&id_for_closure, notes.as_deref())?;
                    store.save_to_path(path)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.handoff.reject",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.handoff.reject", &format!("Reject handoff {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.handoff.complete" => {
                let id = arguments.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let notes = arguments.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/handoff_store.json")
                    .to_string();

                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let (mut store, _) = aiosh_core::handoff_service::HandoffStore::load_or_recover(path);
                    let rec = store.complete_handoff(&id_for_closure, notes.as_deref())?;
                    store.save_to_path(path)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.handoff.complete",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.handoff.complete", &format!("Complete handoff {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.handoff.cancel" => {
                let id = arguments.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let notes = arguments.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/handoff_store.json")
                    .to_string();

                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let (mut store, _) = aiosh_core::handoff_service::HandoffStore::load_or_recover(path);
                    let rec = store.cancel_handoff(&id_for_closure, notes.as_deref())?;
                    store.save_to_path(path)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.handoff.cancel",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.handoff.cancel", &format!("Cancel handoff {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.distro.list" => {
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::distro_service::DistroStore::load_from_path(std::path::Path::new(p))?,
                        None => {
                            let cfg = aiosh_core::distro_config::DistroConfig::from_env().unwrap_or_default();
                            aiosh_core::distro_service::DistroStore::load_from_config(&cfg)?
                        }
                    };
                    let profiles = store.list_profiles();
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.distro.list",
                        "count": profiles.len(),
                        "profiles": profiles
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.distro.list", "List distro profiles", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.distro.show" => {
                let id = match arguments.get("id").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => return json!({ "ok": false, "error": "Missing required field 'id'" }),
                };
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::distro_service::DistroStore::load_from_path(std::path::Path::new(p))?,
                        None => {
                            let cfg = aiosh_core::distro_config::DistroConfig::from_env().unwrap_or_default();
                            aiosh_core::distro_service::DistroStore::load_from_config(&cfg)?
                        }
                    };
                    match store.get_profile(&id_for_closure) {
                        Some(profile) => Ok(json!({
                            "ok": true,
                            "tool": "aios.distro.show",
                            "profile": profile
                        })),
                        None => Err(format!("Distro profile '{}' not found", id_for_closure)),
                    }
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.distro.show", &format!("Show distro profile {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.distro.evaluate" => {
                let id_opt = arguments.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::distro_service::DistroStore::load_from_path(std::path::Path::new(p))?,
                        None => {
                            let cfg = aiosh_core::distro_config::DistroConfig::from_env().unwrap_or_default();
                            aiosh_core::distro_service::DistroStore::load_from_config(&cfg)?
                        }
                    };
                    if let Some(ref id) = id_opt {
                        let ev = store.evaluate_profile(id)?;
                        Ok(json!({
                            "ok": true,
                            "tool": "aios.distro.evaluate",
                            "evaluation": ev
                        }))
                    } else {
                        let evals = store.evaluate_all();
                        Ok(json!({
                            "ok": true,
                            "tool": "aios.distro.evaluate",
                            "evaluations": evals
                        }))
                    }
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.distro.evaluate", "Evaluate distro profiles", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.distro.recommend" => {
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::distro_service::DistroStore::load_from_path(std::path::Path::new(p))?,
                        None => {
                            let cfg = aiosh_core::distro_config::DistroConfig::from_env().unwrap_or_default();
                            aiosh_core::distro_service::DistroStore::load_from_config(&cfg)?
                        }
                    };
                    match store.get_recommended_profile() {
                        Some(profile) => Ok(json!({
                            "ok": true,
                            "tool": "aios.distro.recommend",
                            "profile": profile
                        })),
                        None => Err("No recommended distribution profile found".into()),
                    }
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.distro.recommend", "Get recommended distro profile", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.distro.policy" => {
                let id_opt = arguments.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::distro_service::DistroStore::load_from_path(std::path::Path::new(p))?,
                        None => {
                            let cfg = aiosh_core::distro_config::DistroConfig::from_env().unwrap_or_default();
                            aiosh_core::distro_service::DistroStore::load_from_config(&cfg)?
                        }
                    };
                    let policy = aiosh_core::distro_policy::DistroSecurityPolicy::from_env()?;
                    if let Some(ref id) = id_opt {
                        let profile = store.get_profile(id).ok_or_else(|| format!("Distro profile '{}' not found", id))?;
                        let eval = store.evaluate_profile(id)?;
                        let verdict = policy.check_profile(&profile, &eval);
                        Ok(json!({
                            "ok": true,
                            "tool": "aios.distro.policy",
                            "verdict": verdict
                        }))
                    } else {
                        let verdicts = store.check_security_policy(&policy);
                        Ok(json!({
                            "ok": true,
                            "tool": "aios.distro.policy",
                            "verdicts": verdicts
                        }))
                    }
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.distro.policy", "Check distro security policy", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.distro.stats" => {
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::distro_service::DistroStore::load_from_path(std::path::Path::new(p))?,
                        None => {
                            let cfg = aiosh_core::distro_config::DistroConfig::from_env().unwrap_or_default();
                            aiosh_core::distro_service::DistroStore::load_from_config(&cfg)?
                        }
                    };
                    let policy_opt = aiosh_core::distro_policy::DistroSecurityPolicy::from_env().ok();
                    let report = store.get_observability_report(policy_opt.as_ref());
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.distro.stats",
                        "report": report
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.distro.stats", "Get distro observability metrics", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.distro.check" => {
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::distro_service::DistroStore::load_from_path(std::path::Path::new(p))?,
                        None => {
                            let cfg = aiosh_core::distro_config::DistroConfig::from_env().unwrap_or_default();
                            aiosh_core::distro_service::DistroStore::load_from_config(&cfg)?
                        }
                    };
                    let report = store.validate_health();
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.distro.check",
                        "report": report
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.distro.check", "Validate distro store health", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.image.list" => {
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                if let Some(ref p) = store_path_opt {
                    if p.len() > 4096 {
                        return json!({ "ok": false, "error": "store_path exceeds maximum length of 4096 characters" });
                    }
                }
                let format_opt = arguments.get("format").and_then(|v| v.as_str()).map(|s| s.to_string());
                let distro_opt = arguments.get("distro_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::base_image_service::ImageStore::load_from_path(std::path::Path::new(p))?,
                        None => aiosh_core::base_image_service::ImageStore::new(),
                    };
                    let mut images = store.list_images();
                    if let Some(ref fmt_str) = format_opt {
                        let fmt = match fmt_str.to_lowercase().as_str() {
                            "raw" => aiosh_core::base_image::ImageFormat::Raw,
                            "qcow2" => aiosh_core::base_image::ImageFormat::Qcow2,
                            "iso" => aiosh_core::base_image::ImageFormat::Iso,
                            "tarball" | "tar" => aiosh_core::base_image::ImageFormat::Tarball,
                            other => return Err(format!("Unknown image format '{}'", other)),
                        };
                        images.retain(|img| img.format == fmt);
                    }
                    if let Some(ref distro) = distro_opt {
                        images.retain(|img| img.rootfs.distro_id == *distro);
                    }
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.image.list",
                        "images": images,
                        "count": images.len()
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.image.list", "List base image manifests", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.image.get" => {
                let id = match arguments.get("id").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => return json!({ "ok": false, "error": "Missing required field 'id'" }),
                };
                if id.is_empty() || id.len() > 128 || !id.chars().all(|c| c.is_ascii_graphic()) {
                    return json!({ "ok": false, "error": "Invalid image id: must be 1..128 printable ASCII characters" });
                }
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                if let Some(ref p) = store_path_opt {
                    if p.len() > 4096 {
                        return json!({ "ok": false, "error": "store_path exceeds maximum length of 4096 characters" });
                    }
                }
                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::base_image_service::ImageStore::load_from_path(std::path::Path::new(p))?,
                        None => aiosh_core::base_image_service::ImageStore::new(),
                    };
                    match store.get_image(&id_for_closure) {
                        Some(img) => Ok(json!({
                            "ok": true,
                            "tool": "aios.image.get",
                            "image": img
                        })),
                        None => Err(format!("Base image '{}' not found", id_for_closure)),
                    }
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.image.get", &format!("Get base image manifest {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.image.plan" => {
                let id = match arguments.get("id").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => return json!({ "ok": false, "error": "Missing required field 'id'" }),
                };
                if id.is_empty() || id.len() > 128 || !id.chars().all(|c| c.is_ascii_graphic()) {
                    return json!({ "ok": false, "error": "Invalid image id: must be 1..128 printable ASCII characters" });
                }
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                if let Some(ref p) = store_path_opt {
                    if p.len() > 4096 {
                        return json!({ "ok": false, "error": "store_path exceeds maximum length of 4096 characters" });
                    }
                }
                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::base_image_service::ImageStore::load_from_path(std::path::Path::new(p))?,
                        None => aiosh_core::base_image_service::ImageStore::new(),
                    };
                    let plan = store.generate_build_plan(&id_for_closure)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.image.plan",
                        "plan": plan
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.image.plan", &format!("Generate build plan for image {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.image.config" => {
                let config_path_opt = arguments.get("config_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                if let Some(ref p) = config_path_opt {
                    if p.len() > 4096 {
                        return json!({ "ok": false, "error": "config_path exceeds maximum length of 4096 characters" });
                    }
                }
                let f = move || -> Result<Value, String> {
                    let config = match config_path_opt {
                        Some(ref p) => aiosh_core::base_image_config::ImageBuildConfig::from_file(std::path::Path::new(p))?,
                        None => aiosh_core::base_image_config::ImageBuildConfig::from_env()?,
                    };
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.image.config",
                        "config": config
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.image.config", "Get base image build configuration", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.image.policy" => {
                let id_opt = arguments.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                if let Some(ref id) = id_opt {
                    if id.is_empty() || id.len() > 128 || !id.chars().all(|c| c.is_ascii_graphic()) {
                        return json!({ "ok": false, "error": "Invalid image id: must be 1..128 printable ASCII characters" });
                    }
                }
                if let Some(ref p) = store_path_opt {
                    if p.len() > 4096 {
                        return json!({ "ok": false, "error": "store_path exceeds maximum length of 4096 characters" });
                    }
                }
                let id_for_closure = id_opt.clone();
                let f = move || -> Result<Value, String> {
                    let store = match store_path_opt {
                        Some(ref p) => aiosh_core::base_image_service::ImageStore::load_from_path(std::path::Path::new(p))?,
                        None => aiosh_core::base_image_service::ImageStore::new(),
                    };
                    let policy = aiosh_core::base_image_policy::BaseImageSecurityPolicy::from_env()?;
                    if let Some(ref id) = id_for_closure {
                        match store.get_image(id) {
                            Some(img) => {
                                let verdict = policy.evaluate(img);
                                Ok(json!({
                                    "ok": true,
                                    "tool": "aios.image.policy",
                                    "verdict": verdict
                                }))
                            }
                            None => Err(format!("Base image '{}' not found", id)),
                        }
                    } else {
                        let verdicts = policy.check_all(&store);
                        Ok(json!({
                            "ok": true,
                            "tool": "aios.image.policy",
                            "verdicts": verdicts,
                            "count": verdicts.len()
                        }))
                    }
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.image.policy", "Evaluate base image security policy", arguments,
                    id_opt.as_deref(), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.triage.list" => {
                let status_opt = arguments.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
                let severity_opt = arguments.get("severity").and_then(|v| v.as_str()).map(|s| s.to_string());
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/triage_store.json")
                    .to_string();

                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let store = aiosh_core::triage_service::TriageStore::load_from_path(path)?;
                    let report = store.to_report();
                    let filtered: Vec<_> = report.records.into_iter().filter(|r| {
                        if let Some(ref st) = status_opt {
                            let st_str = match r.status {
                                aiosh_core::triage::TriageStatus::Untriaged => "untriaged",
                                aiosh_core::triage::TriageStatus::Triaged => "triaged",
                                aiosh_core::triage::TriageStatus::FixPending => "fix_pending",
                                aiosh_core::triage::TriageStatus::Resolved => "resolved",
                                aiosh_core::triage::TriageStatus::WontFix => "wont_fix",
                            };
                            if st_str != st.to_lowercase() {
                                return false;
                            }
                        }
                        if let Some(ref sv) = severity_opt {
                            let sv_str = match r.severity {
                                aiosh_core::triage::TriageSeverity::Blocker => "blocker",
                                aiosh_core::triage::TriageSeverity::Critical => "critical",
                                aiosh_core::triage::TriageSeverity::Major => "major",
                                aiosh_core::triage::TriageSeverity::Minor => "minor",
                            };
                            if sv_str != sv.to_lowercase() {
                                return false;
                            }
                        }
                        true
                    }).collect();
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.triage.list",
                        "count": filtered.len(),
                        "records": filtered
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.triage.list", "List triage records", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.triage.show" => {
                let id = arguments.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/triage_store.json")
                    .to_string();

                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let store = aiosh_core::triage_service::TriageStore::load_from_path(path)?;
                    let rec = store.get_by_id(&id_for_closure).ok_or_else(|| format!("Record {} not found", id_for_closure))?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.triage.show",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.triage.show", &format!("Show triage record {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.triage.record" => {
                let test_target = arguments.get("test_target").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let suite_name = arguments.get("suite_name").and_then(|v| v.as_str()).unwrap_or("mcp").to_string();
                let error_message = arguments.get("error_message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let repro_command = arguments.get("repro_command").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let severity_str = arguments.get("severity").and_then(|v| v.as_str()).unwrap_or("critical").to_string();
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/triage_store.json")
                    .to_string();

                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let mut store = aiosh_core::triage_service::TriageStore::load_from_path(path)?;
                    let sev = match severity_str.as_str() {
                        "blocker" => aiosh_core::triage::TriageSeverity::Blocker,
                        "major" => aiosh_core::triage::TriageSeverity::Major,
                        "minor" => aiosh_core::triage::TriageSeverity::Minor,
                        _ => aiosh_core::triage::TriageSeverity::Critical,
                    };
                    let rec = store.record_failure(&test_target, &suite_name, &error_message, &repro_command, sev);
                    store.save_to_path(path)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.triage.record",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.triage.record", "Record triage finding", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.triage.resolve" => {
                let id = arguments.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let notes = arguments.get("notes").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/triage_store.json")
                    .to_string();

                let id_for_closure = id.clone();
                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let mut store = aiosh_core::triage_service::TriageStore::load_from_path(path)?;
                    let rec = store.resolve(&id_for_closure, &notes)?.clone();
                    store.save_to_path(path)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.triage.resolve",
                        "record": rec
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.triage.resolve", &format!("Resolve triage record {}", id), arguments,
                    Some(&id), grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.triage.check" => {
                let store_path_str = arguments
                    .get("store_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".aios/triage_store.json")
                    .to_string();

                let f = move || -> Result<Value, String> {
                    let path = std::path::Path::new(&store_path_str);
                    let store = aiosh_core::triage_service::TriageStore::load_from_path(path)?;
                    let report = store.to_report();
                    let mut blocker_count = 0;
                    let mut critical_count = 0;

                    for r in &report.records {
                        if r.status != aiosh_core::triage::TriageStatus::Resolved && r.status != aiosh_core::triage::TriageStatus::WontFix {
                            match r.severity {
                                aiosh_core::triage::TriageSeverity::Blocker => blocker_count += 1,
                                aiosh_core::triage::TriageSeverity::Critical => critical_count += 1,
                                _ => {}
                            }
                        }
                    }

                    let clean = blocker_count == 0 && critical_count == 0;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.triage.check",
                        "clean": clean,
                        "total_records": report.total_records,
                        "open_records": report.open_records,
                        "blocker_open": blocker_count,
                        "critical_open": critical_count
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.triage.check", "Check triage cleanliness", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.secrets.scan" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let file_path = arguments.get("file_path").and_then(|v| v.as_str());
                let max_bytes = arguments
                    .get("max_bytes")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(aiosh_core::secrets_service::DEFAULT_MAX_SECRET_FILE_BYTES);

                let repo = std::path::Path::new(repo_path);
                let f = || -> Result<Value, String> {
                    let report = if let Some(fp) = file_path {
                        let p = std::path::Path::new(fp);
                        let findings = aiosh_core::secrets_service::scan_file_for_secrets(p, repo, max_bytes)?;
                        aiosh_core::secrets::SecretScanReport::new(p.to_string_lossy().to_string(), findings, 1)
                    } else {
                        aiosh_core::secrets_service::scan_workspace_for_secrets(
                            repo,
                            max_bytes,
                            aiosh_core::secrets_service::DEFAULT_IGNORED_DIRS,
                        )?
                    };
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.secrets.scan",
                        "report": report
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.secrets.scan", "Scan workspace for secrets", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.secrets.check" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let repo = std::path::Path::new(repo_path);
                let f = || -> Result<Value, String> {
                    let report = aiosh_core::secrets_service::scan_workspace_for_secrets(
                        repo,
                        aiosh_core::secrets_service::DEFAULT_MAX_SECRET_FILE_BYTES,
                        aiosh_core::secrets_service::DEFAULT_IGNORED_DIRS,
                    )?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.secrets.check",
                        "is_clean": report.is_clean,
                        "total_findings": report.total_findings,
                        "report": report
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.secrets.check", "Check workspace for secrets", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.repo.health" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let repo = std::path::Path::new(repo_path);
                let f = || -> Result<Value, String> {
                    let report = aiosh_core::repo_health_service::check_repo_health(repo)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.repo.health",
                        "report": report
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.repo.health", "Assess repository health", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.doc.index.get" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let config_path = arguments.get("config_path").and_then(|v| v.as_str());
                let repo = std::path::Path::new(repo_path);
                let default_docs = &["docs/README.md", "docs/SPEC-TASK-LEDGER.md", "docs/tasks/GOALS.md"];
                let f = || -> Result<Value, String> {
                    let _config = match config_path {
                        Some(p) => aiosh_core::doc_index_config::DocIndexConfig::from_path(std::path::Path::new(p))?,
                        None => aiosh_core::doc_index_config::DocIndexConfig::from_env().unwrap_or_default(),
                    };
                    let manifest = aiosh_core::doc_index_service::build_doc_index_from_paths(repo, default_docs)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.doc.index.get",
                        "manifest": manifest
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.doc.index.get", "Get doc index manifest", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.doc.check" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let config_path = arguments.get("config_path").and_then(|v| v.as_str());
                let repo = std::path::Path::new(repo_path);
                let default_docs = &["docs/README.md", "docs/SPEC-TASK-LEDGER.md", "docs/tasks/GOALS.md"];
                let f = || -> Result<Value, String> {
                    let _config = match config_path {
                        Some(p) => aiosh_core::doc_index_config::DocIndexConfig::from_path(std::path::Path::new(p))?,
                        None => aiosh_core::doc_index_config::DocIndexConfig::from_env().unwrap_or_default(),
                    };
                    let (_manifest, report, telemetry) =
                        aiosh_core::doc_index_service::reconcile_doc_index(repo, default_docs)?;
                    Ok(json!({
                        "ok": report.is_valid,
                        "tool": "aios.doc.check",
                        "report": report,
                        "telemetry": telemetry
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.doc.check", "Check doc links", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.doc.search" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let config_path = arguments.get("config_path").and_then(|v| v.as_str());
                let query = arguments
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                let repo = std::path::Path::new(repo_path);
                let default_docs = &["docs/README.md", "docs/SPEC-TASK-LEDGER.md", "docs/tasks/GOALS.md"];
                let f = || -> Result<Value, String> {
                    if query.is_empty() {
                        return Err("query argument is required".into());
                    }
                    let _config = match config_path {
                        Some(p) => aiosh_core::doc_index_config::DocIndexConfig::from_path(std::path::Path::new(p))?,
                        None => aiosh_core::doc_index_config::DocIndexConfig::from_env().unwrap_or_default(),
                    };
                    let manifest = aiosh_core::doc_index_service::build_doc_index_from_paths(repo, default_docs)?;
                    let matches: Vec<_> = manifest.entries.into_iter().filter(|e| {
                        e.title.to_lowercase().contains(&query) ||
                        e.path.to_lowercase().contains(&query) ||
                        e.section.to_lowercase().contains(&query)
                    }).collect();
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.doc.search",
                        "matches": matches
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.doc.search", "Search doc index", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.evidence.verify" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let manifest_path = arguments.get("manifest_path").and_then(|v| v.as_str());
                let repo = std::path::Path::new(repo_path);
                let f = || -> Result<Value, String> {
                    let manifest = match manifest_path {
                        Some(p) => {
                            let content = std::fs::read_to_string(p)
                                .map_err(|e| format!("Failed to read manifest file {}: {}", p, e))?;
                            aiosh_core::evidence::TaskEvidenceManifest::from_json(&content)?
                        }
                        None => aiosh_core::evidence::TaskEvidenceManifest::default(),
                    };
                    let report = aiosh_core::evidence_service::verify_evidence_manifest(repo, &manifest)?;
                    Ok(json!({
                        "ok": report.is_valid,
                        "tool": "aios.evidence.verify",
                        "report": report
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.evidence.verify", "Verify evidence files", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.evidence.hash" => {
                let file_path = arguments
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let f = move || -> Result<Value, String> {
                    if file_path.is_empty() {
                        return Err("file_path argument is required".into());
                    }
                    let hash = aiosh_core::evidence_service::compute_file_sha256(std::path::Path::new(&file_path))?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.evidence.hash",
                        "file_path": file_path,
                        "sha256": hash
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.evidence.hash", "Compute file SHA-256 hash", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.evidence.scan" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".")
                    .to_string();
                let task_filter = arguments
                    .get("task_id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);
                let f = move || -> Result<Value, String> {
                    let repo = std::path::Path::new(&repo_path);
                    let evidence_dir = repo.join("docs/tasks/evidence");
                    if !evidence_dir.exists() {
                        return Err(format!("Evidence directory not found: {}", evidence_dir.display()));
                    }
                    let mut records = Vec::new();
                    if let Ok(entries) = std::fs::read_dir(&evidence_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                                if file_name.starts_with('T') && file_name.contains('-') {
                                    let parts: Vec<&str> = file_name.split('-').collect();
                                    if parts.len() >= 2 {
                                        if let Ok(tid) = parts[1].parse::<u32>() {
                                            if task_filter.map_or(true, |target| target == tid) {
                                                let rel_path = format!("docs/tasks/evidence/{}", file_name);
                                                if let Ok(hash) = aiosh_core::evidence_service::compute_file_sha256(&path) {
                                                    records.push(json!({
                                                        "task_id": tid,
                                                        "file_path": rel_path,
                                                        "sha256": hash
                                                    }));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.evidence.scan",
                        "records": records
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.evidence.scan", "Scan evidence directory", arguments,
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.toolchain.config.get" => {
                let f = || -> Result<Value, String> {
                    let manifest = aiosh_core::toolchain_config::ToolchainManifest::from_env()?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.toolchain.config.get",
                        "config": manifest.to_json_with_sources()
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.toolchain.config.get", "Get toolchain config", &json!({}),
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.toolchain.check" => {
                let f = || -> Result<Value, String> {
                    let manifest = aiosh_core::toolchain_config::ToolchainManifest::from_env()?;
                    aiosh_core::toolchain_service::enforce_toolchain(&manifest)?;
                    Ok(json!({
                        "ok": true,
                        "tool": "aios.toolchain.check",
                        "message": "Toolchain validated successfully."
                    }))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.toolchain.check", "Check toolchain", &json!({}),
                    None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
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
            "aios.release.validate" => {
                let artifact_path = arguments.get("artifact_path").and_then(|v| v.as_str()).unwrap_or("");
                let expected_hash = arguments.get("expected_hash").and_then(|v| v.as_str()).unwrap_or("");
                let f = || -> Result<Value, String> {
                    aiosh_core::release::validate_release(artifact_path, expected_hash)?;
                    Ok(json!({"ok": true, "message": "Release validation passed"}))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.release.validate", &format!("Validate release {}", artifact_path),
                    arguments, Some(artifact_path), grant_id, false,
                    dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.backup.validate" => {
                let backup_path = arguments.get("backup_path").and_then(|v| v.as_str()).unwrap_or("");
                let f = || -> Result<Value, String> {
                    aiosh_core::release::validate_backup(backup_path)?;
                    Ok(json!({"ok": true, "message": "Backup validation passed"}))
                };
                dispatch::recorded_call(
                    &mut self.ring, &self.pep,
                    "aios.backup.validate", &format!("Validate backup {}", backup_path),
                    arguments, Some(backup_path), grant_id, false,
                    dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
                )
            }
            "aios.backup.restore" => {
                let backup_path = arguments.get("backup_path").and_then(|v| v.as_str()).unwrap_or("");
                let target_dir = arguments.get("target_dir").and_then(|v| v.as_str()).unwrap_or("");
                // Restore backup uses its own internal AuditRow emission via check_release_policy,
                // but because MCP routes through recorded_call we can use the inner closure pattern, 
                // but wait, recorded_call also emits!
                // To avoid double emission, we should not use recorded_call, or we should use it and remove the internal emission.
                // Wait, restore_backup emits a row directly. If we just call it, we don't need recorded_call.
                // Actually, let's just call it directly and return the Result.
                match aiosh_core::release::restore_backup(&mut aiosh_core::release::ReleaseCtx {
                    ring: &mut self.ring,
                    actor_id: dispatch::DEFAULT_ACTOR_ID,
                    constitution_rev: "v0.0",
                }, backup_path, target_dir, grant_id) {
                    Ok(_) => json!({"ok": true, "message": format!("Restored {} to {}", backup_path, target_dir)}),
                    Err(e) => json!({"ok": false, "error": e}),
                }
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
                // Pre-compute the live walk OUTSIDE the closure so the
                // closure never borrows self.ring (recorded_call needs
                // &mut for its own audit writes).
                let live = if full {
                    None
                } else {
                    self.ring.verify().ok()
                };
                let db_path = self.ring.path().to_string();
                let f = move || -> Result<Value, String> {
                    let result = if full {
                        let conn = if db_path == ":memory:" {
                            rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())?
                        } else {
                            rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?
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
                    } else {
                        let res = live.as_ref().ok_or_else(|| "audit.verify failed".to_string())?;
                        json!({
                            "ok": res.ok, "mode": "live",
                            "checked": res.checked,
                            "segments": res.segments,
                            "anchor": res.anchor,
                            "broken_at": res.broken_at,
                        })
                    };
                    let ok = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                    let mut out = result;
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
                let grant_owned = grant_id.map(|s| s.to_string());
                let db_path = self.ring.path().to_string();
                let conn = if db_path == ":memory:" {
                    rusqlite::Connection::open_in_memory().unwrap()
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
                        grant_token: grant_owned.clone(),
                        constitution_rev: Some(self.constitution_rev.clone()),
                        ..Default::default()
                    },
                ) {
                    Ok(res) => {
                        let mut out = res.to_json();
                        out["tool"] = json!("audit.rotate");
                        out["classifier_policy_revision"] = json!(verdict.policy_revision);
                        if !res.ok {
                            // retention.rotate already wrote its own refusal row.
                            out["gate"] = json!("retention");
                        }
                        out
                    }
                    Err(e) => {
                        let row = dispatch::commit(
                            &mut self.ring, "audit.rotate", "audit.rotate",
                            &json!({"keep_rows": keep_rows}), None, grant_owned.as_deref(),
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
                let f = move || -> Result<Value, String> {
                    let conn = if db_path == ":memory:" {
                        rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())?
                    } else {
                        rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?
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
                let hash = arguments.get("hash").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let exact = arguments.get("exact").and_then(|v| v.as_bool()).unwrap_or(false);
                let db_path = self.ring.path().to_string();
                let hash_for_closure = hash.clone();
                let f = move || -> Result<Value, String> {
                    let conn = if db_path == ":memory:" {
                        rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())?
                    } else {
                        rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?
                    };
                    let res = retention::seen(&conn, &hash_for_closure, exact, None).map_err(|e| e.to_string())?;
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
                // MCP spec: respond with the version the client asked for
                // when we support it; otherwise advertise OUR latest
                // supported version (never echo an unknown string).
                let requested = params
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or(SCHEMA_VERSION);
                let protocol_version = if requested == SCHEMA_VERSION {
                    requested.to_string()
                } else {
                    SCHEMA_VERSION.to_string()
                };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchain_tools_in_manifest() {
        let server = Server::open();
        let tools = server.tool_manifest();
        
        let tool_names: Vec<&str> = tools.iter()
            .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
            .collect();
            
        assert!(tool_names.contains(&"aios.toolchain.config.get"));
        assert!(tool_names.contains(&"aios.toolchain.check"));
        assert!(tool_names.contains(&"aios.doc.index.get"));
        assert!(tool_names.contains(&"aios.doc.check"));
        assert!(tool_names.contains(&"aios.doc.search"));
        assert!(tool_names.contains(&"aios.evidence.verify"));
        assert!(tool_names.contains(&"aios.evidence.hash"));
        assert!(tool_names.contains(&"aios.evidence.scan"));
        assert!(tool_names.contains(&"aios.repo.health"));
        assert!(tool_names.contains(&"aios.secrets.scan"));
        assert!(tool_names.contains(&"aios.secrets.check"));
        assert!(tool_names.contains(&"aios.distro.list"));
        assert!(tool_names.contains(&"aios.distro.show"));
        assert!(tool_names.contains(&"aios.distro.evaluate"));
        assert!(tool_names.contains(&"aios.distro.recommend"));
    }

    #[test]
    fn test_mcp_secrets_tools_execution() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
        let repo_str = repo_root.to_string_lossy().to_string();

        let mut server = Server::open();

        // 1. aios.secrets.scan single file
        let cargo_toml = repo_root.join("code/aiosh-rust/Cargo.toml").to_string_lossy().to_string();
        let res_scan_file = server.call_tool("aios.secrets.scan", &json!({"file_path": cargo_toml, "repo_path": repo_str}));
        assert_eq!(res_scan_file.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert!(res_scan_file.get("report").is_some());

        // 2. aios.secrets.check
        let res_check = server.call_tool("aios.secrets.check", &json!({"repo_path": repo_str}));
        assert_eq!(res_check.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert!(res_check.get("report").is_some());
    }

    #[test]
    fn test_mcp_repo_health_execution() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
        let repo_str = repo_root.to_string_lossy().to_string();

        let mut server = Server::open();

        let res = server.call_tool("aios.repo.health", &json!({"repo_path": repo_str}));
        assert_eq!(res.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert!(res.get("report").is_some());
    }

    #[test]
    fn test_mcp_doc_tools_execution() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
        let repo_str = repo_root.to_string_lossy().to_string();

        let mut server = Server::open();

        // 1. aios.doc.index.get
        let res_get = server.call_tool("aios.doc.index.get", &json!({"repo_path": repo_str}));
        assert_eq!(res_get.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 2. aios.doc.check
        let res_check = server.call_tool("aios.doc.check", &json!({"repo_path": repo_str}));
        assert_eq!(res_check.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 3. aios.doc.search
        let res_search = server.call_tool("aios.doc.search", &json!({"query": "task", "repo_path": repo_str}));
        assert_eq!(res_search.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 4. aios.doc.search missing query negative test
        let res_search_err = server.call_tool("aios.doc.search", &json!({"repo_path": repo_str}));
        assert_eq!(res_search_err.get("ok").and_then(|v| v.as_bool()), Some(false));

        // 5. aios.evidence.hash
        let readme_path = repo_root.join("docs/README.md").to_string_lossy().to_string();
        let res_hash = server.call_tool("aios.evidence.hash", &json!({"file_path": readme_path}));
        assert_eq!(res_hash.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert!(res_hash.get("sha256").is_some());

        // 6. aios.evidence.verify default manifest
        let res_ev_verify = server.call_tool("aios.evidence.verify", &json!({"repo_path": repo_str}));
        assert_eq!(res_ev_verify.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 7. aios.evidence.scan
        let res_ev_scan = server.call_tool("aios.evidence.scan", &json!({"repo_path": repo_str, "task_id": 501}));
        assert_eq!(res_ev_scan.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert!(res_ev_scan.get("records").is_some());
    }

    #[test]
    fn test_mcp_triage_tools() {
        let store_file = std::env::temp_dir().join(format!("aios_mcp_triage_test_{}.json", std::process::id()));
        let store_str = store_file.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&store_file);

        let mut server = Server::open();

        // 1. aios.triage.list empty
        let res_list_empty = server.call_tool("aios.triage.list", &json!({"store_path": store_str}));
        assert_eq!(res_list_empty.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_list_empty.get("count").and_then(|v| v.as_i64()), Some(0));

        // 2. aios.triage.record
        let res_rec = server.call_tool("aios.triage.record", &json!({
            "test_target": "secrets::test_scan",
            "suite_name": "secrets",
            "error_message": "panic at check",
            "repro_command": "cargo test",
            "severity": "critical",
            "store_path": store_str
        }));
        assert_eq!(res_rec.get("ok").and_then(|v| v.as_bool()), Some(true));
        let rec_id = res_rec.get("record").and_then(|r| r.get("id")).and_then(|v| v.as_str()).unwrap().to_string();

        // 3. aios.triage.show
        let res_show = server.call_tool("aios.triage.show", &json!({
            "id": rec_id,
            "store_path": store_str
        }));
        assert_eq!(res_show.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 4. aios.triage.check (not clean due to open critical)
        let res_chk_fail = server.call_tool("aios.triage.check", &json!({"store_path": store_str}));
        assert_eq!(res_chk_fail.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_chk_fail.get("clean").and_then(|v| v.as_bool()), Some(false));

        // 5. aios.triage.resolve
        let res_resolve = server.call_tool("aios.triage.resolve", &json!({
            "id": rec_id,
            "notes": "Fixed in patch",
            "store_path": store_str
        }));
        assert_eq!(res_resolve.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 6. aios.triage.check (now clean)
        let res_chk_pass = server.call_tool("aios.triage.check", &json!({"store_path": store_str}));
        assert_eq!(res_chk_pass.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_chk_pass.get("clean").and_then(|v| v.as_bool()), Some(true));

        let _ = std::fs::remove_file(&store_file);
    }

    #[test]
    fn test_mcp_handoff_tools() {
        let store_file = std::env::temp_dir().join(format!("aios_mcp_handoff_test_{}.json", std::process::id()));
        let store_str = store_file.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&store_file);

        let mut server = Server::open();

        // 1. aios.handoff.list empty
        let res_list_empty = server.call_tool("aios.handoff.list", &json!({"store_path": store_str}));
        assert_eq!(res_list_empty.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_list_empty.get("count").and_then(|v| v.as_i64()), Some(0));

        // 2. aios.handoff.initiate
        let res_init = server.call_tool("aios.handoff.initiate", &json!({
            "sender": "operator",
            "receiver": "agent-1",
            "summary": "Execute sub-task",
            "priority": "high",
            "store_path": store_str
        }));
        assert_eq!(res_init.get("ok").and_then(|v| v.as_bool()), Some(true));
        let rec_id = res_init.get("record").and_then(|r| r.get("id")).and_then(|v| v.as_str()).unwrap().to_string();

        // 3. aios.handoff.show
        let res_show = server.call_tool("aios.handoff.show", &json!({
            "id": rec_id,
            "store_path": store_str
        }));
        assert_eq!(res_show.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 4. aios.handoff.accept
        let res_accept = server.call_tool("aios.handoff.accept", &json!({
            "id": rec_id,
            "notes": "Accepted task",
            "store_path": store_str
        }));
        assert_eq!(res_accept.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 5. aios.handoff.complete
        let res_complete = server.call_tool("aios.handoff.complete", &json!({
            "id": rec_id,
            "notes": "Task done",
            "store_path": store_str
        }));
        assert_eq!(res_complete.get("ok").and_then(|v| v.as_bool()), Some(true));

        let _ = std::fs::remove_file(&store_file);
    }

    #[test]
    fn test_mcp_distro_tools() {
        let mut server = Server::open();

        // 1. aios.distro.list
        let res_list = server.call_tool("aios.distro.list", &json!({}));
        assert_eq!(res_list.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert!(res_list.get("count").and_then(|v| v.as_u64()).unwrap_or(0) >= 2);

        // 2. aios.distro.show
        let res_show = server.call_tool("aios.distro.show", &json!({ "id": "debian-12-minimal-x86_64" }));
        assert_eq!(res_show.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 3. aios.distro.evaluate
        let res_eval = server.call_tool("aios.distro.evaluate", &json!({}));
        assert_eq!(res_eval.get("ok").and_then(|v| v.as_bool()), Some(true));

        let res_eval_one = server.call_tool("aios.distro.evaluate", &json!({ "id": "alpine-319-container-x86_64" }));
        assert_eq!(res_eval_one.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 4. aios.distro.recommend
        let res_rec = server.call_tool("aios.distro.recommend", &json!({}));
        assert_eq!(res_rec.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 5. aios.distro.policy
        let res_policy = server.call_tool("aios.distro.policy", &json!({}));
        assert_eq!(res_policy.get("ok").and_then(|v| v.as_bool()), Some(true));
        let res_policy_one = server.call_tool("aios.distro.policy", &json!({ "id": "debian-12-minimal-x86_64" }));
        assert_eq!(res_policy_one.get("ok").and_then(|v| v.as_bool()), Some(true));

        // 6. aios.distro.stats
        let res_stats = server.call_tool("aios.distro.stats", &json!({}));
        assert_eq!(res_stats.get("ok").and_then(|v| v.as_bool()), Some(true));
        let total = res_stats.pointer("/report/total_profiles").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(total >= 2);

        // 7. aios.distro.check
        let res_check = server.call_tool("aios.distro.check", &json!({}));
        assert_eq!(res_check.get("ok").and_then(|v| v.as_bool()), Some(true));
        let healthy = res_check.pointer("/report/healthy").and_then(|v| v.as_bool()).unwrap_or(false);
        assert!(healthy);
    }

    #[test]
    fn test_mcp_image_tools() {
        let mut server = Server::open();

        // 1. aios.image.list
        let res_list = server.call_tool("aios.image.list", &json!({}));
        assert_eq!(res_list.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_list.get("count").and_then(|v| v.as_u64()), Some(4));

        // Format filter
        let res_list_filter = server.call_tool("aios.image.list", &json!({ "format": "raw" }));
        assert_eq!(res_list_filter.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_list_filter.get("count").and_then(|v| v.as_u64()), Some(1));

        // 2. aios.image.get
        let res_get = server.call_tool("aios.image.get", &json!({ "id": "debian-12-minimal-raw" }));
        assert_eq!(res_get.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_get.pointer("/image/format").and_then(|v| v.as_str()), Some("raw"));

        // Negative: not found
        let res_get_missing = server.call_tool("aios.image.get", &json!({ "id": "nonexistent" }));
        assert_eq!(res_get_missing.get("ok").and_then(|v| v.as_bool()), Some(false));

        // Negative: missing id
        let res_get_no_id = server.call_tool("aios.image.get", &json!({}));
        assert_eq!(res_get_no_id.get("ok").and_then(|v| v.as_bool()), Some(false));

        // 3. aios.image.plan
        let res_plan = server.call_tool("aios.image.plan", &json!({ "id": "debian-12-minimal-raw" }));
        assert_eq!(res_plan.get("ok").and_then(|v| v.as_bool()), Some(true));
        let stages_count = res_plan.pointer("/plan/stages").and_then(|v| v.as_array()).map(|a| a.len());
        assert_eq!(stages_count, Some(4));

        // Negative: plan missing image
        let res_plan_missing = server.call_tool("aios.image.plan", &json!({ "id": "nonexistent" }));
        assert_eq!(res_plan_missing.get("ok").and_then(|v| v.as_bool()), Some(false));

        // Negative: plan missing id field
        let res_plan_no_id = server.call_tool("aios.image.plan", &json!({}));
        assert_eq!(res_plan_no_id.get("ok").and_then(|v| v.as_bool()), Some(false));

        // Hardening negative tests
        let res_control_char = server.call_tool("aios.image.get", &json!({ "id": "bad\x07id" }));
        assert_eq!(res_control_char.get("ok").and_then(|v| v.as_bool()), Some(false));

        let res_long_id = server.call_tool("aios.image.plan", &json!({ "id": "a".repeat(129) }));
        assert_eq!(res_long_id.get("ok").and_then(|v| v.as_bool()), Some(false));

        let res_long_store = server.call_tool("aios.image.list", &json!({ "store_path": "a".repeat(4097) }));
        assert_eq!(res_long_store.get("ok").and_then(|v| v.as_bool()), Some(false));

        // 4. aios.image.config
        let res_cfg = server.call_tool("aios.image.config", &json!({}));
        assert_eq!(res_cfg.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_cfg.pointer("/config/default_target").and_then(|v| v.as_str()), Some("debian-12-minimal-raw"));

        // 5. aios.image.policy
        let res_policy_all = server.call_tool("aios.image.policy", &json!({}));
        assert_eq!(res_policy_all.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert!(res_policy_all.pointer("/count").and_then(|v| v.as_u64()).unwrap_or(0) >= 4);

        let res_policy_single = server.call_tool("aios.image.policy", &json!({ "id": "debian-12-minimal-raw" }));
        assert_eq!(res_policy_single.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(res_policy_single.pointer("/verdict/allowed").and_then(|v| v.as_bool()), Some(true));

        let res_policy_missing = server.call_tool("aios.image.policy", &json!({ "id": "nonexistent" }));
        assert_eq!(res_policy_missing.get("ok").and_then(|v| v.as_bool()), Some(false));

        let res_policy_bad_id = server.call_tool("aios.image.policy", &json!({ "id": "bad\x07id" }));
        assert_eq!(res_policy_bad_id.get("ok").and_then(|v| v.as_bool()), Some(false));
    }
}
