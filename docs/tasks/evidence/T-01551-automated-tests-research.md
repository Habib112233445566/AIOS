# T-01551 — Filesystem Layout automated tests: Research

## Metadata
- **Task ID:** `T-01551`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — research established prior art, existing test suite inventory, gaps, and decisions for the automated test suite.
- **Date:** 2026-09-19
- **Depends on:** `T-01550` (Configuration Verification & Evidence)
- **Feeds:** `T-01552` (Automated Tests Specification)
- **Artifacts:** `docs/tasks/evidence/T-01551-automated-tests-research.md`, `docs/tasks/evidence/T-01551-research.md`

---

## 1. Existing Test Inventory & Prior Art

The Filesystem Layout subsystem currently possesses the following test assets:
1. **In-Tree Rust Unit Tests (`code/aiosh-rust/`)**:
   - `aiosh-core/tests/test_fs_layout_data_model.rs`: 19 tests validating `FsType`, `PartitionType`, GPT GUIDs, path hygiene, and invariants FL1..FL6.
   - `aiosh-core/tests/test_fs_layout_service.rs`: 11 tests validating `FilesystemLayoutStore`, `FilesystemLayoutService`, differential comparison (`LayoutDiff`), disk probing, and atomic persistence.
   - `aiosh-cli/src/main.rs::test_cmd_fs_layout_flow`: In-crate CLI flow unit test.
   - `aiosh-mcp/src/main.rs::test_mcp_fs_layout_tools`: In-crate MCP tool dispatch test.
2. **External Python Harnesses (`code/aiosh-cli/tests/` and `code/aiosh-mcp/tests/`)**:
   - `test_fs_layout_cli_smoke.py` (`FL3`): CLI subcommands, flag precedence, JSON output formatting.
   - `test_fs_layout_audit_security.py` (`FL4`): ADR-0035 SQLite WAL audit row emission, CWE-150 control character escaping.
   - `test_fs_layout_hardening.py` (`FL7`): Non-regular paths, bounded reads (10 MiB), atomic persistence.
   - `test_fs_layout_mcp_contract.py` (`FL8`): C1..C9 test cases covering advertised inputSchema, audit target, destructive verdict, grant scope.paths confinement, and configuration parity.
   - `test_fs_layout_config_validation.py` (`FL9`): U1..U12 test cases covering D1..D9 configuration validation rules.
3. **Aggregate Orchestrator (`tools/test_fs_layout_suites.py`)**:
   - Coordinates execution across FL1..FL9, enforcing zero regressions across all crates and harnesses.

---

## 2. Authoritative Sources & Standards

- **Rust Testing Framework**: The Rust Reference §11 (Testing), `cargo test` conventions for unit and integration testing.
- **Python Subprocess & Isolation**: Python 3.11 standard library `subprocess` and `tempfile.TemporaryDirectory` with strict timeout bounds (180s per suite).
- **ADR-0035 (Audit Ring Architecture)**: All mutating and failed operations must emit hash-chained rows to `audit.db` with verifiable tamper evidence.
- **ADR-0034 (Policy Enforcement Point)**: Authorization gating via capability grants (`scope.tools`, `scope.paths`).
- **Linux Standards**: FHS 3.0 (Filesystem Hierarchy Standard) and Linux `fstab(5)` format.

---

## 3. Fact vs. Assumption Analysis

| Topic | Established Fact | Assumption / Working Hypothesis |
|---|---|---|
| **Subprocess Isolation** | Running CLI commands against a shared `AIOSH_HOME` leads to SQLite WAL contention and non-deterministic test results. | Isolated temp directories with explicit `AIOSH_HOME` ensure 100% deterministic test isolation. |
| **Command Coverage** | Existing suites test individual commands in isolation, but no single test exercises an end-to-end lifecycle (init -> import fstab -> register -> diff -> probe -> set-active -> remove). | A unified automated lifecycle test suite (`FL10`) will catch state-machine regressions across multi-step flows. |
| **Platform Compatibility** | The CLI binary runs cross-platform (Windows / Linux), handling path separators and platform quirks. | Python subprocess tests using `Path` abstractions run identically on Windows and Linux CI. |

---

## 4. Unknowns & Decisions Needed for T-01552

- **D1: What is the scope of the new automated test suite?**
  *Decision:* Implement `test_fs_layout_automated_cases.py` covering end-to-end lifecycle flows: creating custom layouts, importing fstab specs, registering, diffing, capacity probing, active switching, and safe deletion.
- **D2: What is the new aggregate runner criterion?**
  *Decision:* Register `FL10` in `tools/test_fs_layout_suites.py`.
- **D3: How should edge cases and corruptions be tested?**
  *Decision:* Test unexpected store truncation, read-only store files, concurrent command collisions, and corrupted JSON recovery.

---

## 5. Acceptance Confirmation

- [x] Evidence file exists and separates facts from assumptions.
- [x] No code changed; decisions needed are listed explicitly.
