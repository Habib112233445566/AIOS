//! Sprint 1/2 — MCP dispatch helper (classifier → PEP → audit gate).
//!
//! Port of `_dispatch.py` / `server.py:_recorded_call`. Every tool call
//! routed from the AI to a host action must:
//!   1. be classified (C-1..C-4),
//!   2. be authorized by a valid PEP grant (or refused with reason),
//!   3. emit exactly one audit row extending the chain,
//!   4. return the row id alongside the result.
//!
//! Gate ordering (ADR-0035 §D-4):
//!   1. classifier verdict — a "refused" verdict refuses regardless of
//!      grant presence (the safety boundary),
//!   2. PEP grant check — the grant must authorize (tool, target, args).

use crate::audit::{active_constitution_rev, AuditRing, AuditRowInput};
use crate::classifier::{classify, ClassificationResult};
use crate::pep::PepStore;
use crate::types::CFlags;

#[derive(Debug, Clone)]
pub struct DispatchResult {
    pub ok: bool,
    pub tool: String,
    pub audit_id: Option<i64>,
    pub reason: Option<String>,
    pub gate: Option<String>, // "classifier" | "pep"
    pub policy_revision: String,
    pub classify_rule_ids: Vec<String>,
    pub classify_evidence: serde_json::Value,
    pub classify_overall_verdict: String,
    pub classify_verdict_reason: String,
    pub c_flags: CFlags,
}

impl DispatchResult {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "ok": self.ok,
            "tool": self.tool,
            "audit_id": self.audit_id,
            "reason": self.reason,
            "gate": self.gate,
            "policy_revision": self.policy_revision,
        })
    }
}

/// Run the gate and (on refusal) write the refusal row. Returns the
/// verdict; on gate-pass the caller writes the outcome row via
/// `commit()`.
pub fn dispatch(
    ring: &mut AuditRing,
    pep: &PepStore,
    tool: &str,
    command: &str,
    args: &serde_json::Value,
    target: Option<&str>,
    grant_id: Option<&str>,
    require_grant: bool,
    actor_id: &str,
    actor: &str,
) -> DispatchResult {
    let cls = classify(tool, target, args);
    let c_flags = CFlags {
        c1: cls.c1.flag,
        c2: cls.c2.flag,
        c3: cls.c3.flag,
        c4: cls.c4.flag,
    };

    // Gate #1 — classifier.
    if cls.overall_verdict == "refused" {
        let reason = format!(
            "classifier refused (policy={}, verdict={})",
            cls.policy_revision,
            if cls.verdict_reason.is_empty() {
                "refused"
            } else {
                cls.verdict_reason.as_str()
            }
        );
        let row = ring.write(AuditRowInput {
            ts: crate::canonical::utcnow_iso(),
            actor: actor.into(),
            actor_id: actor_id.into(),
            tool: tool.into(),
            command: command.into(),
            args: args.clone(),
            target: target.map(|s| s.into()),
            outcome: "refused".into(),
            outcome_detail: Some(reason.clone()),
            constitution_rev: Some(active_constitution_rev(None)),
            grant_token: grant_id.map(|s| s.into()),
            c_flags: c_flags.clone(),
            policy_revision: Some(cls.policy_revision.clone()),
            classify_rule_ids: Some(cls.rule_ids.clone()),
            classify_evidence: Some(cls.evidence_per_flag()),
            classify_overall_verdict: Some(cls.overall_verdict.clone()),
            classify_verdict_reason: Some(cls.verdict_reason.clone()),
        });
        return DispatchResult {
            ok: false,
            tool: tool.into(),
            audit_id: row.ok().map(|r| r.id),
            reason: Some(reason),
            gate: Some("classifier".into()),
            policy_revision: cls.policy_revision.clone(),
            classify_rule_ids: cls.rule_ids.clone(),
            classify_evidence: cls.evidence_per_flag(),
            classify_overall_verdict: cls.overall_verdict.clone(),
            classify_verdict_reason: cls.verdict_reason.clone(),
            c_flags,
        };
    }

    // Gate #2 — PEP grant.
    let mut verdict = pep.check(grant_id, tool, target);
    if require_grant && grant_id.is_none() {
        verdict = Err(format!("tool '{}' requires explicit PEP grant", tool));
    }
    if let Err(reason) = verdict {
        let row = ring.write(AuditRowInput {
            ts: crate::canonical::utcnow_iso(),
            actor: actor.into(),
            actor_id: actor_id.into(),
            tool: tool.into(),
            command: command.into(),
            args: args.clone(),
            target: target.map(|s| s.into()),
            outcome: "refused".into(),
            outcome_detail: Some(reason.clone()),
            constitution_rev: Some(active_constitution_rev(None)),
            grant_token: grant_id.map(|s| s.into()),
            c_flags: c_flags.clone(),
            policy_revision: Some(cls.policy_revision.clone()),
            classify_rule_ids: Some(cls.rule_ids.clone()),
            classify_evidence: Some(cls.evidence_per_flag()),
            classify_overall_verdict: Some(cls.overall_verdict.clone()),
            classify_verdict_reason: Some(cls.verdict_reason.clone()),
        });
        return DispatchResult {
            ok: false,
            tool: tool.into(),
            audit_id: row.ok().map(|r| r.id),
            reason: Some(reason),
            gate: Some("pep".into()),
            policy_revision: cls.policy_revision.clone(),
            classify_rule_ids: cls.rule_ids.clone(),
            classify_evidence: cls.evidence_per_flag(),
            classify_overall_verdict: cls.overall_verdict.clone(),
            classify_verdict_reason: cls.verdict_reason.clone(),
            c_flags,
        };
    }

    // Gate passed.
    DispatchResult {
        ok: true,
        tool: tool.into(),
        audit_id: None,
        reason: None,
        gate: None,
        policy_revision: cls.policy_revision.clone(),
        classify_rule_ids: cls.rule_ids.clone(),
        classify_evidence: cls.evidence_per_flag(),
        classify_overall_verdict: cls.overall_verdict.clone(),
        classify_verdict_reason: cls.verdict_reason.clone(),
        c_flags,
    }
}

