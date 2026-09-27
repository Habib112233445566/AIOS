# T-02319: Audit Chain Extensions Core Service Documentation

## Overview
The `AuditChainService` (`code/aiosh-rust/aiosh-core/src/audit_chain_service.rs`) exposes high-level APIs for interacting with the extended audit trail, querying execution provenance, exploring causal DAG lineage, and verifying cryptographic signatures.

## Core API Methods

| Method | Parameters | Description |
|---|---|---|
| `record_event` | `ExtendedAuditRowInput` | Validates bounds and writes an extended event into `audit_ring`. |
| `get_row_by_hash` | `&str` (hash) | Fetches an extended audit event by exact SHA-256 hash. |
| `query_events` | `&AuditQueryFilter` | Multi-field query supporting session, trace, actor, tool, and parent links. |
| `trace_ancestry` | `&str` (hash), `usize` (max_depth) | Traverses backwards through `causal_links` DAG with cycle detection. |
| `verify_event_signature` | `&str` (hash) | Verifies digital signature block attached to the row. |
| `verify_integrity` | none | Verifies continuous cryptographic hash chain across all rows. |

## Usage Example

```rust
use aiosh_core::audit::{AuditRing, OpenOptions, AuditRowInput, ExtendedAuditRowInput};
use aiosh_core::audit_chain_ext::{AuditCausalLink, AuditProvenance};
use aiosh_core::audit_chain_service::{AuditChainService, AuditQueryFilter};

// Initialize service from standard audit DB
let ring = AuditRing::open(OpenOptions::default())?;
let mut service = AuditChainService::new(ring);

// 1. Record an event with provenance and causal link
let mut base = AuditRowInput::default();
base.actor = "agent:coordinator".into();
base.tool = "aios.pep.grant.issue".into();
base.command = "issue".into();

let mut input = ExtendedAuditRowInput::new(base);
input.provenance = Some(AuditProvenance {
    session_id: Some("session-wf-99".into()),
    pep_grant_id: Some("grant-root".into()),
    delegation_depth: 0,
    trace_id: Some("trace-w3c-001".into()),
    span_id: None,
});
let row = service.record_event(input)?;

// 2. Query events for session
let filter = AuditQueryFilter {
    session_id: Some("session-wf-99".into()),
    limit: Some(10),
    ..Default::default()
};
let events = service.query_events(&filter)?;
println!("Found {} events for session", events.len());

// 3. Trace DAG ancestry
let lineage = service.trace_ancestry(&row.hash, 10)?;
println!("Ancestors found: {}", lineage.ancestors.len());
```

## Constraints & Limitations
- Ancestry traversal is depth-limited to at most 64 hops.
- Query result sets are capped at 1,000 records per invocation.
- Sub-event hashes must exist in the SQLite database to resolve node details during DAG traversal.

## Linked Evidence
- Research: [T-02311](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02311-core-service-research.md)
- Specification: [T-02312](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02312-core-service-specification.md)
- Scaffold: [T-02313](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02313-core-service-scaffold.md)
- Implementation: [T-02314](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02314-core-service-implementation.md)
- Unit Tests: [T-02315](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02315-core-service-unit-test.md)
- Integration: [T-02316](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02316-core-service-integration.md)
- Security Review: [T-02317](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02317-core-service-security-review.md)
- Hardening: [T-02318](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02318-core-service-hardening.md)
