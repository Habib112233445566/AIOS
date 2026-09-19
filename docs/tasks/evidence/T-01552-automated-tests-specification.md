# T-01552 — Filesystem Layout automated tests: Specification

## Metadata
- **Task ID:** `T-01552`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — specification established for the automated test suite (`test_fs_layout_automated_cases.py`) covering test cases A1..A8 and aggregate runner criterion `FL10`.
- **Date:** 2026-09-19
- **Depends on:** `T-01551` (Automated Tests Research)
- **Feeds:** `T-01553` (Automated Tests Scaffold)
- **Artifacts:** `docs/tasks/evidence/T-01552-automated-tests-specification.md`, `docs/tasks/evidence/T-01552-spec.md`

---

## 1. Specification Overview

The automated test suite `code/aiosh-cli/tests/test_fs_layout_automated_cases.py` provides end-to-end integration and lifecycle verification for the Filesystem Layout subsystem, registered under `tools/test_fs_layout_suites.py` as criterion **FL10**.

---

## 2. Test Cases Specification (A1..A8)

### Case A1: Full Lifecycle State Machine Flow
- **Inputs:** A clean ephemeral store path, valid custom layout JSON specification (`id: custom-srv-v1`).
- **Steps:**
  1. `aiosh layout list --store <path>`: confirms built-ins (`standard_uefi`, `minimal_container`) present.
  2. `aiosh layout register --spec <file> --store <path>`: registers `custom-srv-v1`.
  3. `aiosh layout list --store <path>`: verifies `custom-srv-v1` present.
  4. `aiosh layout set-active custom-srv-v1 --store <path>`: sets active.
  5. `aiosh layout show --store <path>`: verifies active layout resolved as `custom-srv-v1`.
  6. `aiosh layout diff aios-uefi-standard-v1 custom-srv-v1 --store <path>`: computes partition/mount diff.
  7. `aiosh layout probe --bytes 100000000000 --store <path>`: probes feasibility against active layout.
  8. `aiosh layout set-active aios-uefi-standard-v1 --store <path>`: restores active layout.
  9. `aiosh layout remove custom-srv-v1 --store <path>`: cleanly deletes custom layout.
- **Expected Output:** All commands return exit code `0` and standard JSON envelope `{"code":0,"data":...,"error":null}`.

### Case A2: Built-in Layout Deletion Protection
- **Inputs:** `--store <path>`, target `aios-uefi-standard-v1` or `aios-container-minimal-v1`.
- **Behavior:** `aiosh layout remove <id>` fails with exit code `1` and error message indicating built-in presets cannot be removed. Store remains intact.

### Case A3: Active Layout Deletion Protection
- **Inputs:** Target layout marked as currently active in the store.
- **Behavior:** `aiosh layout remove <id>` fails with exit code `1` and error message stating active layout cannot be removed. Store remains intact.

### Case A4: fstab Import & Generation Cycle
- **Inputs:** Valid Linux `fstab` file containing root `/`, `/boot/efi`, `/tmp`, and `/dev/shm`.
- **Behavior:**
  1. `aiosh layout import-fstab imported-v1 "Imported" --fstab <path> --store <path>`: synthesizes layout, validates FL1..FL6, and registers in store.
  2. `aiosh layout fstab imported-v1 --store <path>`: emits valid 6-field `fstab(5)` file content with correct pass numbers (root pass 1, others 0/2).

### Case A5: Block Device Capacity Probing
- **Inputs:** Device sizes: (a) undersized (< min bytes), (b) tight capacity (< 10% slack), (c) generous capacity (> 50% slack).
- **Behavior:**
  - Case (a): `feasible: false, error: "target disk size ... is smaller than required minimum ..."`.
  - Case (b): `feasible: true, warning: "target disk capacity provides only ...% slack headroom"`.
  - Case (c): `feasible: true, warning: null`.

### Case A6: Differential Analysis & Destructive Change Flagging
- **Inputs:** Layout specs with: (a) deleted partition / shrunk size, (b) added new mount point.
- **Behavior:**
  - Case (a): `diff` output contains `destructive: true`.
  - Case (b): `diff` output contains `destructive: false`.

### Case A7: Corrupted Store Tamper Resistance
- **Inputs:** A layout store JSON file corrupted with invalid JSON syntax or unparseable top-level fields.
- **Behavior:** Any mutating or query command fails closed (exit code 1), error envelope emitted, and the corrupt file is not overwritten.

### Case A8: Audit Trail Emission
- **Inputs:** Dedicated `AIOSH_HOME` with SQLite WAL `audit.db`.
- **Behavior:** Every CLI invocation in A1..A7 writes exactly one SHA-256 hash-chained record with matching `tool`, `target`, and `outcome`.

---

## 3. Interfaces & Reused Infrastructure

- **Binary Under Test:** `code/aiosh-rust/target/debug/aiosh.exe` (or release).
- **Runner Script:** `code/aiosh-cli/tests/test_fs_layout_automated_cases.py`.
- **Aggregate Integration:** `tools/test_fs_layout_suites.py::test_fl10_automated_cases()`.
- **No new dependencies:** Uses standard Python library (`subprocess`, `tempfile`, `json`, `sqlite3`, `pathlib`).

---

## 4. Acceptance Confirmation

- [x] Spec covers happy path, failure path, and audit effects.
- [x] Spec is reviewable without reading the implementation.
