# T-00461 — Documentation Index Control / automated tests: Research

## 1. Goal
Establish facts, constraints, coverage objectives, and prior art for the automated test harness of Documentation Index Control.

## 2. Facts vs. Assumptions

### Facts (Empirical Repository Context):
1. **Existing Automated Test Architecture**:
   - Unit tests embedded in Rust crates (`aiosh-core`, `aiosh-cli`, `aiosh-mcp`).
   - Python smoke test scripts in `code/aiosh-cli/tests/` and `code/aiosh-mcp/tests/`.
   - Invariant check scripts in `tools/` (`check_task_docs.py`, `check_security_policy.py`, `test_ci_suites.py`, `test_ci_service.py`).
2. **Current Coverage Matrix**:
   - `doc_index.rs`: Manifest validation, duplicate paths, max limits.
   - `doc_index_service.rs`: Parsing markdown title/links, validating relative file resolution, root escapes.
   - `doc_index_config.rs`: Serialization, 64 KiB read limits, path validation.
   - `aiosh doc` CLI: Output formats (prose and JSON), argument parsing, error exits.
   - `aiosh-mcp`: `tools/list` registration, `aios.doc.*` tool invocations.

### Assumptions:
1. A unified test runner (`tools/test_doc_index_suites.py`) structured with explicit assertions (D1..D7) will ensure continuous automated regression testing across all Documentation Index Control layers.

## 3. Prior Art & Authoritative Specifications
- **`tools/test_ci_suites.py` (W1..W7)**: Deterministic, stdlib-only Python test harness with explicit assertion reporting.
- **`tools/check_task_docs.py` (C1..C6)**: Read-only invariant enforcement running in CI.

## 4. Proposed Test Suite Matrix (D1..D7)
- **D1**: Manifest data model integrity & query helpers.
- **D2**: Configuration hierarchy, env override, and default fallback.
- **D3**: Title parsing & Markdown link extraction accuracy.
- **D4**: File existence checking and root traversal containment.
- **D5**: CLI subcommands (`show`, `check`, `search`) and error envelopes.
- **D6**: MCP JSON-RPC tool schemas and invocation results.
- **D7**: Hardening limits (64 KiB config cap, 16 MiB doc read cap, 10,000 max entries).

## 5. Decisions Needed
1. **Runner Structure**: Implement `tools/test_doc_index_suites.py` using Python 3 stdlib only.
   - *Decision*: Adopt `tools/test_doc_index_suites.py` with criteria D1..D7.

## 6. Next Steps
Advance to Specification (T-00462) to define the test criteria contracts and error expectations.
