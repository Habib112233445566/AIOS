//! Task Ledger Control — core service layer (implemented T-00024).
//!
//! Contract: `docs/tasks/evidence/T-00022-spec.md`.
//!
//! Separation of duties:
//!   - THIS module: input validation + direct dispatch into the
//!     existing ledger data model (`crate::ledger`) + shared access to
//!     the repaired tasks-directory resolver.
//!   - The MCP surface (`aiosh-mcp`): gate ordering — classify → PEP
//!     → audit via `crate::dispatch`, then calls [`TaskCall::execute`].
//!
//! Nothing here duplicates ledger logic; it wraps it. All APIs are
//! AIOS-specific (spec header), built on upstream-conformant tool
//! semantics only at the server boundary.

use crate::ledger::{self, LedgerPaths};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// The seven actions exposed by the grouped `aios.task` MCP tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskAction {
    Status,
    Check,
    Validate,
    Done,
    Block,
    Unblock,
    Skip,
    Rebuild,
    Metrics,
}

impl TaskAction {
    /// Parse the wire `action` string; `None` for anything outside the enum.
    pub fn parse(s: &str) -> Option<TaskAction> {
        match s {
            "status" => Some(TaskAction::Status),
            "check" => Some(TaskAction::Check),
            "validate" => Some(TaskAction::Validate),
            "done" => Some(TaskAction::Done),
            "block" => Some(TaskAction::Block),
            "unblock" => Some(TaskAction::Unblock),
            "skip" => Some(TaskAction::Skip),
            "rebuild" => Some(TaskAction::Rebuild),
            "metrics" => Some(TaskAction::Metrics),
            _ => None,
        }
    }

    /// Canonical wire spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskAction::Status => "status",
            TaskAction::Check => "check",
            TaskAction::Validate => "validate",
            TaskAction::Done => "done",
            TaskAction::Block => "block",
            TaskAction::Unblock => "unblock",
            TaskAction::Skip => "skip",
            TaskAction::Rebuild => "rebuild",
            TaskAction::Metrics => "metrics",
        }
    }

    /// True for `done|block|unblock|skip|rebuild` — the consequential
    /// set that requires a PEP grant covering `aios.task` (spec §3.1).
    pub fn requires_grant(&self) -> bool {
        matches!(
            self,
            TaskAction::Done | TaskAction::Block | TaskAction::Unblock | TaskAction::Skip | TaskAction::Rebuild
        )
    }

        fn needs_task_id(&self) -> bool {
        !matches!(
            self,
            TaskAction::Status | TaskAction::Check | TaskAction::Validate
            | TaskAction::Rebuild | TaskAction::Metrics
        )
    }
}

/// DEFAULT length caps mirroring the spec's inputSchema bounds.
/// Runtime values come from LedgerConfig (AIOSH_LEDGER_MAX_TEXT /
/// AIOSH_LEDGER_MAX_EVIDENCE_ITEMS) so operators can tighten/loosen
/// without a rebuild. Note: the PUBLISHED MCP inputSchema stays at the
/// defaults — env can only effectively tighten wire clients' bounds.
pub const MAX_TEXT_LEN: usize = 4096;
pub const MAX_EVIDENCE_ITEMS: usize = 16;

fn config() -> Result<crate::ledger_config::LedgerConfig, String> {
    crate::ledger_config::LedgerConfig::from_env()
}

fn bounded_text<'a>(
    field: &'a str,
    raw: Option<&'a str>,
    cap: usize,
) -> Result<Option<&'a str>, String> {
    match raw {
        None => Ok(None),
        Some(t) if t.is_empty() => Err(format!("'{}' must be non-empty when present", field)),
        Some(t) if t.len() > cap => Err(format!("'{}' exceeds {} bytes", field, cap)),
        Some(t) => Ok(Some(t)),
    }
}

/// A validated-on-entry task-service call (spec §3).
#[derive(Debug, Clone)]
pub struct TaskCall<'a> {
    pub action: TaskAction,
    pub task_id: Option<u64>,
    pub note: Option<&'a str>,
    pub reason: Option<&'a str>,
    pub evidence: &'a [String],
}

