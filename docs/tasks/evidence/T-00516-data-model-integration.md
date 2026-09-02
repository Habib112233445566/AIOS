# T-00516 — Evidence & Audit Trail / data model: Integration

## 1. Integration Scope
This task integrates the `evidence` data model into `code/aiosh-rust/aiosh-core/src/lib.rs` and the workspace dependency graph, providing programmatic access to `EvidenceRecord`, `EvidenceStep`, `TaskEvidenceManifest`, and `EvidenceVerificationReport`.

## 2. Integrated Exports
- `code/aiosh-rust/aiosh-core/src/lib.rs`:
  - Registered `pub mod evidence;`.
  - Exposed data models across `aiosh-core`, `aiosh-cli`, and `aiosh-mcp`.

## 3. Verification
- `cargo test --workspace` -> 138 unit tests in `aiosh_core` + 2 in `aiosh_mcp` all pass green.
