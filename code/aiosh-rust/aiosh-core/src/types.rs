//! Shared AIOS types.
//!
//! Mirrors the legacy `code/aiosh-cli/src/types.ts` + the Python
//! `AuditRow` dataclass. The `AuditRow` struct is the in-memory form of
//! one SQLite `audit_ring` row; the hash proto is derived from it.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// All-zero genesis hash (first row's prev_hash).
pub const GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

pub type ActorKind = String; // "user" | "agent" | "system"
pub type OutcomeKind = String; // "ok" | "refused" | "error"

/// C-1..C-4 caution flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CFlags {
    pub c1: bool,
    pub c2: bool,
    pub c3: bool,
    pub c4: bool,
}

impl CFlags {
    pub fn to_json(&self) -> Value {
        serde_json::json!({"c1": self.c1, "c2": self.c2, "c3": self.c3, "c4": self.c4})
    }
}

/// One audit-ring row.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditRow {
    pub id: i64,
    pub ts: String,
    pub actor: ActorKind,
    pub actor_id: String,
    pub tool: String,
    pub command: String,
    pub args: Value, // object
    pub target: Option<String>,
    pub outcome: OutcomeKind,
    pub outcome_detail: Option<String>,
    pub constitution_rev: Option<String>,
    pub grant_token: Option<String>,
    pub c_flags: CFlags,
    // Sprint 2 classifier fields — only present when the row carried them.
    pub policy_revision: Option<String>,
    pub classify_rule_ids: Option<Vec<String>>,
    pub classify_evidence: Option<Value>, // {"c1": [...], ...}
    pub classify_overall_verdict: Option<String>,
    pub classify_verdict_reason: Option<String>,
    pub prev_hash: String,
    pub hash: String,
}

impl AuditRow {
    /// The hash proto (everything except `id` and `hash`), with
    /// classifier fields conditionally included — exactly the shape
    /// that was hashed at write time.
    pub fn hash_proto(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("ts".into(), Value::String(self.ts.clone()));
        map.insert("actor".into(), Value::String(self.actor.clone()));
        map.insert("actor_id".into(), Value::String(self.actor_id.clone()));
        map.insert("tool".into(), Value::String(self.tool.clone()));
        map.insert("command".into(), Value::String(self.command.clone()));
        map.insert("args".into(), self.args.clone());
        map.insert(
            "target".into(),
            match &self.target {
                Some(t) => Value::String(t.clone()),
                None => Value::Null,
            },
        );
        map.insert("outcome".into(), Value::String(self.outcome.clone()));
        map.insert(
            "outcome_detail".into(),
            match &self.outcome_detail {
                Some(d) => Value::String(d.clone()),
                None => Value::Null,
            },
        );
        map.insert(
            "constitution_rev".into(),
            match &self.constitution_rev {
                Some(r) => Value::String(r.clone()),
                None => Value::Null,
            },
        );
        map.insert(
            "grant_token".into(),
            match &self.grant_token {
                Some(t) => Value::String(t.clone()),
                None => Value::Null,
            },
        );
        map.insert("c_flags".into(), self.c_flags.to_json());
        map.insert("prev_hash".into(), Value::String(self.prev_hash.clone()));
        if let Some(p) = &self.policy_revision {
            map.insert("policy_revision".into(), Value::String(p.clone()));
        }
        if let Some(ids) = &self.classify_rule_ids {
            map.insert(
                "classify_rule_ids".into(),
                Value::Array(ids.iter().map(|s| Value::String(s.clone())).collect()),
            );
        }
        if let Some(ev) = &self.classify_evidence {
            map.insert("classify_evidence".into(), ev.clone());
        }
        if let Some(v) = &self.classify_overall_verdict {
            map.insert("classify_overall_verdict".into(), Value::String(v.clone()));
        }
        if let Some(r) = &self.classify_verdict_reason {
            map.insert("classify_verdict_reason".into(), Value::String(r.clone()));
        }
        Value::Object(map)
    }

    /// The full row dict (including id + hash) — used for archive
    /// segment lines.
    pub fn to_dict(&self) -> Value {
        let mut map = match self.hash_proto() {
            Value::Object(m) => m,
            _ => unreachable!(),
        };
        map.insert("id".into(), Value::from(self.id));
        map.insert("hash".into(), Value::String(self.hash.clone()));
        Value::Object(map)
    }

    pub fn recompute_hash(&self) -> String {
        let proto = self.hash_proto();
        crate::canonical::sha256_hex(&format!(
            "{}{}",
            self.prev_hash,
            crate::canonical::canonical(&proto)
        ))
    }
}

/// Grant scope (PEP).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrantScope {
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub networks: Vec<String>,
    #[serde(default)]
    pub paths: PathScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_irreversible: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathScope {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_row() -> AuditRow {
        AuditRow {
            id: 1,
            ts: "2026-08-21T06:59:00.000000Z".into(),
            actor: "user".into(),
            actor_id: "user:test@host".into(),
            tool: "system.status".into(),
            command: "aiosh status".into(),
            args: serde_json::json!({}),
            target: None,
            outcome: "ok".into(),
            outcome_detail: None,
            constitution_rev: Some("v0.0".into()),
            grant_token: None,
            c_flags: CFlags { c4: true, ..Default::default() },
            policy_revision: None,
            classify_rule_ids: None,
            classify_evidence: None,
            classify_overall_verdict: None,
            classify_verdict_reason: None,
            prev_hash: GENESIS_HASH.into(),
            hash: String::new(),
        }
    }

    #[test]
    fn hash_proto_is_canonical_json_compatible() {
        let row = sample_row();
        let proto = row.hash_proto();
        // Keys sorted by canonical() — just ensure it round-trips.
        let s = crate::canonical::canonical(&proto);
        assert!(s.starts_with('{'));
        assert!(s.contains("\"ts\":\"2026-08-21T06:59:00.000000Z\""));
        // Exact byte-for-byte expected form (compact, sorted keys,
        // no structural whitespace — matches TS canonicalJson).
        assert_eq!(
            s,
            "{\"actor\":\"user\",\"actor_id\":\"user:test@host\",\"args\":{},\"c_flags\":{\"c1\":false,\"c2\":false,\"c3\":false,\"c4\":true},\"command\":\"aiosh status\",\"constitution_rev\":\"v0.0\",\"grant_token\":null,\"outcome\":\"ok\",\"outcome_detail\":null,\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"target\":null,\"tool\":\"system.status\",\"ts\":\"2026-08-21T06:59:00.000000Z\"}"
        );
    }

    #[test]
    fn recompute_hash_matches_manual() {
        let mut row = sample_row();
        let proto = row.hash_proto();
        let expected = crate::canonical::sha256_hex(&format!(
            "{}{}",
            row.prev_hash,
            crate::canonical::canonical(&proto)
        ));
        row.hash = expected.clone();
        assert_eq!(row.recompute_hash(), expected);
    }
}
