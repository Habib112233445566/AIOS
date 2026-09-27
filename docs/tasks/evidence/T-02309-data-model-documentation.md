# T-02309: Audit Chain Extensions Data Model Documentation

## Overview
The Audit Chain Extensions data model enhances the AIOS SQLite append-only audit ring with structured execution provenance, distributed trace identifiers, causal DAG lineage, asymmetric cryptographic signatures, and extensible payload namespaces.

## Data Structures

### 1. `AuditProvenance`
Records execution context for the audit event:
- `session_id`: Optional session UUID (max 128 chars).
- `pep_grant_id`: Associated PEP authorization grant token.
- `delegation_depth`: Integer depth in hierarchical delegation tree.
- `trace_id`: Distributed trace identifier (e.g., W3C TraceContext).
- `span_id`: Distributed span identifier.

### 2. `AuditCausalLink`
Defines an explicit causal dependency in the event DAG:
- `parent_event_hash`: 64-character lowercase hex SHA-256 hash of the parent audit event.
- `causality_type`: Semantic relationship ("delegation", "subtask", "trigger", "retry").

### 3. `AuditSignature`
Asymmetric cryptographic signature block:
- `algorithm`: Signing algorithm (e.g., "ed25519", "secp256k1").
- `public_key`: Hex-encoded public key.
- `signature`: Hex-encoded signature over the row's canonical SHA-256 hash.

### 4. `ExtendedAuditRow` & `ExtendedAuditRowInput`
Complete representation stored in `audit_ring`:
- Base fields: `ts`, `actor`, `actor_id`, `tool`, `command`, `args`, `target`, `outcome`, `c_flags`, etc.
- Extended fields: `provenance`, `causal_links`, `signature`, `extensions`.
- Hash chain: `prev_hash`, `hash`.

## Rust Usage Example

```rust
use aiosh_core::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};
use aiosh_core::audit_chain_ext::{AuditCausalLink, AuditProvenance, AuditSignature};
use serde_json::json;

// Open audit ring database
let mut ring = AuditRing::open_in_memory()?;

// Construct input
let mut base = AuditRowInput::default();
base.actor = "agent:worker-01".to_string();
base.tool = "aios.pep.grant.issue".to_string();
base.command = "issue".to_string();

let mut ext_input = ExtendedAuditRowInput::new(base);
ext_input.provenance = Some(AuditProvenance {
    session_id: Some("sess-1234".to_string()),
    pep_grant_id: Some("grant-root".to_string()),
    delegation_depth: 1,
    trace_id: Some("trace-5678".to_string()),
    span_id: Some("span-99".to_string()),
});
ext_input.causal_links = vec![AuditCausalLink::new(
    "0000111122223333444455556666777788889999aaaabbbbccccddddeeeeffff",
    "delegation",
)];

// Write extended row into ring
let row = ring.write_extended(ext_input)?;
println!("Appended extended audit event id={} hash={}", row.id, row.hash);

// Tail extended rows
let rows = ring.tail_extended(5)?;
assert_eq!(rows.len(), 1);
```

## Constraints & Limitations
1. **Link Cap**: At most 16 `causal_links` per row.
2. **Payload Size**: `extensions` map cannot exceed 32 entries or 64 KB total serialized size.
3. **Identifier Format**: Session and trace identifiers cannot contain ASCII control characters or whitespace.

## Related Evidence Artifacts
- Unit Tests: [T-02305](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02305-data-model-unit-test.md)
- Integration: [T-02306](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02306-data-model-integration.md)
- Security Review: [T-02307](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02307-data-model-security-review.md)
- Hardening: [T-02308](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02308-data-model-hardening.md)
