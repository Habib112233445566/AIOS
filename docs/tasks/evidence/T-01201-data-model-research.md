# T-01201: Package Management - Data Model: Research

## Metadata
- **Task ID:** `T-01201`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model
- **Status:** Complete

## 1. Executive Overview & Prior Art
Package management in AIOS provides a secure, deterministic, and audited abstraction layer across underlying Linux package systems (specifically Debian's APT/dpkg and Alpine's APK), enabling both autonomous AI agents and human operators to safely query, plan, install, remove, and verify system packages.

### Existing Substrate Context
- `code/aiosh-rust/aiosh-core/src/distro.rs`: Defines `DistroFamily` (`Debian`, `Alpine`), package manager bindings, and distro-specific default packages.
- `code/aiosh-rust/aiosh-core/src/distro_policy.rs`: Establishes security baseline policies requiring HTTPS package repositories and mandatory cryptographic signatures (`require_signed_packages: true`, `require_https_repos: true`).
- `code/aiosh-rust/aiosh-core/src/base_image.rs` & `base_image_policy.rs`: Governs package inclusion in rootfs images, enforcing package name validation (`^[a-z0-9][a-z0-9+.-]*$`), prohibited package blacklists (e.g., `telnet`, `rsh-client`), and size budgets.

## 2. Authoritative Sources & Upstream Standards
1. **Debian Policy Manual §5.6 (Package Control Files & Naming)**:
   - Package name specification: lowercase alphanumeric plus `+`, `-`, `.`, beginning with alphanumeric, minimum 2 characters in Debian proper.
   - Version syntax: `[epoch:]upstream_version[-debian_revision]` with strictly ordered component comparison.
   - Relationship fields: `Depends`, `Recommends`, `Suggests`, `Conflicts`, `Provides`, `Replaces`.
2. **Alpine Linux APK (apk-tools v2 / v3 Reference Manual)**:
   - Package specification: `<name>-<version>-r<revision>.apk`.
   - Signed package indices: `APKINDEX.tar.gz` with detached cryptographic signatures.
   - Musl and Busybox integrated dependency tracking.
3. **NIST SP 800-161 (Supply Chain Risk Management)**:
   - Cryptographic verification of software components (SHA-256 / Ed25519 / GPG).
   - Immutability and audit logging of software inventory modifications.

## 3. Facts vs. Assumptions

| Fact | Assumption |
|---|---|
| AIOS supports both Debian 12 (APT) and Alpine 3.19 (APK) as first-class base distributions. | A unified data model can cleanly abstract both Debian and Alpine package metadata without losing fidelity. |
| Cryptographic signature and HTTPS repository enforcement are mandated by AIOS security policy. | Autonomous agents will invoke package operations via structured MCP tool calls (`aios.package.*`). |
| Uncontrolled package modifications risk compromising host integrity and breaking reproducibility. | Package transactions must be planned and dry-run before execution, emitting audit records to SQLite WAL. |
| In-tree Rust modules follow strict typed models with Serde serialization and explicit error envelopes. | The in-memory data model can be validated with zero external runtime dependencies. |

## 4. Proposed Data Model & Invariants

### Core Types:
1. `PackageFormat`: Enum (`Deb`, `Apk`, `Flatpak`, `Tarball`).
2. `PackageState`: Enum (`Available`, `Installed`, `Upgradable`, `PendingInstall`, `PendingRemoval`, `Broken`).
3. `PackageDependency`: Struct (`name: String`, `version_constraint: Option<String>`, `optional: bool`).
4. `PackageSpec`: Struct identifying a package:
   - `name: String` (validated syntax)
   - `version: String` (SemVer or distro-specific version)
   - `architecture: String` (`x86_64`, `aarch64`, `riscv64`, `all`)
   - `format: PackageFormat`
   - `state: PackageState`
   - `description: String`
   - `installed_size_bytes: u64`
   - `sha256: Option<String>`
   - `repository_url: Option<String>`
   - `dependencies: Vec<PackageDependency>`
5. `PackageQuery`: Struct for searching/filtering packages (name pattern, state filter, format filter).
6. `PackageTransaction`: Struct for batch operations (actions: `Install`, `Remove`, `Upgrade`, target packages, total size delta, dry_run flag).

### Invariants (PM1..PM5):
- **`PM1` (Naming Syntax)**: Package name must match `^[a-z0-9][a-z0-9+.-]*$`, length between 1 and 128 chars, no control characters, null bytes, or path delimiters.
- **`PM2` (Bounds & Limits)**: Version length <= 64 chars, description <= 4096 bytes, dependencies count <= 256, package size <= 100 GiB.
- **`PM3` (Dependency Hygiene)**: A package cannot depend on itself (`dep.name != package.name`); duplicate dependencies prohibited.
- **`PM4` (Integrity & Origin)**: SHA-256 checksum (if present) must be 64 hexadecimal characters; repository URLs must use HTTPS.
- **`PM5` (State Consistency)**: Installed packages must have valid size >= 0; transaction actions must not contain contradictory operations on the same package.

## 5. Decisions & Unknowns
1. **Decision**: Implement `code/aiosh-rust/aiosh-core/src/package.rs` in `aiosh-core` containing `PackageSpec`, `PackageDependency`, `PackageFormat`, `PackageState`, `PackageQuery`, `PackageTransaction`, and `validate_package_spec`.
2. **Decision**: Enforce validation invariants `PM1..PM5` in standalone unit tests before integrating into CLI and MCP servers.
3. **Unknown / Future**: Real-time integration with live host `apt` and `apk` binaries will be handled in execution wrappers; the data model and store will provide the canonical representation and plan synthesis.
