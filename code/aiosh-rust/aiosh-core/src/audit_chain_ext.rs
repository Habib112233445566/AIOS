//! Audit Chain Extensions Data Model (T-02301..T-02310).
//!
//! Extends the core audit ring with structured provenance metadata, causal DAG ancestry,
//! asymmetric digital signatures, and extensible payload namespaces, while preserving
//! strict SHA-256 hash chaining and canonical JSON backward compatibility.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::canonical::{canonical, sha256_hex};
use crate::types::{AuditRow, CFlags};

pub const MAX_SESSION_ID_LEN: usize = 128;
pub const MAX_TRACE_ID_LEN: usize = 128;
pub const MAX_CAUSAL_LINKS: usize = 16;
pub const MAX_EXTENSION_ENTRIES: usize = 32;
pub const MAX_EXTENSION_PAYLOAD_BYTES: usize = 65536;

pub const AUDIT_EXT_ERR_VALIDATION: &str = "AUDIT_EXT_ERR_VALIDATION";
pub const AUDIT_EXT_ERR_HASH: &str = "AUDIT_EXT_ERR_HASH";
pub const AUDIT_EXT_ERR_SIGNATURE: &str = "AUDIT_EXT_ERR_SIGNATURE";
pub const AUDIT_EXT_ERR_BOUNDS: &str = "AUDIT_EXT_ERR_BOUNDS";

/// Asymmetric cryptographic signature block over the row hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSignature {
    /// Signing algorithm name, e.g. "ed25519", "secp256k1"
    pub algorithm: String,
    /// Hex-encoded public verification key
    pub public_key: String,
    /// Hex-encoded digital signature over the row's SHA-256 hash
    pub signature: String,
}

impl AuditSignature {
    pub fn new(algorithm: impl Into<String>, public_key: impl Into<String>, signature: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            public_key: public_key.into(),
            signature: signature.into(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.algorithm.trim().is_empty() {
            return Err(format!("{}: algorithm cannot be empty", AUDIT_EXT_ERR_SIGNATURE));
        }
        if self.public_key.trim().is_empty() || self.public_key.len() > 256 {
            return Err(format!("{}: invalid public_key length", AUDIT_EXT_ERR_SIGNATURE));
        }
        if self.signature.trim().is_empty() || self.signature.len() > 512 {
            return Err(format!("{}: invalid signature length", AUDIT_EXT_ERR_SIGNATURE));
        }
        Ok(())
    }
}

/// Causal provenance link in the execution DAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditCausalLink {
    /// SHA-256 hash of the ancestor causal audit row
    pub parent_event_hash: String,
    /// Causality relationship: "delegation", "subtask", "trigger", "retry"
    pub causality_type: String,
}

impl AuditCausalLink {
    pub fn new(parent_event_hash: impl Into<String>, causality_type: impl Into<String>) -> Self {
        Self {
            parent_event_hash: parent_event_hash.into(),
            causality_type: causality_type.into(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.parent_event_hash.len() != 64 || !self.parent_event_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("{}: parent_event_hash must be a 64-char hex string", AUDIT_EXT_ERR_VALIDATION));
        }
        if self.causality_type.trim().is_empty() || self.causality_type.len() > 64 {
            return Err(format!("{}: causality_type must be 1..64 characters", AUDIT_EXT_ERR_VALIDATION));
        }
        Ok(())
    }
}

/// Session, trace, and authorization provenance metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuditProvenance {
    /// Interactive session or workflow UUID
    pub session_id: Option<String>,
    /// Bound PEP authorization grant identifier
    pub pep_grant_id: Option<String>,
    /// Delegation depth at execution time
    pub delegation_depth: u32,
    /// Distributed trace ID (e.g. W3C / OpenTelemetry)
    pub trace_id: Option<String>,
    /// Distributed span ID
    pub span_id: Option<String>,
}

