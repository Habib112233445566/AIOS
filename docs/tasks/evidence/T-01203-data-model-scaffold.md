# T-01203: Package Management - Data Model: Scaffold

## Metadata
- **Task ID:** `T-01203`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model Scaffold
- **Status:** Complete

## 1. Scaffold Deliverables
Created module skeleton `code/aiosh-rust/aiosh-core/src/package.rs` and registered within `aiosh-core` (`code/aiosh-rust/aiosh-core/src/lib.rs`).

### Defined Typed Interfaces & Data Structures:
- `PackageFormat`: Enum (`Deb`, `Apk`, `Flatpak`, `Tarball`) with snake_case Serde bindings.
- `PackageState`: Enum (`Available`, `Installed`, `Upgradable`, `PendingInstall`, `PendingRemoval`, `Broken`).
- `PackageDependency`: Struct (`name: String`, `version_constraint: Option<String>`, `optional: bool`).
- `PackageSpec`: Full metadata struct with bounds on sizes, architecture, repository, and checksum.
- `PackageActionType`: Enum (`Install`, `Remove`, `Upgrade`, `Purge`).
- `PackageAction`: Struct capturing individual actions within batch transactions.
- `PackageTransaction`: Container for multi-package atomic operations.
- `PackageQuery`: Filtering structure for package lookups.
- Typed function signatures with fail-loud `unimplemented!` stubs:
  - `validate_package_name(name: &str) -> Result<(), String>`
  - `validate_package_spec(spec: &PackageSpec) -> Result<(), Vec<String>>`
  - `validate_package_transaction(tx: &PackageTransaction) -> Result<(), Vec<String>>`

### Module Registration & Exports:
- Exported in `code/aiosh-rust/aiosh-core/src/lib.rs` under `pub mod package;` and public re-exports.
- Initial unit test stub `test_scaffold_types_instantiation` validating memory layout and Serde compatibility.
