# T-02302: Audit Chain Extensions Data Model Specification

## 1. Domain Types & Data Structures

### 1.1 Cryptographic Signature Block
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSignature {
    /// Signing algorithm: "ed25519" or "secp256k1"
    pub algorithm: String,
    /// Hex-encoded public key of the signer
    pub public_key: String,
    /// Hex-encoded cryptographic signature over the row hash
    pub signature: String,
}
```

### 1.2 Causal DAG Provenance Link
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditCausalLink {
    /// SHA-256 hash of the parent causal audit event
    pub parent_event_hash: String,
    /// Type of causal dependency: "delegation", "subtask", "trigger", "retry"
    pub causality_type: String,
}
```

### 1.3 Execution & PEP Provenance Context
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuditProvenance {
    /// Interactive session or workflow UUID
    pub session_id: Option<String>,
    /// Bound PEP authorization grant identifier
    pub pep_grant_id: Option<String>,
    /// Delegation depth at execution time
    pub delegation_depth: u32,
    /// OpenTelemetry-compatible distributed trace ID
    pub trace_id: Option<String>,
    /// OpenTelemetry-compatible span ID
    pub span_id: Option<String>,
}
```

### 1.4 Extended Audit Row Entity
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtendedAuditRow {
    pub id: i64,
    pub ts: String,
    pub actor: String,
    pub actor_id: String,
    pub tool: String,
    pub command: String,
    pub args: serde_json::Value,
    pub target: Option<String>,
    pub outcome: String,
    pub outcome_detail: Option<String>,
    pub constitution_rev: Option<String>,
    pub grant_token: Option<String>,
    pub c_flags: crate::types::CFlags,
    pub policy_revision: Option<String>,
    pub classify_rule_ids: Option<Vec<String>>,
    pub classify_evidence: Option<serde_json::Value>,
    pub classify_overall_verdict: Option<String>,
    pub classify_verdict_reason: Option<String>,
    // Extensions
    pub provenance: Option<AuditProvenance>,
    pub causal_links: Vec<AuditCausalLink>,
    pub signature: Option<AuditSignature>,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    // Hash chain
    pub prev_hash: String,
    pub hash: String,
}
```

## 2. Invariants & Validation Contracts

1. **Hash Chaining Invariant**:
   `row.hash == sha256_hex(row.prev_hash || canonical_json(row.hash_proto()))`.
   The `hash_proto` excludes `id` and `hash`. Optional fields that are `None` are omitted from the canonical JSON map to preserve backward hash parity with legacy Sprint 1-3 verifiers.
2. **Causal Link Constraints**:
   - `parent_event_hash` must be a valid 64-character lowercase hexadecimal SHA-256 digest.
   - Max causal links per row: 16.
3. **Provenance Limits**:
   - `session_id`, `trace_id`, `span_id`, and `pep_grant_id` must not exceed 128 characters.
   - Control characters or whitespace in identifiers are strictly prohibited.
4. **Signature Constraints**:
   - `algorithm` must be supported (e.g. `"ed25519"`).
   - `signature` and `public_key` must be valid hex strings.
5. **Extension Payloads**:
   - Extension map keys must conform to namespace pattern `^[a-z0-9_.-]{1,64}$`.
   - Max extension entries: 32.
   - Max total serialized extension payload: 64 KiB.