impl AuditProvenance {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref sid) = self.session_id {
            if sid.trim().is_empty() || sid.len() > MAX_SESSION_ID_LEN || sid.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(format!("{}: invalid session_id format or length", AUDIT_EXT_ERR_VALIDATION));
            }
        }
        if let Some(ref gid) = self.pep_grant_id {
            if gid.trim().is_empty() || gid.len() > 128 || gid.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(format!("{}: invalid pep_grant_id format or length", AUDIT_EXT_ERR_VALIDATION));
            }
        }
        if let Some(ref tid) = self.trace_id {
            if tid.trim().is_empty() || tid.len() > MAX_TRACE_ID_LEN {
                return Err(format!("{}: invalid trace_id length", AUDIT_EXT_ERR_VALIDATION));
            }
        }
        if let Some(ref spid) = self.span_id {
            if spid.trim().is_empty() || spid.len() > MAX_TRACE_ID_LEN {
                return Err(format!("{}: invalid span_id length", AUDIT_EXT_ERR_VALIDATION));
            }
        }
        Ok(())
    }
}

/// Extended audit-ring row incorporating provenance, DAG causality, and signatures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtendedAuditRow {
    pub id: i64,
    pub ts: String,
    pub actor: String,
    pub actor_id: String,
    pub tool: String,
    pub command: String,
    pub args: Value,
    pub target: Option<String>,
    pub outcome: String,
    pub outcome_detail: Option<String>,
    pub constitution_rev: Option<String>,
    pub grant_token: Option<String>,
    pub c_flags: CFlags,
    // Sprint-2 classifier fields
    pub policy_revision: Option<String>,
    pub classify_rule_ids: Option<Vec<String>>,
    pub classify_evidence: Option<Value>,
    pub classify_overall_verdict: Option<String>,
    pub classify_verdict_reason: Option<String>,
    // Extensions
    pub provenance: Option<AuditProvenance>,
    pub causal_links: Vec<AuditCausalLink>,
    pub signature: Option<AuditSignature>,
    pub extensions: HashMap<String, Value>,
    // Hash chain
    pub prev_hash: String,
    pub hash: String,
}

impl ExtendedAuditRow {
    /// Builds the canonical hash proto dictionary for hash computation and verification.
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

        // Classifier fields (conditional inclusion)
        if let Some(ref pr) = self.policy_revision {
            map.insert("policy_revision".into(), Value::String(pr.clone()));
        }
        if let Some(ref rids) = self.classify_rule_ids {
            map.insert(
                "classify_rule_ids".into(),
                Value::Array(rids.iter().map(|s| Value::String(s.clone())).collect()),
            );
        }
        if let Some(ref ev) = self.classify_evidence {
            map.insert("classify_evidence".into(), ev.clone());
        }
        if let Some(ref ov) = self.classify_overall_verdict {
            map.insert("classify_overall_verdict".into(), Value::String(ov.clone()));
        }
        if let Some(ref vr) = self.classify_verdict_reason {
            map.insert("classify_verdict_reason".into(), Value::String(vr.clone()));
        }

        // Extensions fields (conditional inclusion)
        if let Some(ref prov) = self.provenance {
            if let Ok(v) = serde_json::to_value(prov) {
                map.insert("provenance".into(), v);
            }
        }
        if !self.causal_links.is_empty() {
            if let Ok(v) = serde_json::to_value(&self.causal_links) {
                map.insert("causal_links".into(), v);
            }
        }
        if !self.extensions.is_empty() {
            if let Ok(v) = serde_json::to_value(&self.extensions) {
                map.insert("extensions".into(), v);
            }
        }

