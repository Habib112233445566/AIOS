# T-00536 — Evidence & Audit Trail / CLI surface: Integration

## 1. Integration Scope
This task verifies that the `aiosh evidence` CLI surface (`verify`, `hash`, `scan`) is fully integrated into the global dispatcher, discoverable via `--help`, emits structured events to the SQLite WAL audit ring, and passes all smoke suites.

## 2. Integrated Points
- `aiosh --help` lists `aiosh evidence <verify|hash|scan>`.
- `aiosh evidence <verify|hash|scan>` executes cleanly and writes audit entries.
- `code/aiosh-cli/tests/test_evidence_cli_smoke.py` passes all 8 test cases.

## 3. Verification
- `cargo test --workspace` -> 143 unit tests in `aiosh_core` + 2 in `aiosh_mcp`.
- `python code/aiosh-cli/tests/test_evidence_cli_smoke.py` -> All 8 tests PASS.
