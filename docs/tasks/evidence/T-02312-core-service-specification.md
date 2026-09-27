# T-02312: Audit Chain Extensions Core Service Specification

## Overview
This specification defines the functional contract, API boundaries, error conditions, and state transitions for `AuditChainService` (`code/aiosh-rust/aiosh-core/src/audit_chain_service.rs`).

## Interface Definitions

### 1. Types & Data Structures

```rust
pub struct AuditQueryFilter {
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub actor: Option<String>,
    pub tool: Option<String>,
    pub parent_hash: Option<String>,
    pub limit: Option<usize>,
}

pub struct CausalLineageNode {
    pub row: ExtendedAuditRow,
    pub depth: usize,
    pub relationship: String,
}

pub struct CausalLineageReport {
    pub target_hash: String,
    pub ancestors: Vec<CausalLineageNode>,
    pub max_depth_reached: bool,
}

pub struct SignatureVerificationReport {
    pub target_hash: String,
    pub has_signature: bool,
    pub algorithm: Option<String>,
    pub is_valid: bool,
    pub error: Option<String>,
}
```

### 2. Service Methods

```rust
pub struct AuditChainService {
    ring: AuditRing,
}

impl AuditChainService {
    pub fn new(ring: AuditRing) -> Self;
    pub fn ring(&self) -> &AuditRing;
    pub fn ring_mut(&mut self) -> &mut AuditRing;

    /// Appends an extended event to the ring, validating all field bounds.
    pub fn record_event(&mut self, input: ExtendedAuditRowInput) -> Result<ExtendedAuditRow, String>;

    /// Finds a single row by its SHA-256 hash.
    pub fn get_row_by_hash(&self, hash: &str) -> Result<Option<ExtendedAuditRow>, String>;

    /// Queries events matching filtering criteria.
    pub fn query_events(&self, filter: &AuditQueryFilter) -> Result<Vec<ExtendedAuditRow>, String>;

    /// Traverses the DAG upwards following `parent_event_hash` causal links.
    pub fn trace_ancestry(&self, target_hash: &str, max_depth: usize) -> Result<CausalLineageReport, String>;

    /// Verifies the digital signature attached to an event.
    pub fn verify_event_signature(&self, target_hash: &str) -> Result<SignatureVerificationReport, String>;

    /// Full cryptographic verify of the continuous live hash ring.
    pub fn verify_integrity(&self) -> Result<VerifyResult, String>;
}
```

## Error Handling & Invariants
- `record_event`: Enforces `input.base.validate()` and extended bounds (`MAX_CAUSAL_LINKS=16`, `MAX_EXTENSION_ENTRIES=32`, `MAX_EXTENSION_PAYLOAD_BYTES=64KB`). Rejects invalid inputs with descriptive error string.
- `trace_ancestry`: Detects cycles and limits traversal depth to `min(max_depth, 64)`.
- Reused Interfaces: Reuses `AuditRing`, `ExtendedAuditRowInput`, `ExtendedAuditRow`, `VerifyResult`.