        Value::Object(map)
    }

    /// Computes the cryptographic SHA-256 hash over prev_hash and canonical proto.
    pub fn compute_hash(&self) -> String {
        let proto = self.hash_proto();
        let payload = format!("{}{}", self.prev_hash, canonical(&proto));
        sha256_hex(&payload)
    }

    /// Validates all field bounds, link hygiene, and hash correctness.
    pub fn validate(&self) -> Result<(), String> {
        if self.ts.trim().is_empty() {
            return Err(format!("{}: ts cannot be empty", AUDIT_EXT_ERR_VALIDATION));
        }
        if self.actor.trim().is_empty() {
            return Err(format!("{}: actor cannot be empty", AUDIT_EXT_ERR_VALIDATION));
        }
        if self.actor_id.trim().is_empty() {
            return Err(format!("{}: actor_id cannot be empty", AUDIT_EXT_ERR_VALIDATION));
        }
        if self.tool.trim().is_empty() {
            return Err(format!("{}: tool cannot be empty", AUDIT_EXT_ERR_VALIDATION));
        }
        if self.prev_hash.len() != 64 {
            return Err(format!("{}: prev_hash must be 64 characters", AUDIT_EXT_ERR_HASH));
        }
        if self.hash.len() != 64 {
            return Err(format!("{}: hash must be 64 characters", AUDIT_EXT_ERR_HASH));
        }

        if let Some(ref prov) = self.provenance {
            prov.validate()?;
        }

        if self.causal_links.len() > MAX_CAUSAL_LINKS {
            return Err(format!(
                "{}: causal_links count {} exceeds maximum {}",
                AUDIT_EXT_ERR_BOUNDS,
                self.causal_links.len(),
                MAX_CAUSAL_LINKS
            ));
        }
        for link in &self.causal_links {
            link.validate()?;
        }

        if let Some(ref sig) = self.signature {
            sig.validate()?;
        }

        if self.extensions.len() > MAX_EXTENSION_ENTRIES {
            return Err(format!(
                "{}: extensions count {} exceeds maximum {}",
                AUDIT_EXT_ERR_BOUNDS,
                self.extensions.len(),
                MAX_EXTENSION_ENTRIES
            ));
        }

        let serialized_ext = serde_json::to_string(&self.extensions).unwrap_or_default();
        if serialized_ext.len() > MAX_EXTENSION_PAYLOAD_BYTES {
            return Err(format!(
                "{}: extensions payload {} bytes exceeds limit of {}",
                AUDIT_EXT_ERR_BOUNDS,
                serialized_ext.len(),
                MAX_EXTENSION_PAYLOAD_BYTES
            ));
        }

        self.verify_hash()
    }

    /// Verifies that the stored hash matches the recomputed hash.
    pub fn verify_hash(&self) -> Result<(), String> {
        let expected = self.compute_hash();
        if self.hash != expected {
            return Err(format!(
                "{}: row hash mismatch (stored={}, computed={})",
                AUDIT_EXT_ERR_HASH, self.hash, expected
            ));
        }
        Ok(())
    }

    /// Converts this extended row to the legacy `AuditRow` representation.
    pub fn to_legacy_row(&self) -> AuditRow {
        AuditRow {
            id: self.id,
            ts: self.ts.clone(),
            actor: self.actor.clone(),
            actor_id: self.actor_id.clone(),
            tool: self.tool.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            target: self.target.clone(),
            outcome: self.outcome.clone(),
            outcome_detail: self.outcome_detail.clone(),
            constitution_rev: self.constitution_rev.clone(),
            grant_token: self.grant_token.clone(),
            c_flags: self.c_flags.clone(),
            policy_revision: self.policy_revision.clone(),
            classify_rule_ids: self.classify_rule_ids.clone(),
            classify_evidence: self.classify_evidence.clone(),
            classify_overall_verdict: self.classify_overall_verdict.clone(),
            classify_verdict_reason: self.classify_verdict_reason.clone(),
            prev_hash: self.prev_hash.clone(),
            hash: self.hash.clone(),
        }
    }

    /// Creates an `ExtendedAuditRow` from a legacy `AuditRow`.
    pub fn from_legacy_row(row: AuditRow) -> Self {
        Self {
            id: row.id,
            ts: row.ts,
            actor: row.actor,
            actor_id: row.actor_id,
            tool: row.tool,
            command: row.command,
            args: row.args,
            target: row.target,
            outcome: row.outcome,
            outcome_detail: row.outcome_detail,
            constitution_rev: row.constitution_rev,
            grant_token: row.grant_token,
            c_flags: row.c_flags,
            policy_revision: row.policy_revision,
            classify_rule_ids: row.classify_rule_ids,
            classify_evidence: row.classify_evidence,
            classify_overall_verdict: row.classify_overall_verdict,
            classify_verdict_reason: row.classify_verdict_reason,
            provenance: None,
            causal_links: Vec::new(),
            signature: None,
            extensions: HashMap::new(),
            prev_hash: row.prev_hash,
            hash: row.hash,
        }
    }
}
