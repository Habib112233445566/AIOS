# T-02304: Audit Chain Extensions Data Model Implementation

## Overview
This task implements the minimal working behavior and data structures for the Audit Chain Extensions subsystem (`code/aiosh-rust/aiosh-core/src/audit_chain_ext.rs`).

## Key Implementations

### 1. Data Model Entities
- `AuditSignature`: Stores asymmetric signature metadata (`algorithm`, `public_key`, `signature`) verifying non-repudiation over the row hash.
- `AuditCausalLink`: Models explicit causal links between audit rows (`parent_event_hash` pointing to an ancestor event, with semantic relationship `causality_type` such as `"delegation"` or `"subtask"`).
- `AuditProvenance`: Tracks interactive `session_id`, bound `pep_grant_id`, `delegation_depth`, and distributed tracing identifiers (`trace_id`, `span_id`).
- `ExtendedAuditRow`: Core extended event row containing base attributes, Sprint-2 classifier fields, and Phase-2 extension attributes.

### 2. Invariant & Hashing Guarantees
- **Canonical Serialization**: `hash_proto()` constructs canonical JSON omitting empty or null extension fields to maintain byte-identical hash parity with legacy Sprint 1-3 audit rows.
- **Strict Chaining**: `compute_hash()` computes `sha256_hex(prev_hash || canonical(hash_proto))`.
- **Bidirectional Conversions**:
  - `to_legacy_row(&self) -> AuditRow`
  - `from_legacy_row(row: AuditRow) -> Self`
- **Validation**:
  - Bounds checks on identifiers ($\le 128$ chars), causal link counts ($\le 16$), extension keys ($\le 32$), and extension JSON payload size ($\le 64$ KiB).
  - Exact cryptographic hash verification (`verify_hash()`) detecting any tampering with command, arguments, provenance, or previous hash links.

## Test Verification
- Automated unit test suite in `code/aiosh-rust/aiosh-core/tests/test_audit_chain_ext.rs`:
  - `test_extended_audit_row_legacy_compatibility`: PASS
  - `test_extended_audit_row_with_provenance_and_causality`: PASS
  - `test_extended_audit_row_tamper_detection`: PASS
  - `test_extended_audit_row_bounds_enforcement`: PASS
- Result: 4/4 passing tests in 0.08s.