/// Append the actual outcome row after a gate-passed call, carrying the
/// classifier provenance from `dispatch()`.
#[allow(clippy::too_many_arguments)]
pub fn commit(
    ring: &mut AuditRing,
    tool: &str,
    command: &str,
    args: &serde_json::Value,
    target: Option<&str>,
    grant_id: Option<&str>,
    outcome: &str,
    outcome_detail: Option<&str>,
    actor_id: &str,
    actor: &str,
    verdict: &DispatchResult,
) -> crate::types::AuditRow {
    ring.write(AuditRowInput {
        ts: crate::canonical::utcnow_iso(),
        actor: actor.into(),
        actor_id: actor_id.into(),
        tool: tool.into(),
        command: command.into(),
        args: args.clone(),
        target: target.map(|s| s.into()),
        outcome: outcome.into(),
        outcome_detail: outcome_detail.map(|s| s.into()),
        constitution_rev: Some(active_constitution_rev(None)),
        grant_token: grant_id.map(|s| s.into()),
        c_flags: verdict.c_flags.clone(),
        policy_revision: Some(verdict.policy_revision.clone()),
        classify_rule_ids: Some(verdict.classify_rule_ids.clone()),
        classify_evidence: Some(verdict.classify_evidence.clone()),
        classify_overall_verdict: Some(verdict.classify_overall_verdict.clone()),
        classify_verdict_reason: Some(verdict.classify_verdict_reason.clone()),
    })
    .expect("audit row write failed")
}

/// Run a non-pentest MCP function behind the authoritative gate and
/// append exactly one result row after it returns (mirrors
/// `server.py:_recorded_call`).
pub fn recorded_call<F>(
    ring: &mut AuditRing,
    pep: &PepStore,
    tool: &str,
    command: &str,
    args: &serde_json::Value,
    target: Option<&str>,
    grant_id: Option<&str>,
    require_grant: bool,
    actor_id: &str,
    actor: &str,
    mut f: F,
) -> serde_json::Value
where
    F: FnMut() -> Result<serde_json::Value, String>,
{
    let verdict = dispatch(ring, pep, tool, command, args, target, grant_id, require_grant, actor_id, actor);
    if !verdict.ok {
        return verdict.to_json();
    }
    match f() {
        Ok(mut raw) => {
            if !raw.is_object() {
                raw = serde_json::json!({"ok": true, "result": raw});
            }
            let outcome = if raw.get("ok").and_then(|v| v.as_bool()).unwrap_or(true) {
                "ok"
            } else {
                "error"
            };
            let detail = if outcome == "ok" {
                None
            } else {
                raw.get("error").and_then(|v| v.as_str()).map(|s| s.to_string())
            };
            let row = commit(
                ring, tool, command, args, target, grant_id, outcome, detail.as_deref(),
                actor_id, actor, &verdict,
            );
            raw["audit_id"] = serde_json::json!(row.id);
            raw["classifier_policy_revision"] = serde_json::json!(verdict.policy_revision);
            raw
        }
        Err(detail) => {
            let row = commit(
                ring, tool, command, args, target, grant_id, "error", Some(&detail),
                actor_id, actor, &verdict,
            );
            serde_json::json!({
                "ok": false,
                "tool": tool,
                "error": detail,
                "audit_id": row.id,
            })
        }
    }
}

