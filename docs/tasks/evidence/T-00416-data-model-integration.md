# T-00416 — Documentation Index Control / data model: Integration

## 1. Integration Scope
This task integrates the Documentation Index Control data model (`DocIndexEntry`, `DocIndexManifest`) into `aiosh-core`'s public API surface and verifies workspace-wide compilation and visibility.

## 2. Integration Details
- **Module Export (`code/aiosh-rust/aiosh-core/src/lib.rs`)**:
  - Exported `pub mod doc_index;` making all documentation indexing types and operations accessible to downstream consumers (`aiosh-cli`, `aiosh-mcp`).
- **Workspace Conformance**:
  - `cargo check --workspace` builds all crates (`aiosh-core`, `aiosh-cli`, `aiosh-sandbox`, `aiosh-mcp`) without type, borrow, or module resolution errors.
- **Cross-Substrate Parity**:
  - `DocIndexManifest` uses standard `serde` serialization to ensure byte-consistent JSON across CLI and MCP JSON-RPC layers.

## 3. Verification
- `cargo test -p aiosh-core doc_index::tests` -> PASS (9/9 unit tests)
- `cargo check --workspace` -> PASS (Exit 0)
