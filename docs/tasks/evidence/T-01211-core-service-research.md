# T-01211: Package Management - Core Service: Research

## Metadata
- **Task ID:** `T-01211`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package_service`
- **Component:** Package Management Core Service Research
- **Status:** Complete

## 1. Executive Overview & Prior Art
The Core Service of Package Management provides the stateful management engine, registry store, query execution, transaction planner, and persistence subsystem for software packages in AIOS.

### Prior Substrate Patterns
- `code/aiosh-rust/aiosh-core/src/base_image_service.rs`: `ImageStore` maintaining canonical manifests, deterministic build plan synthesis (`generate_build_plan`), atomic disk persistence, and 10 MiB file limits.
- `code/aiosh-rust/aiosh-core/src/distro_service.rs`: `DistroStore` with multi-criteria scoring, evaluation caching, and disk persistence.
- `code/aiosh-rust/aiosh-core/src/package.rs`: Data models (`PackageSpec`, `PackageTransaction`, `PackageQuery`) and validation invariants `PM1..PM5`.

## 2. Authoritative Sources & Upstream Concepts
1. **Debian APT / libapt-pkg Dependency Resolver (EDSP - External Dependency Solver Protocol)**:
   - SAT-solver based resolution for dependency satisfaction and conflict prevention.
   - Size delta tracking: calculating archive download sizes and installed disk usage deltas.
   - Lock handling: `/var/lib/dpkg/lock` and atomic state commits.
2. **Alpine apk-tools World & Commit Engine**:
   - `/etc/apk/world` declarative package manifest.
   - Atomic simulation (dry run / `--simulate`) calculating exact disk block deltas.
3. **Reproducible Builds & Transaction Invariants**:
   - Transactions must be deterministic: same actions + same store state $\to$ identical size deltas and execution plans.
   - State rollback capability in the event of partial execution failure.

## 3. Facts vs. Assumptions

| Fact | Assumption |
|---|---|
| Package operations must be dry-runnable so autonomous agents can inspect size deltas and dependencies before mutating the system. | A deterministic transaction planner in `PackageStore` will provide accurate delta calculations without requiring active root privileges. |
| In-tree services in `aiosh-core` persist state as canonical JSON with atomic renames and file bounds. | Storing the canonical package catalog in `/var/lib/aios/packages/package_store.json` provides an auditable offline cache. |
| Dependency cycles or missing packages break package management transactions. | Checking dependency closure during transaction planning prevents invalid transaction states. |

## 4. Proposed Core Service Architecture (`package_service.rs`)

### Core Structures:
1. `PackageStore`:
   - `packages: HashMap<String, PackageSpec>`
   - `store_path: Option<PathBuf>`
2. `TransactionReport`:
   - `transaction_id: String`
   - `packages_installed: Vec<String>`
   - `packages_removed: Vec<String>`
   - `total_size_delta_bytes: i64`
   - `timestamp: String`

### Service Invariants (CS1..CS5):
- **`CS1` (Registry Uniqueness)**: Every package in the store has a unique identifier conforming to PM1 syntax.
- **`CS2` (Transaction Determinism)**: Repeated planning of identical action sets produces identical transaction plans and size deltas.
- **`CS3` (Dependency Closure)**: Install actions verify that all mandatory dependencies are either already installed or included in the same transaction.
- **`CS4` (Size Delta Arithmetic)**: `total_size_delta_bytes == sum(installed_sizes) - sum(removed_sizes)`.
- **`CS5` (Persistence Atomicity & Limits)**: Saves use temporary file rename with `0o644` permissions; loads enforce a 10 MiB file size ceiling.

## 5. Decisions Needed Before Implementation
1. **Module Name**: `code/aiosh-rust/aiosh-core/src/package_service.rs`.
2. **Default Store**: Seed with canonical reference packages across Debian (`curl`, `libc6`, `libssl3`, `bash`, `coreutils`, `openssh-server`, `python3.11`) and Alpine (`musl`, `busybox`, `apk-tools`, `neovim`).
3. **Transaction Execution**: In-memory state transition with full report generation; host exec hooks mapped to subsequent integration tasks.
