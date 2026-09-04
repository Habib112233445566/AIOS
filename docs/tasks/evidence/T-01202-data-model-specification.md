# T-01202: Package Management - Data Model: Specification

## Metadata
- **Task ID:** `T-01202`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model Specification
- **Status:** Complete

## 1. Scope & System Interfaces
This specification defines the core types, validation functions, and persistence contracts for the AIOS Package Management data model.

### Reused Interfaces
- Standard `serde::{Serialize, Deserialize}` for JSON transport and persistence.
- Standard ISO 8601 / RFC 3339 timestamps for transaction provenance.
- Standard Result/Error envelope conventions (`code`, `data`, `error`).

### New Data Structures (`code/aiosh-rust/aiosh-core/src/package.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageFormat {
    Deb,
    Apk,
    Flatpak,
    Tarball,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageState {
    Available,
    Installed,
    Upgradable,
    PendingInstall,
    PendingRemoval,
    Broken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub version_constraint: Option<String>,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSpec {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub format: PackageFormat,
    pub state: PackageState,
    pub description: String,
    pub installed_size_bytes: u64,
    pub sha256: Option<String>,
    pub repository_url: Option<String>,
    pub dependencies: Vec<PackageDependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageActionType {
    Install,
    Remove,
    Upgrade,
    Purge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageAction {
    pub action: PackageActionType,
    pub package_name: String,
    pub target_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTransaction {
    pub id: String,
    pub created_at: String,
    pub actions: Vec<PackageAction>,
    pub dry_run: bool,
    pub total_size_delta_bytes: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageQuery {
    pub name_pattern: Option<String>,
    pub format: Option<PackageFormat>,
    pub state: Option<PackageState>,
    pub limit: Option<usize>,
}
```

## 2. Invariants & Validation Rules (PM1..PM5)

### `validate_package_name(name: &str) -> Result<(), String>`
1. Length between 1 and 128 bytes.
2. Must begin with lowercase ASCII alphanumeric `[a-z0-9]`.
3. Subsequent characters must be lowercase ASCII alphanumeric, `+`, `-`, or `.`.
4. No uppercase, control characters, whitespace, slashes, or null bytes.

### `validate_package_spec(spec: &PackageSpec) -> Result<(), Vec<String>>`
- **PM1 (Name Syntax)**: `validate_package_name(&spec.name)` must succeed.
- **PM2 (Bounds & Lengths)**:
  - `version.len()` in `1..=64`, non-empty, no control chars.
  - `architecture` in `{"x86_64", "aarch64", "riscv64", "all", "any"}`.
  - `description.len() <= 4096`.
  - `dependencies.len() <= 256`.
  - `installed_size_bytes <= 100 * 1024 * 1024 * 1024` (100 GiB ceiling).
- **PM3 (Dependency Hygiene)**:
  - No dependency can have the same name as the package itself (`dep.name != spec.name`).
  - No duplicate dependency names.
  - Each dependency name must satisfy `validate_package_name`.
- **PM4 (Checksum & Provenance)**:
  - If `sha256` is provided: must be exactly 64 hexadecimal characters `[0-9a-fA-F]`.
  - If `repository_url` is provided: must begin with `https://` (or `http://127.0.0.1` / `http://localhost` for local testing). No control characters.
- **PM5 (State Consistency)**:
  - `Installed` packages must have `installed_size_bytes > 0`.

### `validate_package_transaction(tx: &PackageTransaction) -> Result<(), Vec<String>>`
- `tx.id` must be non-empty, <= 64 chars, graphic ASCII.
- `tx.created_at` must parse as RFC 3339.
- `tx.actions` must not be empty and must not exceed 256 entries.
- No package can have multiple contradictory actions in the same transaction (e.g. `Install` and `Remove`).
- Each action's `package_name` must pass `validate_package_name`.

## 3. Error Envelope & Failure Modes
Validation failures return structured error vectors:
```json
{
  "code": 2,
  "data": null,
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Package specification violates invariants: ['package name contains invalid characters: MyPackage']"
  }
}
```

## 4. Audit Effects
State-altering operations that construct or execute `PackageTransaction` emit an immutable audit row into the SQLite WAL ring (`audit.db`) with:
- `action`: `package.transaction.plan` / `package.transaction.execute`
- `actor`: Caller context (CLI user or MCP agent ID)
- `input_digest`: SHA-256 of `PackageTransaction` JSON
- `status`: `Success` or `Failed`
