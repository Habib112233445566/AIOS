# Task Evidence: T-01921 - System Update Mechanism / CLI surface: Research

## 1. Overview
- **Task ID**: `T-01921`
- **Sub-Epic**: 3 (System Update Mechanism CLI Surface)
- **Goal**: Research CLI command structures, operator ergonomics, argument parsing patterns, audit logging integrations, and CLI invariants `UCLI1..UCLI6` for the System Update Mechanism.

---

## 2. Research Findings

### 2.1 Command Structure & Operator Workflows
The CLI command `aiosh update` (aliased as `aiosh upd`) exposes operator commands:
1. `aiosh update status [--state-dir PATH] [--json]`:
   - Reports current update state, active version, target version, active slot, and progress percentage.
2. `aiosh update slots [--state-dir PATH] [--json]`:
   - Reports dual boot partition configuration (`current_slot`, `target_slot`, `rollback_slot`, `slot_a_version`, `slot_b_version`).
3. `aiosh update check <manifest_path> [--state-dir PATH] [--staging-dir PATH] [--json]`:
   - Validates the update manifest against current running version, ensures valid channel and artifacts, and prepares staging.
4. `aiosh update apply [--state-dir PATH] [--json]`:
   - Validates all staged artifacts and commits inactive target slot as the next boot slot.
5. `aiosh update confirm <running_version> [--state-dir PATH] [--json]`:
   - Confirms newly booted slot, updates slot success flag, and resets update state to `Idle`.
6. `aiosh update rollback [--state-dir PATH] [--json]`:
   - Triggers partition reversal to `rollback_slot` upon boot failure.

### 2.2 CLI Invariants (`UCLI1..UCLI6`)
- **`UCLI1` (Subcommand Dispatch & Help Ergonomics)**: Root command with no args, `--help`, or `-h` prints structured usage banner and returns exit code 0. Unknown subcommands return exit code 2.
- **`UCLI2` (Path Hygiene)**: `--state-dir` and `--staging-dir` arguments are rejected if longer than 1024 characters or containing control characters (exit code 2).
- **`UCLI3` (Envelope Consistency)**: When `--json` is supplied, all outputs conform to standard AIOS JSON envelope:
  - Success: `{"code": 0, "data": <payload>, "error": null}`
  - Error: `{"code": <1|2>, "data": null, "error": {"code": <ERR_STR>, "message": <MSG>}}`
- **`UCLI4` (Audit Ring Emission)**: All commands emit tamper-evident audit records into the SQLite WAL ring via `classify_and_emit()`.
- **`UCLI5` (Hermetic File Isolation)**: CLI flags allow hermetic overrides (`--state-dir`, `--staging-dir`) for test execution without requiring root or modifying `/var/lib/aiosh`.
- **`UCLI6` (Deterministic Exit Codes)**:
  - `0`: Success.
  - `1`: Operational or domain validation failure.
  - `2`: Bad arguments, syntax error, or path hygiene violation.

---

## 3. Implementation Plan
- Add `Some("update") | Some("upd") => cmd_update(&args[1..])` in `main()` of `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Implement `cmd_update(args: &[String]) -> i32`.
- Add unit tests module `update_cli_tests` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Add Python integration smoke test `code/aiosh-cli/tests/test_system_update_cli_smoke.py`.
