# T-00661 — Repository Health / automated tests: Research

## 1. Research Context & Purpose
The automated test suite for Repository Health must provide deterministic, end-to-end verification across all sub-components: data models, diagnostic routines, CLI commands, MCP tool endpoints, and configuration systems.

## 2. Test Suite Architecture (Criteria H1..H7)

| ID | Domain | Assertion Target |
| :--- | :--- | :--- |
| **H1** | Data Model Integrity | `RepoHealthCheck` and `RepoHealthReport` creation, boundary checks, JSON roundtrip, status aggregation. |
| **H2** | Git Tree Hygiene | Porcelain v2 status parser, clean tree vs modified file tracking. |
| **H3** | File Bounds Scanner | Recursive tree walk, 16 MiB size limit, exclusion of `.git`, `target`, `node_modules`, `.venv`. |
| **H4** | Security Governance | Validation of root `SECURITY.md`, character length, absence of placeholder markers. |
| **H5** | CLI Surface | `aiosh repo health` and `check` stdout table formatting, `--json` flag, `--repo` path routing. |
| **H6** | MCP Tool Interface | `tools/list` schema declaration for `aios.repo.health`, JSON-RPC `tools/call` execution. |
| **H7** | Configuration & Hardening | `RepoHealthConfig` validation, 64 KiB config bounds, env overrides. |

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Working Assumption |
| :--- | :--- | :--- |
| **Test Runner Pattern** | Other subsystems use `tools/test_<name>_suites.py` returning exit code 0/1. | `tools/test_repo_health_suites.py` follows the established pattern. |
| **Cross-Platform Compatibility** | Python test runners execute seamlessly on Windows PowerShell and POSIX shells. | Standard library `subprocess` and `json` modules provide reliable cross-platform execution. |
| **Isolation** | Tests with disk mutations use `tempfile.TemporaryDirectory()`. | Temporary workspaces prevent polluting active repository state. |

## 4. Key Design Decisions for Implementation
1. Create standalone test runner `tools/test_repo_health_suites.py`.
2. Implement verification functions for all criteria `H1..H7`.
3. Ensure test runner can be run standalone or incorporated into CI pipelines.
