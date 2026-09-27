# T-02314: Audit Chain Extensions Core Service Implementation

## Overview
This task implements `AuditChainService` (`code/aiosh-rust/aiosh-core/src/audit_chain_service.rs`) providing complete lineage traversal, multi-attribute event querying, digital signature verification, and chain integrity validation.

## Key Implementation Features
1. **`record_event`**: Validates input bounds via `ExtendedAuditRowInput::validate()`, computes SHA-256 hash chaining over the extended canonical JSON prototype, and writes into `audit_ring`.
2. **`get_row_by_hash`**: Retrieves rows by exact 64-character SHA-256 hex hash using indexed SQLite lookups and deserializes extended attributes.
3. **`query_events`**: Flexible multi-attribute filtering supporting `actor`, `tool`, `session_id`, `trace_id`, and `parent_hash` with bounded pagination limit.
4. **`trace_ancestry`**: Breadth-first traversal up the causal DAG, resolving parent audit events via `AuditCausalLink`, preventing circular loops via `HashSet<String>` visited tracker, and capping traversal depth at `MAX_LINEAGE_DEPTH` (64).
5. **`verify_event_signature`**: Validates asymmetric digital signatures against row hashes.
6. **`verify_integrity`**: Validates end-to-end continuous SHA-256 chain integrity across legacy and extended records.

## Build Status
- `cargo check -p aiosh-core` compiles cleanly with zero warnings.
