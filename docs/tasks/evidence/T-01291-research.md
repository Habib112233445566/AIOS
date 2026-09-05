# T-01291: Package Management Recovery & Validation Research

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Recovery & Validation  
**Task ID:** T-01291  

---

## 1. Executive Summary & Objective
Task `T-01291` establishes facts, architectural constraints, and prior art for the **Recovery & Validation** subsystem of Package Management (`T-01291..T-01300`). This subsystem provides resilience against corrupted on-disk package stores, invalid or drifted specifications, broken dependency closures, and accidental truncated files, ensuring that autonomous AI agents and operators can detect store corruption and perform automated non-destructive self-healing.

---

## 2. Codebase Audit & Existing Mechanisms

### Current State of Package Store Loading
- `code/aiosh-rust/aiosh-core/src/package_service.rs`:
  - `PackageStore::load_from_path(path: &Path)`:
    - Verifies file existence and reads file bounded to 10 MiB (`10 * 1024 * 1024` bytes).
    - Deserializes into `BTreeMap<String, PackageSpec>`.
    - Enforces entity limit of $\le 10,000$ packages.
    - Failure mode: If the file is malformed, truncated, or unreadable, `load_from_path` returns `Err(String)`. It currently does not perform automatic recovery or fallback quarantine.
  - `PackageStore::new()`: Seeds reference packages for Debian 12 (`libc6`, `coreutils`, `bash`, `libssl3`, `curl`) and Alpine 3.19 (`musl`, `busybox`, `apk-tools`).

### Prior Art in AIOS (`base_image_recovery.rs` & `distro_service.rs`)
1. **`load_or_recover` Pattern**:
   - If the store file does not exist, initialize a clean store seeded with defaults and persist it.
   - If the file exists and deserializes cleanly, validate specification constraints.
   - If the file is corrupted (e.g. truncated JSON, invalid bytes), create a timestamped backup (`<path>.bak.<timestamp>`), log a warning, and reseed with default reference packages.
2. **Deep Validation Report (`PackageValidationReport`)**:
   - `total_packages: usize`
   - `valid_packages: usize`
   - `invalid_packages: usize`
   - `errors: Vec<String>`
   - `healthy: bool`
3. **Operational Surfaces**:
   - Operator CLI: `aiosh package check [--store <path>] [--fix] [--json]`.
   - Autonomous Agent MCP: `aios.package.check` tool with `store_path` and `auto_recover` parameters.
4. **Standalone Test Runner**:
   - Master suite `tools/test_package_suites.py` criterion `PM10` validating recovery, corruption quarantine, and health validation.

---

## 3. Invariants & Health Constraints (RV1..RV4)
- **`RV1`**: `valid_packages + invalid_packages == total_packages`.
- **`RV2`**: `healthy == (errors.is_empty() && invalid_packages == 0)`.
- **`RV3`**: `errors.len() >= invalid_packages` (each invalid package produces at least one explicit error description).
- **`RV4`**: Non-destructive quarantine (corrupted store files are safely preserved in `<store_path>.bak.<timestamp>` prior to replacement, preserving forensic evidence).

---

## 4. Facts vs. Assumptions

### Facts (Empirically Verified in Codebase)
- Power loss, hard crashes, or concurrent process file writes can corrupt the JSON store on disk.
- If store loading fails closed without a recovery mechanism, agents cannot execute package queries or commands, creating an operational deadlock.
- Creating a timestamped `.bak` file before reseeding ensures zero accidental data loss while allowing continuous system operation.
- Invariants `PM1..PM5` define the exact structural validity rules for packages.

### Assumptions
- In production, `/var/lib/aios/packages.json` is owned by the local AIOS daemon/process.
- Reseeding with reference Debian 12 and Alpine packages restores baseline operational readiness.
- AI agents will invoke `aios.package.check` to evaluate package health prior to high-stakes system migrations.

---

## 5. Decisions Needed for Specification (`T-01292`)
1. **Module Placement**:
   - Place in `code/aiosh-rust/aiosh-core/src/package_recovery.rs` to maintain high cohesion and follow the modular pattern of `package_config.rs`, `package_policy.rs`, and `package_observability.rs`.
2. **CLI Surface**:
   - Expose `aiosh package check [--store <path>] [--fix] [--json]`.
   - When `--fix` is passed, corrupted files are backed up and reseeded.
3. **MCP Surface**:
   - Register `aios.package.check` tool accepting `store_path` and `auto_recover: bool`.
4. **Test Suite Matrix**:
   - Add criterion `PM10` (`test_package_recovery`) to `tools/test_package_suites.py`, bringing the sub-epic to full completion (PM1..PM10 PASS).
