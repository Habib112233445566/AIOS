# T-01262: Package Management - Security Policy: Specification

## Metadata
- **Task ID:** `T-01262`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Package Management / Security Policy
- **Status:** Complete
- **Date:** 2026-09-04

---

## 1. Scope & Objective
Specify the contract, types, and invariant rules for the Package Management Security Policy subsystem (`code/aiosh-rust/aiosh-core/src/package_policy.rs`).
The subsystem validates package specifications, transactions, and store registries against mandatory security rules before installation, upgrade, or rootfs deployment.

---

## 2. Data Types & Interfaces

### Policy Modes
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackagePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}
```

### Security Policy Specification
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageSecurityPolicy {
    pub mode: PackagePolicyMode,
    pub prohibited_packages: Vec<String>,
    pub allowed_architectures: Vec<String>,
    pub allowed_formats: Vec<PackageFormat>,
    pub require_checksum: bool,
    pub require_https_or_file_repo: bool,
    pub max_package_size_bytes: u64,
    pub max_dependencies_per_package: usize,
    pub allowed_repositories: Vec<String>,
}
```

### Violations & Verdicts
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackagePolicyViolation {
    pub rule_id: String,
    pub package_name: String,
    pub description: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackagePolicyVerdict {
    pub package_name: String,
    pub allowed: bool,
    pub mode: PackagePolicyMode,
    pub violations: Vec<PackagePolicyViolation>,
    pub evaluated_at: String,
}
```

---

## 3. Invariants (PP1..PP6)

### PP1: Policy Configuration Bounds
- `allowed_architectures`: Must not be empty, max 64 items.
- `allowed_formats`: Must not be empty.
- `prohibited_packages`: Maximum 1,024 entries. Each entry must be valid package name syntax.
- `max_package_size_bytes`: Bounded between $[10\text{ KiB} \dots 100\text{ GiB}]$.
- `max_dependencies_per_package`: Bounded between $[1 \dots 1024]$.

### PP2: Prohibited Package Rejection
- Evaluates `spec.name` against `prohibited_packages`.
- Default prohibited list includes unencrypted, insecure network legacy services: `telnet`, `rsh-client`, `rsh-server`, `rlogin`, `rexec`, `nis`, `yp-tools`.
- Generates violation `PP2-PROHIBITED-PACKAGE` (fatal = true).

### PP3: Cryptographic Integrity Validation
- When `require_checksum == true`, `spec.sha256` must be `Some(hash)` where `hash` is exactly 64 hexadecimal characters.
- Missing or malformed hash generates violation `PP3-MISSING-CHECKSUM` (fatal = true).

### PP4: Transport Protocol & Repository Whitelist
- When `require_https_or_file_repo == true`, `spec.repository_url` must start with `https://` or `file://`. Insecure `http://` or other schemes generate violation `PP4-INSECURE-TRANSPORT` (fatal = true).
- If `allowed_repositories` is non-empty, `spec.repository_url` must match a configured repository prefix; otherwise generates `PP4-UNAUTHORIZED-REPO`.

### PP5: Architecture & Format Conformance
- `spec.architecture` must exist in `allowed_architectures`. (Violation: `PP5-DISALLOWED-ARCH`).
- `spec.format` must exist in `allowed_formats`. (Violation: `PP5-DISALLOWED-FORMAT`).
- `spec.installed_size_bytes` must not exceed `max_package_size_bytes`. (Violation: `PP5-SIZE-EXCEEDED`).
- `spec.dependencies.len()` must not exceed `max_dependencies_per_package`. (Violation: `PP5-DEP-COUNT-EXCEEDED`).

### PP6: Mode Semantics & Audit Effects
- `Enforcing`: If any fatal violation exists, `verdict.allowed = false`.
- `Audit`: `verdict.allowed = true` even with fatal violations, but violations are reported.
- `Permissive`: Non-fatal violations are suppressed; fatal violations still reported.
- All evaluations write an immutable hash-chained audit row to SQLite WAL (`classify_and_emit`).

---

## 4. Reused vs. New Interfaces
- **Reused Interfaces**:
  - `PackageSpec`, `PackageFormat`, `PackageState`, `PackageDependency`, `PackageTransaction` (`package.rs`).
  - `PackageStore` (`package_service.rs`).
  - `audit` ring & `classifier` (`aiosh-core`).
- **New Interfaces**:
  - Module `code/aiosh-rust/aiosh-core/src/package_policy.rs`.
  - Methods: `evaluate_spec`, `evaluate_transaction`, `evaluate_store`, `from_file`, `from_env`, `resolve`.
