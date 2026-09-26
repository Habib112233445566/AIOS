# T-02303: Audit Chain Extensions Data Model Scaffold

## Overview
This task creates the scaffold for the Audit Chain Extensions data model in `code/aiosh-rust/aiosh-core/src/audit_chain_ext.rs`, exporting new provenance, DAG causality, and asymmetric signature structures into the core library.

## Scaffold Elements Created

### 1. Source Module
- **Path**: `code/aiosh-rust/aiosh-core/src/audit_chain_ext.rs`
- **Module Declarations**:
  - `pub mod audit_chain_ext;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
  - Re-exports of `AuditSignature`, `AuditCausalLink`, `AuditProvenance`, `ExtendedAuditRow`, and error constants.

### 2. Core Structs & Interfaces
- `AuditSignature`: Encapsulates `algorithm`, `public_key`, and `signature`.
- `AuditCausalLink`: Encapsulates `parent_event_hash` (64-char hex) and `causality_type`.
- `AuditProvenance`: Encapsulates `session_id`, `pep_grant_id`, `delegation_depth`, `trace_id`, `span_id`.
- `ExtendedAuditRow`: Core entity maintaining full Sprint 1-3 backward compatibility while accommodating cryptographic signatures and causal DAG links.
- Methods:
  - `hash_proto(&self) -> Value`: Constructs canonical dictionary omitting empty/null extension fields.
  - `compute_hash(&self) -> String`: Computes `sha256(prev_hash || canonical(proto))`.
  - `validate(&self) -> Result<(), String>`: Comprehensive bound and invariant checks.
  - `verify_hash(&self) -> Result<(), String>`: Verifies integrity of recorded SHA-256 digest.
  - `to_legacy_row(&self) -> AuditRow`: Transparent conversion to legacy `AuditRow`.
  - `from_legacy_row(row: AuditRow) -> Self`: Upgrade constructor from legacy `AuditRow`.

### 3. Build & Compilation Verification
- Compilation verified with `cargo check -p aiosh-core` yielding zero errors and zero warnings.
