# Task Evidence: T-02634 — Secrets Handling MCP/API Surface Implementation

## 1. Summary
Implemented the full runtime behavior for the Secrets Handling tool surface in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
- Implemented `aios.secret.store`:
  - Validates `id`, `name`, `kind`, and bounds.
  - Rejects store paths containing directory traversal (`..`).
  - Serializes and atomically commits to the target vault file.
- Implemented `aios.secret.get`:
  - Resolves secret metadata and evaluates caller scope containment against secret scope.
  - Enforces masked output by default (`val.masked_display()`).
  - Discloses raw secret bytes only when caller passes `expose: true`.
- Implemented `aios.secret.list`:
  - Returns array of secret metadata without payload leakage.
  - Supports filtering by `kind` and `scope`.
- Implemented `aios.secret.rotate`:
  - Validates updated payload bounds (<= 64 KiB).
  - Increments secret version and recalculates SHA-256 fingerprint.
- Implemented `aios.secret.revoke`:
  - Transitions secret to revoked state and locks further reads.
- Integrated all tools into `dispatch::recorded_call`, preserving PEP evaluation and audit trail logging.

## 2. Invariants Preserved
- **SECMCP1**: Masked retrieval by default.
- **SECMCP2**: Zero payload disclosure in listing.
- **SECMCP3**: Capped store size (1 MiB) and payload size (64 KiB).
- **SECMCP4**: Atomic file persistence with tempfile cleanup.
- **SECMCP5**: Every consequential call writes exactly one audit row to `AuditRing`.
