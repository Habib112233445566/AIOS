# T-00496 — Documentation Index Control / documentation: Integration

## 1. Integration Scope
This task integrates `format_doc_index_summary` into the `aiosh doc show` CLI command path, providing unified human-readable formatting across terminal output modes.

## 2. Integrated Changes
- `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `aiosh doc show` invokes `format_doc_index_summary(&manifest)` for standard terminal output.
  - `--json` mode returns the canonical `DocIndexManifest` JSON payload.
  - Both paths record structured audit entries in SQLite WAL.

## 3. Verification
- `cargo test --workspace` -> 126 passed in `aiosh_core`, 2 passed in `aiosh_mcp`.
- `python code/aiosh-cli/tests/test_doc_cli_smoke.py` -> PASS.
