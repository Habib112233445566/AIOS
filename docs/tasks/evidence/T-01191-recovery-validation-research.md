# T-01191: Base Image Build Recovery & Validation Research

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Recovery & Validation  
**Task ID:** T-01191  

## 1. Executive Summary & Objective
Task `T-01191` establishes facts, constraints, and prior art for the **Recovery & Validation** subsystem of Base Image Build (`T-01191..T-01200`). This subsystem ensures registry resilience against on-disk corruption, partial writes, and schema drift, providing automated non-destructive recovery and deep health validation across all registered image manifests.

## 2. Codebase Audit & Existing Mechanisms

### Current State of Registry Loading
- `code/aiosh-rust/aiosh-core/src/base_image_service.rs`:
  - `ImageStore::load_from_path`: Bounded to 10 MiB, reads JSON and deserializes into `BTreeMap<String, BaseImageManifest>`.
  - Current failure mode: If the JSON file is corrupt, truncated, or unreadable, `load_from_path` returns `Err(String)`. It does not recover or provide fallback isolation.
  - Current store seeding: `ImageStore::new()` seeds 4 canonical manifests (`debian-12-minimal-raw`, `debian-12-minimal-qcow2`, `debian-12-minimal-iso`, `alpine-319-container-tarball`).

### Prior Art in AIOS (`DistroStore::load_or_recover`)
- In `code/aiosh-rust/aiosh-core/src/distro_service.rs` (`T-01091..T-01100`):
  - `load_or_recover` pattern:
    1. If file does not exist, initialize fresh store and write to disk.
    2. If file exists and parses cleanly, return store.
    3. If file is corrupt or unreadable, backup corrupted file to `<path>.bak.<timestamp>`, log warning, and reseed with default canonical profiles.
  - Deep validation report (`DistroValidationReport`):
    - `healthy: bool` (`healthy == errors.is_empty()`)
    - Tracks valid vs invalid profiles and specific schema violations.
  - CLI `aiosh distro check` and MCP `aios.distro.check`.

## 3. Invariants & Health Constraints (`RV1..RV4`)
- **`RV1`**: `valid_manifests + invalid_manifests == total_manifests`
- **`RV2`**: `healthy == (errors.is_empty() && invalid_manifests == 0)`
- **`RV3`**: `errors.len() >= invalid_manifests` (each invalid manifest yields at least one explicit error message)
- **`RV4`**: Non-destructive corruption handling (original corrupted payload is preserved in `.bak.<timestamp>` prior to replacement).

## 4. Facts vs. Assumptions

### Facts
- On-disk corruption (e.g. abrupt power loss, system crash, concurrent write collision) can leave `/var/lib/aios/images` in a truncated or malformed state.
- Without automated recovery, CLI and MCP commands fail-closed and become completely inoperable for autonomous agents.
- Creating a timestamped backup preserves forensic evidence and prevents irreversible data loss.

### Assumptions
- In production, `/var/lib/aios/images` or `./images.json` is owned by the local AIOS daemon/process.
- Reseeding with reference Debian 12 minimal and Alpine manifests restores full baseline operational readiness.

## 5. Decisions Needed for Specification (`T-01192`)
1. **Module Placement**: Should recovery and validation logic live in a dedicated module `base_image_recovery.rs` or extend `base_image_service.rs`?
   - *Decision*: Place in `code/aiosh-rust/aiosh-core/src/base_image_recovery.rs` to maintain high cohesion and follow the modular pattern established by `base_image_config.rs`, `base_image_policy.rs`, and `base_image_observability.rs`.
2. **CLI Command**: Expose as `aiosh image check [--fix] [--json] [--store <path>]`.
   - `--fix`: When supplied, automatically repairs invalid or corrupted stores.
3. **MCP Tool**: Expose as `aios.image.check` with optional `store_path` and `auto_recover` parameters.
4. **Standalone Test Criterion**: Introduce criterion `B9` in `tools/test_image_suites.py` validating recovery and health checking.