impl<'a> TaskCall<'a> {
    /// T-00084: consolidated observability snapshot (spec T-00082 §2).
    /// Pure composer — callers supply audit-ring facts from their own
    /// ring handle (CLI ctx / MCP server), keeping this testable.
    /// Stable ADDITIVE-ONLY key set (T-00082 promise): tasks{…},
    /// audit{…}, config{…}.
    #[allow(clippy::too_many_arguments)]
    pub fn build_metrics(
        tasks: Value,
        audit_rows: usize,
        audit_verify_ok: bool,
        head_hash_prefix: &str,
        cfg: &crate::ledger_config::LedgerConfig,
    ) -> Result<Value, String> {
        let completed = tasks["completed"].as_array().map(|a| a.len()).unwrap_or(0);
        let blocked = tasks["blocked"].as_array().map(|a| a.len()).unwrap_or(0);
        let skipped = tasks["skipped"].as_array().map(|a| a.len()).unwrap_or(0);
        Ok(json!({
            "tasks": {
                "total_tasks": tasks["total_tasks"].clone(),
                "completed": completed,
                "blocked": blocked,
                "skipped": skipped,
                "next_task": tasks["next_task"].clone(),
                "last_event_seq": tasks["last_event_seq"].clone(),
                "last_completed_at": tasks["last_completed_at"].clone(),
            },
            "audit": {
                "rows": audit_rows,
                "verify_ok": audit_verify_ok,
                "head_hash_prefix": head_hash_prefix,
            },
            "config": {
                "lock_timeout_secs": cfg.lock_timeout_secs,
                "max_ledger_bytes": cfg.max_ledger_bytes,
                "max_events_bytes": cfg.max_events_bytes,
                "max_state_bytes": cfg.max_state_bytes,
                "max_text": cfg.max_text,
                "max_evidence_items": cfg.max_evidence_items,
            }
        }))
    }

    /// Spec §3.3 pre-validation: conditional requirements per action.
    /// Runs INSIDE the gate so refusals still earn an honest audit row.
    pub fn validate(&self) -> Result<(), String> {
        let cfg = config()?;
        if self.action.needs_task_id() && self.task_id.is_none() {
            return Err(format!(
                "action '{}' requires 'task_id'",
                self.action.as_str()
            ));
        }
        if !self.action.needs_task_id() && self.task_id.is_some() {
            return Err(format!(
                "action '{}' does not take 'task_id'",
                self.action.as_str()
            ));
        }
        match self.action {
            TaskAction::Done => {
                let note = bounded_text("note", self.note, cfg.max_text)?;
                if note.is_none() {
                    return Err("action 'done' requires a non-empty 'note'".into());
                }
            }
            TaskAction::Block | TaskAction::Unblock | TaskAction::Skip => {
                if bounded_text("reason", self.reason, cfg.max_text)?.is_none() {
                    return Err(format!(
                        "action '{}' requires a non-empty 'reason'",
                        self.action.as_str()
                    ));
                }
            }
            _ => {}
        }
        if self.evidence.len() > cfg.max_evidence_items {
            return Err(format!(
                "'evidence' exceeds {} items",
                cfg.max_evidence_items
            ));
        }
        for e in self.evidence {
            if e.len() > cfg.max_text {
                return Err(format!("'evidence' item exceeds {MAX_TEXT_LEN} bytes"));
            }
        }
        Ok(())
    }

    /// Public composition entry (CLI/MCP callers hold the ring).
    pub fn build_metrics_pub(
        tasks: Value,
        audit_rows: usize,
        audit_verify_ok: bool,
        head_hash_prefix: &str,
        cfg: &crate::ledger_config::LedgerConfig,
    ) -> Result<Value, String> {
        Self::build_metrics(tasks, audit_rows, audit_verify_ok, head_hash_prefix, cfg)
    }

    /// Execute against the ledger data model, resolving the tasks
    /// directory through the shared repaired resolver. Persistence
    /// effects are exactly those of the underlying `ledger::` calls
    /// (spec §3.1). The caller (MCP server) owns gate + audit wrapping.
    pub fn execute(&self) -> Result<Value, String> {
        let p = ledger::paths()?;
        self.execute_with(&p)
    }

