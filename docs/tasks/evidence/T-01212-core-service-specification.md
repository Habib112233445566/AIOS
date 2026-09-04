# T-01212: Package Management - Core Service: Specification

## Metadata
- **Task ID:** `T-01212`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package_service`
- **Component:** Package Management Core Service Specification
- **Status:** Complete

## 1. Scope & Core Contracts
This specification defines the registry, query engine, dependency closure verification, transaction planning, and persistence mechanisms for the AIOS Package Management Core Service.

### Reused Interfaces
- Types from `aiosh_core::package`: `PackageSpec`, `PackageFormat`, `PackageState`, `PackageDependency`, `PackageAction`, `PackageActionType`, `PackageTransaction`, `PackageQuery`, and validators `validate_package_spec`, `validate_package_transaction`.
- Standard library: `std::collections::HashMap`, `std::path::Path`.

### New Data Structures (`code/aiosh-rust/aiosh-core/src/package_service.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionReport {
    pub transaction_id: String,
    pub packages_installed: Vec<String>,
    pub packages_removed: Vec<String>,
    pub packages_upgraded: Vec<String>,
    pub total_size_delta_bytes: i64,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageStore {
    pub packages: HashMap<String, PackageSpec>,
}
```

### Method Contracts on `PackageStore`:
1. `new() -> Self`: Initializes store seeded with canonical reference packages for Debian and Alpine.
2. `empty() -> Self`: Initializes an empty store.
3. `list_packages(&self) -> Vec<&PackageSpec>`: Returns all packages sorted by name.
4. `get_package(&self, name: &str) -> Option<&PackageSpec>`: Exact lookup by package name.
5. `register_package(&mut self, spec: PackageSpec) -> Result<(), String>`:
   - Validates `spec` with `validate_package_spec`.
   - Rejects if name already exists.
6. `unregister_package(&mut self, name: &str) -> Result<PackageSpec, String>`: Removes and returns existing package, or returns error if not found.
7. `query(&self, query: &PackageQuery) -> Vec<&PackageSpec>`:
   - Filters packages by name pattern (case-insensitive substring or glob), format, and state.
   - Applies optional `limit`.
8. `plan_transaction(&self, actions: Vec<PackageAction>, dry_run: bool) -> Result<PackageTransaction, String>`:
   - Checks that all action target packages exist in the store.
   - Validates dependency closure: for each package being installed, all required dependencies must already be `Installed` or present in the `actions` batch.
   - Calculates `total_size_delta_bytes`: adds size of newly installed packages, subtracts size of removed packages.
   - Emits validated `PackageTransaction`.
9. `execute_transaction(&mut self, tx: &PackageTransaction) -> Result<TransactionReport, String>`:
   - Validates transaction against invariants.
   - If `tx.dry_run`: evaluates state changes in-memory and returns report without modifying store.
   - If not `dry_run`: updates store package states (`Installed` / `Available`) and returns report.
10. `save_to_path(&self, path: &Path) -> Result<(), String>`:
    - Writes atomic temporary file with `0o644` permissions and renames over destination.
11. `load_from_path(path: &Path) -> Result<PackageStore, String>`:
    - Enforces 10 MiB (`10,485,760` bytes) ceiling.
    - Validates all package specs inside loaded store.

## 2. Invariants (CS1..CS5)
- **`CS1` (Registry Uniqueness)**: Every entry in `packages` has a unique key matching `spec.name`.
- **`CS2` (Transaction Determinism)**: `plan_transaction` called on identical inputs with unchanged store state produces byte-identical transaction plans.
- **`CS3` (Dependency Closure)**: Install actions require that every dependency either already exists in state `Installed` or is included as an `Install` action in the transaction.
- **`CS4` (Size Delta Arithmetic)**:
  $$\Delta_{\text{size}} = \sum_{p \in \text{Installed}} \text{size}(p) - \sum_{p \in \text{Removed}} \text{size}(p)$$
- **`CS5` (Persistence Atomicity & Limit)**: Writes are atomic via fsync and rename; file reads strictly reject files > 10 MiB.

## 3. Failure Envelopes & Audit Effects
- Operational failures return standard error messages (`Result<T, String>`).
- State-changing transaction executions emit audit records with caller provenance and transaction digest.
