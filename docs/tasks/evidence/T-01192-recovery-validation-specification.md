# T-01192: Base Image Build Recovery & Validation Specification

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Recovery & Validation  
**Task ID:** T-01192  

## 1. Specification Overview
This document specifies the exact schemas, validation algorithms, recovery workflows, error envelopes, and audit semantics for the Base Image Build Recovery & Validation subsystem.

## 2. Core Data Types & Enums

### `RecoveryAction`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    LoadedExisting,
    CreatedDefaultFresh,
    RecoveredFromBackup { backup_path: String, reason: String },
}
```

### `BaseImageValidationReport`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseImageValidationReport {
    pub healthy: bool,
    pub total_manifests: usize,
    pub valid_manifests: usize,
    pub invalid_manifests: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub generated_at: String,
}
```

## 3. Mathematical & Structural Invariants (`RV1..RV4`)
- **`RV1`**: `valid_manifests + invalid_manifests == total_manifests`
- **`RV2`**: `healthy == (errors.is_empty() && invalid_manifests == 0)`
- **`RV3`**: `errors.len() >= invalid_manifests`
- **`RV4`**: Non-destructive recovery. If on-disk store data is invalid or corrupt, the original payload must be copied to `<path>.bak.<timestamp>` prior to rewriting with the safe canonical store.

## 4. Manifest Deep Validation Rules
A manifest is judged `invalid` if any of the following fail:
1. **Identifier**: Non-empty, ASCII graphic printable characters only (`33..=126`), length $\le 128$ chars.
2. **Distribution ID**: Non-empty, ASCII alphanumeric and hyphens only, length $\le 64$ chars.
3. **Filesystem**: Must match one of the authorized filesystem types (`ext4`, `squashfs`, `btrfs`, `erofs`, `xfs`).
4. **Architecture**: Must match one of the authorized architectures (`x86_64`, `aarch64`, `riscv64`).
5. **Packages**: Must contain at least one package; no package name may contain control characters or be blacklisted.
6. **Kernel Configuration**: Kernel version must be non-empty; command line parameters must not contain blacklisted tokens (`nokaslr`, `mitigations=off`, etc.).
7. **Size Budget**: Must be positive and $\le 100\text{ GiB}$.
8. **Build Plan Synthesis**: `store.generate_build_plan(&manifest.id)` must synthesize a valid 4-stage build plan without error.

## 5. Automated Recovery Algorithm (`load_or_recover`)
```
function load_or_recover(path):
    if not exists(path):
        store = ImageStore::new()
        store.save_to_path(path)
        return (store, CreatedDefaultFresh)

    try:
        store = ImageStore::load_from_path(path)
        report = validate_store(store)
        if report.healthy:
            return (store, LoadedExisting)
        else:
            raise ValidationError(report.errors)
    catch Exception as e:
        timestamp = now_utc_compact()
        backup_path = path + ".bak." + timestamp
        copy(path, backup_path)
        store = ImageStore::new()
        store.save_to_path(path)
        return (store, RecoveredFromBackup(backup_path, e.message))
```

## 6. CLI Surface Contract
- Subcommand: `aiosh image check [--fix] [--json] [--store <path>]`
- Inputs:
  - `--store <path>`: Path to image registry JSON file (default: `/var/lib/aios/images`).
  - `--fix`: If provided, automatically trigger recovery on unhealthy stores.
  - `--json`: Output as JSON result envelope.
- Outputs:
  - Exit code `0` on healthy (or successfully fixed) store.
  - Exit code `1` on unhealthy store without `--fix`.
  - Exit code `2` on invalid argument syntax.

## 7. MCP Tool Surface Contract
- Tool Name: `aios.image.check`
- Arguments:
  - `store_path?: string` (string, max 4096 chars, ASCII graphic)
  - `auto_recover?: bool` (boolean, default false)
- Return value: JSON result containing `BaseImageValidationReport` and optional `RecoveryAction`.

## 8. Audit Effects
- Every invocation records an immutable hash-chained row in SQLite WAL (`audit.db`) capturing:
  - action: `image.check` or `image.repair`
  - status: `SUCCESS` or `VALIDATION_FAILED`
  - details: total manifests, error count, and recovery action taken.