    /// Test/diagnostic variant with an explicit directory bundle.
    pub fn execute_with(&self, p: &LedgerPaths) -> Result<Value, String> {
        match self.action {
            TaskAction::Status => ledger::load_state(&p.state, &p.events),
            TaskAction::Check => ledger::assert_ledger_invariants(&p.ledger),
            TaskAction::Validate => ledger::validate_state(p),
            TaskAction::Done => {
                let id = self.task_id.expect("validate ensures task_id");
                ledger::complete_task(p, id, self.note.unwrap_or(""), self.evidence)
            }
            TaskAction::Block => {
                let id = self.task_id.expect("validate ensures task_id");
                ledger::block_task(p, id, self.reason.expect("validate ensures reason"))
            }
            TaskAction::Unblock => {
                let id = self.task_id.expect("validate ensures task_id");
                ledger::unblock_task(p, id, self.reason.expect("validate ensures reason"))
            }
            TaskAction::Skip => {
                let id = self.task_id.expect("validate ensures task_id");
                ledger::skip_task(p, id, self.reason.expect("validate ensures reason"))
            }
            TaskAction::Rebuild => ledger::rebuild_state(p),
            TaskAction::Metrics => Err(
                "metrics requires an audit-ring context; use the CLI/MCP surface"
                    .into(),
            ),
        }
    }
}

/// Shared resolver entry point — delegates to the data-model resolver
/// (`ledger::tasks_dir`, spec §5) so CLI and service cannot drift:
/// 1. `$AIOSH_TASKS_DIR` when set and non-empty;
/// 2. else ancestor walk from `current_exe()` for
///    `docs/tasks/MASTER_TASK_LEDGER.jsonl`;
/// 3. else Err("cannot locate docs/tasks (set AIOSH_TASKS_DIR)").
pub fn tasks_dir() -> Result<PathBuf, String> {
    ledger::tasks_dir()
}

/// Walk helper re-exported for tests/diagnostics.
pub fn find_ancestor_tasks_dir(start: &Path) -> Option<PathBuf> {
    ledger::find_ancestor_tasks_dir(start)
}

/// Owned form of a parsed tool-call argument set (wire → service).
/// Strict typing per spec §8: unknown keys, non-conforming types, or a
/// non-enum `action` are schema violations — the server maps these to
/// JSON-RPC -32602, while semantic refusals go through the gate.
#[derive(Debug, Clone)]
pub struct TaskArgsOwned {
    pub action: TaskAction,
    pub task_id: Option<u64>,
    pub note: Option<String>,
    pub reason: Option<String>,
    pub evidence: Vec<String>,
    pub grant_id: Option<String>,
}

impl TaskArgsOwned {
    /// Borrow view for gate + execution.
    pub fn call(&self) -> TaskCall<'_> {
        TaskCall {
            action: self.action,
            task_id: self.task_id,
            note: self.note.as_deref(),
            reason: self.reason.as_deref(),
            evidence: &self.evidence,
        }
    }
}

const ALLOWED_ARGS: [&str; 6] = ["action", "task_id", "note", "reason", "evidence", "grant_id"];

