# T-00761 — Secrets & Access Hygiene / automated tests: Research

## 1. Prior Art & Existing Automated Test Landscape
- **Unit Test Layers**:
  - `aiosh-core::secrets::tests`: Data model serialization, discrete severity ordering, and UTF-8 safe redaction.
  - `aiosh-core::secrets_service::tests`: File-level scanners (`SEC-001`..`SEC-005`), workspace recursion, binary filtering, and config integration.
  - `aiosh-core::secrets_config::tests`: Deserialization, schema bounds, and JSON roundtrips.
  - `aiosh-cli::task_cli_tests::test_cmd_secrets_scan_and_check`: CLI flags (`--file`, `--repo`, `--config`, `--json`) and exit codes.
  - `aiosh-mcp::tests::test_mcp_secrets_tools_execution`: MCP JSON-RPC tool schemas and dispatch.
- **Standalone Test Suite Runner (`tools/test_secrets_suites.py`)**:
  - Python-based test orchestrator verifying all components across criteria K1..K7 with structured terminal output and non-zero exit codes on failure.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Test Harness | Fact | `tools/test_secrets_suites.py` provides standalone zero-dependency verification runnable locally and in CI. |
| Test Matrix | Fact | Covers K1 (data model), K2 (private keys), K3 (API tokens), K4 (configs/env), K5 (CLI surface), K6 (MCP server), K7 (configuration). |
| Integration Coverage | Fact | End-to-end multi-crate integration verifies data flow from filesystem to redact engine to CLI / MCP output. |

## 3. Decisions & Contracts Needed
1. Create a dedicated multi-crate integration test file `code/aiosh-rust/aiosh-cli/tests/secrets_integration_tests.rs` or comprehensive test harness validating the full scanning pipeline against synthetic test workspaces.
