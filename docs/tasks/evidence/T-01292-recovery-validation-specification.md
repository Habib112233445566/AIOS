# T-01292: Package Management Recovery & Validation Specification

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Recovery & Validation  
**Task ID:** T-01292  

---

## 1. Specification Overview & Objectives
This specification establishes the interface, behavioral contracts, error envelopes, and audit side-effects for the **Recovery & Validation** subsystem of Package Management (`T-01291..T-01300`).

The recovery & validation subsystem guarantees store integrity, structural adherence to package standards (Debian/Alpine format constraints), forensic preservation of damaged state, and automated zero-downtime self-healing for autonomous agent operations.

---

## 2. Core Data Models & Validation Reports

### `PackageValidationReport`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageValidationReport {
    pub store_path: String,
    pub total_packages: usize,
    pub valid_packages: usize,
    pub invalid_packages: usize,
    pub errors: Vec<String>,
    pub healthy: bool,
}
```

### Invariant Equations (RV1..RV4)
- **`RV1` (Count Conservation):**
  $$\text{valid\_packages} + \text{invalid\_packages} = \text{total\_packages}$$
- **`RV2` (Health Equivalence):**
  $$\text{healthy} \iff (\text{errors.is\_empty}() \land \text{invalid\_packages} == 0)$$
- **`RV3` (Error Completeness):**
  $$\text{errors.len}() \ge \text{invalid\_packages}$$
- **`RV4` (Forensic Preservation):**
  Before any destructive modification or reseeding of a corrupted package store, the original file content must be preserved at `<store_path>.bak.<unix_timestamp_secs>`.

---

## 3. Interfaces and Methods

### Subsystem Module: `package_recovery.rs`

```rust
pub fn validate_package_store(
    store: &PackageStore,
    store_path: &Path,
) -> PackageValidationReport;

pub fn load_or_recover(
    store_path: &Path,
) -> Result<(PackageStore, PackageValidationReport, bool, Option<PathBuf>), String>;
```

#### Behavior of `validate_package_store`:
1. Iterates over all packages in `store.list()`.
2. Evaluates structural validity for each package:
   - Package ID non-empty, length $\le 64$, matching `^[a-zA-Z0-9._-]+$`.
   - Package name non-empty, length $\le 64$, matching `^[a-zA-Z0-9._-]+$`.
   - Version string non-empty, length $\le 32$.
   - Distro matches supported set (`debian`, `alpine`, `arch`, `fedora`).
   - Architecture valid (`x86_64`, `aarch64`, `all`, `noarch`).
   - SHA-256 is 64 hex characters or valid placeholder.
   - Dependencies exist in store or represent external virtual provides.
3. Computes report satisfying invariants `RV1`, `RV2`, and `RV3`.

#### Behavior of `load_or_recover`:
1. **Case A: Missing File**
   - Initializes default reference `PackageStore::new()`.
   - Saves store to `store_path`.
   - Returns `(store, report, true, None)`.
2. **Case B: Existing Healthy File**
   - Loads store via `PackageStore::load_from_path(store_path)`.
   - Validates store via `validate_package_store`.
   - If `report.healthy == true`:
     - Returns `(store, report, false, None)`.
3. **Case C: Corrupted or Malformed File (JSON syntax error / truncation / invalid spec)**
   - Reads existing bytes and writes backup to `<store_path>.bak.<timestamp>`.
   - Reseeds default `PackageStore::new()`.
   - Saves clean store to `store_path`.
   - Validates newly reseeded store.
   - Returns `(store, report, true, Some(backup_path))`.

---

## 4. CLI Surface (`aiosh package check`)

### Command-line Syntax:
```bash
aiosh package check [--store <PATH>] [--fix] [--json]
```

### Options:
- `--store <PATH>`: Explicit path to package store (defaults to `/var/lib/aios/packages.json` or local equivalent).
- `--fix`: Enables automated recovery (backup corrupted file and reseed defaults).
- `--json`: Outputs structured JSON envelope matching ADR-0035.

### Standard ADR-0035 Exit & Response Envelopes:
- **Clean store (`--json`):**
  ```json
  {
    "ok": true,
    "code": 0,
    "data": {
      "store_path": "/var/lib/aios/packages.json",
      "total_packages": 8,
      "valid_packages": 8,
      "invalid_packages": 0,
      "errors": [],
      "healthy": true,
      "recovered": false,
      "backup_path": null
    },
    "error": null
  }
  ```
- **Corrupted store without `--fix`:**
  Exits with non-zero status code (`1`) and outputs error envelope:
  ```json
  {
    "ok": false,
    "code": 1,
    "data": null,
    "error": "Package store at /var/lib/aios/packages.json is corrupted: EOF while parsing at line 1 column 12. Run with --fix to recover."
  }
  ```
- **Corrupted store with `--fix`:**
  Exits with code `0`, outputs recovery notice and backup path.

---

## 5. MCP Surface (`aios.package.check`)

### Tool Definition:
```json
{
  "name": "aios.package.check",
  "description": "Validate on-disk package store integrity and optionally perform non-destructive recovery",
  "inputSchema": {
    "type": "object",
    "properties": {
      "store_path": {
        "type": "string",
        "description": "Optional custom path to the package store JSON file"
      },
      "auto_recover": {
        "type": "boolean",
        "description": "Whether to create a timestamped backup and reseed defaults if corrupted (default: false)"
      }
    }
  }
}
```

### JSON-RPC Response:
Returns standard `code`, `data`, and `error` envelope adhering to ADR-0035 / ADR-0036.

---

## 6. Audit Logging & Security Contract
- Every invocation of `aiosh package check` and `aios.package.check` classifies and records an audit row in `audit.db` using SQLite WAL mode.
- Action tag: `package.check` or `package.recover`.
- Non-destructive guarantee: Zero data loss; raw corrupted bytes are immutably archived into `.bak.<timestamp>` with restricted filesystem permissions.

---

## 7. Verification Plan (Criterion PM10)
- Master test runner `tools/test_package_suites.py` will include criterion `PM10` (`test_package_recovery.rs`).
- Tests cover:
  1. Missing file creation & default seed.
  2. Perfectly valid store passing `validate_package_store` with `healthy: true`.
  3. Structural errors (invalid ID characters, invalid distro, missing version) generating appropriate error entries while satisfying RV1..RV3.
  4. Truncated/garbled JSON store triggering non-destructive backup (`RV4`) and successful reseed when `--fix` / `auto_recover` is enabled.
  5. CLI `--json` output format and return codes.