/// Parse and type-check raw tool arguments (spec §3 inputSchema).
pub fn parse_args(v: &Value) -> Result<TaskArgsOwned, String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "'arguments' must be an object".to_string())?;
    for k in obj.keys() {
        if !ALLOWED_ARGS.contains(&k.as_str()) {
            return Err(format!("unexpected argument '{k}'"));
        }
    }
    let action_s = obj
        .get("action")
        .ok_or_else(|| "missing required argument 'action'".to_string())?
        .as_str()
        .ok_or_else(|| "'action' must be a string".to_string())?;
    let action =
        TaskAction::parse(action_s).ok_or_else(|| format!("unknown action '{action_s}'"))?;
    let opt_u64 = |key: &str| -> Result<Option<u64>, String> {
        match obj.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(x) => x
                .as_u64()
                .filter(|n| *n >= 1)
                .map(Some)
                .ok_or_else(|| format!("'{key}' must be a positive integer >= 1")),
        }
    };
    let opt_str = |key: &str| -> Result<Option<String>, String> {
        match obj.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(x) => {
                let s = x
                    .as_str()
                    .ok_or_else(|| format!("'{key}' must be a string"))?;
                // inputSchema minLength/maxLength (spec §3): schema
                // violations are protocol errors, enforced here.
                if s.is_empty() {
                    return Err(format!("'{key}' must be non-empty"));
                }
                if s.len() > MAX_TEXT_LEN {
                    return Err(format!("'{key}' exceeds {MAX_TEXT_LEN} bytes"));
                }
                Ok(Some(s.to_string()))
            }
        }
    };
    let evidence = match obj.get("evidence") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => {
            if items.len() > MAX_EVIDENCE_ITEMS {
                return Err(format!(
                    "'evidence' exceeds {MAX_EVIDENCE_ITEMS} items"
                ));
            }
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                let s = it
                    .as_str()
                    .ok_or_else(|| "'evidence' items must be strings".to_string())?;
                if s.len() > MAX_TEXT_LEN {
                    return Err(format!("'evidence' item exceeds {MAX_TEXT_LEN} bytes"));
                }
                out.push(s.to_string());
            }
            out
        }
        Some(_) => return Err("'evidence' must be an array of strings".into()),
    };
    Ok(TaskArgsOwned {
        action,
        task_id: opt_u64("task_id")?,
        note: opt_str("note")?,
        reason: opt_str("reason")?,
        evidence,
        grant_id: opt_str("grant_id")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tmp_ledger_paths() -> (tempfile::TempDir, LedgerPaths) {
        let tmp = tempfile::tempdir().unwrap();
        let p = LedgerPaths {
            dir: tmp.path().to_path_buf(),
            ledger: tmp.path().join("MASTER_TASK_LEDGER.jsonl"),
            state: tmp.path().join("TASK_STATE.json"),
            events: tmp.path().join("COMPLETIONS.jsonl"),
            lock: tmp.path().join(".TASK_STATE.lock"),
            evidence: tmp.path().join("evidence"),
        };
        // Minimal well-formed ledger (5 tasks).
        let mut lines = String::new();
        for i in 1..=5u64 {
            lines.push_str(&format!(
                "{{\"id\":{},\"title\":\"t{}\",\"depends_on\":{},\"next_task\":{},\"instructions\":[\"i\"],\"acceptance\":[\"a\"]}}\n",
                i,
                i,
                if i == 1 { "[]".to_string() } else { format!("[{}]", i - 1) },
                if i < 5 { (i + 1).to_string() } else { "null".to_string() },
            ));
        }
        std::fs::write(&p.ledger, lines).unwrap();
        let state = json!({
            "schema_version": 2,
            "ledger": "MASTER_TASK_LEDGER.jsonl",
            "total_tasks": 5,
            "next_task": 1,
            "completed": [],
            "blocked": [],
            "skipped": [],
            "last_completed_at": null,
            "last_event_seq": 0,
            "rule": ledger::RULE,
        });
        ledger::save_state_atomic(&state, &p.state).unwrap();
        (tmp, p)
    }

    #[test]
    fn parse_round_trip_and_unknowns() {
        for name in ["status", "check", "done", "block", "unblock", "skip", "rebuild"] {
            let a = TaskAction::parse(name).unwrap();
            assert_eq!(a.as_str(), name);
        }
        assert_eq!(TaskAction::parse(""), None);
        assert_eq!(TaskAction::parse("DONE"), None);
        assert_eq!(TaskAction::parse("complete"), None);
    }

    #[test]
    fn grant_truth_table_matches_spec_d1() {
        assert!(!TaskAction::Status.requires_grant());
        assert!(!TaskAction::Check.requires_grant());
        for a in [TaskAction::Done, TaskAction::Block, TaskAction::Unblock, TaskAction::Skip, TaskAction::Rebuild] {
            assert!(a.requires_grant(), "{} must require grant", a.as_str());
        }
    }

    #[test]
    fn validate_conditional_requirements() {
        let ok_status = TaskCall { action: TaskAction::Status, task_id: None, note: None, reason: None, evidence: &[] };
        assert!(ok_status.validate().is_ok());

        let done_no_id = TaskCall { action: TaskAction::Done, task_id: None, note: Some("x"), reason: None, evidence: &[] };
        assert!(done_no_id.validate().unwrap_err().contains("task_id"));

        let done_no_note = TaskCall { action: TaskAction::Done, task_id: Some(2), note: None, reason: None, evidence: &[] };
        assert!(done_no_note.validate().unwrap_err().contains("'note'"));

        let block_no_reason = TaskCall { action: TaskAction::Block, task_id: Some(2), note: None, reason: None, evidence: &[] };
        assert!(block_no_reason.validate().unwrap_err().contains("'reason'"));

        let skip_ok = TaskCall { action: TaskAction::Skip, task_id: Some(2), note: None, reason: Some("because"), evidence: &[] };
        assert!(skip_ok.validate().is_ok());

        let status_with_id = TaskCall { action: TaskAction::Status, task_id: Some(1), note: None, reason: None, evidence: &[] };
        assert!(status_with_id.validate().unwrap_err().contains("does not take"));
    }

    #[test]
    fn parse_args_schema_bounds() {
        // minLength / maxLength / maxItems / minimum are schema
        // constraints -> protocol-level rejections (spec §3.3, §8).
        assert!(parse_args(&json!({"action":"done","task_id":1,"note":""})).is_err());
        let long = "x".repeat(MAX_TEXT_LEN + 1);
        assert!(parse_args(&json!({"action":"done","task_id":1,"note":long})).is_err());
        let ok_len = "x".repeat(MAX_TEXT_LEN);
        assert!(parse_args(&json!({"action":"done","task_id":1,"note":ok_len})).is_ok());
        let ev: Vec<String> = (0..=MAX_EVIDENCE_ITEMS).map(|_| "e".to_string()).collect();
        assert!(parse_args(&json!({"action":"done","task_id":1,"note":"x","evidence":ev})).is_err());
    }

    #[test]
    fn execute_status_and_done_against_explicit_paths() {
        let (_t, p) = tmp_ledger_paths();
        let st = TaskCall { action: TaskAction::Status, task_id: None, note: None, reason: None, evidence: &[] }
            .execute_with(&p)
            .unwrap();
        assert_eq!(st["next_task"], json!(1));

        let done = TaskCall { action: TaskAction::Done, task_id: Some(1), note: Some("service impl"), reason: None, evidence: &["docs/tasks/evidence/x.md".to_string()] };
        assert!(done.validate().is_ok());
        let r = done.execute_with(&p).unwrap();
        assert_eq!(r["completed"], json!(1));
        assert_eq!(r["next_task"], json!(2));
        let after = TaskCall { action: TaskAction::Status, task_id: None, note: None, reason: None, evidence: &[] }
            .execute_with(&p)
            .unwrap();
        assert_eq!(after["completed"], json!([1]));
    }

    #[test]
    fn execute_rebuild_replays_skip_pointer() {
        let (_t, p) = tmp_ledger_paths();
        TaskCall { action: TaskAction::Done, task_id: Some(1), note: Some("n"), reason: None, evidence: &[] }
            .execute_with(&p)
            .unwrap();
        TaskCall { action: TaskAction::Skip, task_id: Some(2), note: None, reason: Some("scope"), evidence: &[] }
            .execute_with(&p)
            .unwrap();
        std::fs::write(&p.state, "{}").unwrap();
        let st = TaskCall { action: TaskAction::Rebuild, task_id: None, note: None, reason: None, evidence: &[] }
            .execute_with(&p)
            .unwrap();
        assert_eq!(st["next_task"], json!(3));
        assert_eq!(st["skipped"], json!([2]));
    }

    #[test]
    fn parse_args_strict_types() {
        let good = json!({"action":"done","task_id":3,"note":"n","evidence":["a"],"grant_id":"g1"});
        let a = parse_args(&good).unwrap();
        assert_eq!(a.action, TaskAction::Done);
        assert_eq!(a.task_id, Some(3));
        assert_eq!(a.grant_id.as_deref(), Some("g1"));
        let call = a.call();
        assert!(call.validate().is_ok());

        assert!(parse_args(&json!({"action":42})).is_err());
        assert!(parse_args(&json!({})).is_err());
        assert!(parse_args(&json!({"action":"frobnicate"})).is_err());
        assert!(parse_args(&json!({"action":"status","extra":1})).is_err());
        assert!(parse_args(&json!({"action":"done","task_id":-2,"note":"x"})).is_err());
        assert!(parse_args(&json!({"action":"done","task_id":0,"note":"x"})).is_err());
        assert!(parse_args(&json!({"action":"done","task_id":1,"note":"x","evidence":[5]})).is_err());
    }

    #[test]
    fn resolver_helper_finds_and_refuses() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("docs/tasks")).unwrap();
        std::fs::write(tmp.path().join("docs/tasks/MASTER_TASK_LEDGER.jsonl"), "{}\n").unwrap();
        let deep = tmp.path().join("a/b/c");
        std::fs::create_dir_all(&deep).unwrap();
        assert_eq!(
            find_ancestor_tasks_dir(&deep),
            Some(tmp.path().join("docs/tasks"))
        );
        let empty = tempfile::tempdir().unwrap();
        assert_eq!(find_ancestor_tasks_dir(empty.path()), None);
    }
}