/// Helper that produces a fresh classification dict for commit paths
/// that lost the verdict (mirrors `_classify_dict`).
pub fn fresh_classification(
    tool: &str,
    target: Option<&str>,
    args: &serde_json::Value,
) -> ClassificationResult {
    classify(tool, target, args)
}

pub const DEFAULT_ACTOR_ID: &str = "agent:mcp@aiosh-mcp";
pub const DEFAULT_ACTOR: &str = "agent";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditRing;
    use crate::pep::PepStore;
    use rusqlite::Connection;
    use serde_json::json;

    fn setup() -> (AuditRing, PepStore) {
        let conn = Connection::open_in_memory().unwrap();
        let ring = AuditRing::from_conn(conn, String::new());
        ring.ensure_schema().unwrap();
        let pep = PepStore::new(Connection::open_in_memory().unwrap()).unwrap();
        (ring, pep)
    }

    #[test]
    fn classifier_refusal_beats_grant() {
        let (mut ring, pep) = setup();
        // Create a valid grant for pentest.nmap.
        let mut scope = crate::types::GrantScope::default();
        scope.tools = vec!["pentest.*".into()];
        scope.networks = vec!["10.0.0.0/8".into()];
        let g = pep
            .create(&scope, 3600, "agent:test", "abc123")
            .unwrap();
        // But target is an external aggregator → classifier refuses first.
        let d = dispatch(
            &mut ring,
            &pep,
            "pentest.nmap",
            "nmap shodan.io",
            &json!({"target": "shodan.io"}),
            Some("shodan.io"),
            Some(&g.grant_id),
            false,
            DEFAULT_ACTOR_ID,
            DEFAULT_ACTOR,
        );
        assert!(!d.ok);
        assert_eq!(d.gate.as_deref(), Some("classifier"));
        // Refusal row was written and extends the chain.
        assert!(d.audit_id.is_some());
        assert!(ring.verify().unwrap().ok);
    }

    #[test]
    fn pep_refuses_without_grant() {
        let (mut ring, pep) = setup();
        let d = dispatch(
            &mut ring,
            &pep,
            "pentest.nmap",
            "nmap 10.0.0.5",
            &json!({"target": "10.0.0.5"}),
            Some("10.0.0.5"),
            None,
            false,
            DEFAULT_ACTOR_ID,
            DEFAULT_ACTOR,
        );
        assert!(!d.ok);
        assert_eq!(d.gate.as_deref(), Some("pep"));
        assert!(d.reason.as_deref().unwrap().contains("requires explicit PEP grant"));
    }

    #[test]
    fn gate_passes_with_valid_grant_and_commit_extends_chain() {
        let (mut ring, pep) = setup();
        let mut scope = crate::types::GrantScope::default();
        scope.tools = vec!["pentest.*".into()];
        scope.networks = vec!["10.0.0.0/8".into()];
        let g = pep
            .create(&scope, 3600, "agent:test", "abc123")
            .unwrap();
        let d = dispatch(
            &mut ring,
            &pep,
            "pentest.nmap",
            "nmap 10.0.0.5",
            &json!({"target": "10.0.0.5"}),
            Some("10.0.0.5"),
            Some(&g.grant_id),
            false,
            DEFAULT_ACTOR_ID,
            DEFAULT_ACTOR,
        );
        assert!(d.ok);
        let row = commit(
            &mut ring, "pentest.nmap", "nmap 10.0.0.5", &json!({"target": "10.0.0.5"}),
            Some("10.0.0.5"), Some(&g.grant_id), "ok", None,
            DEFAULT_ACTOR_ID, DEFAULT_ACTOR, &d,
        );
        assert!(row.id > 0);
        // Refusal from earlier classifier test + this row → chain intact.
        assert!(ring.verify().unwrap().ok);
    }
}
