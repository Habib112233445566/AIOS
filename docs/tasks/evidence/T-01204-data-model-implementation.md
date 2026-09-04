# T-01204: Package Management - Data Model: Implementation

## Metadata
- **Task ID:** `T-01204`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model Implementation
- **Status:** Complete

## 1. Implemented Functions & Invariant Enforcement
Implemented complete data model and validation functions in `code/aiosh-rust/aiosh-core/src/package.rs`:

1. **`validate_package_name(name: &str) -> Result<(), String>`**:
   - Strict Debian & Alpine syntax checking: starts with lowercase alphanumeric `[a-z0-9]`, followed by `[a-z0-9+.-]`.
   - Length checked within `1..=128`.
   - Explicit rejection of uppercase letters, whitespace, slashes, null bytes, and control characters.

2. **`validate_package_spec(spec: &PackageSpec) -> Result<(), Vec<String>>`**:
   - **PM1**: Syntax checked via `validate_package_name`.
   - **PM2**: Bounds checks for version (`1..=64`), architecture (printable graphic ASCII), description (`<= 4096` bytes), dependencies count (`<= 256`), and size ceiling (`100 GiB`).
   - **PM3**: Dependency hygiene: self-dependency rejection (`dep.name != spec.name`), duplicate dependency detection, and version constraint bounds.
   - **PM4**: SHA-256 verification (exact 64 hexadecimal characters) and repository URL protocol enforcement (mandatory HTTPS, with local testing loopback allowed).
   - **PM5**: State consistency: installed packages must possess positive `installed_size_bytes > 0`.

3. **`validate_package_transaction(tx: &PackageTransaction) -> Result<(), Vec<String>>`**:
   - Transaction ID graphic ASCII validation and length caps (`<= 64`).
   - RFC 3339 timestamp verification via `chrono::DateTime::parse_from_rfc3339`.
   - Actions bounds (`1..=256`).
   - Conflicting action detection (preventing multiple operations targeted at the same package within a single transaction).
   - Target version length and character validation.

## 2. In-Tree Unit Tests
- `test_validate_package_name`: Positive cases (`curl`, `libc6`, `libssl3`, `g++`, `python3.11`) and negative cases (uppercase, leading symbols, spaces, slashes, null bytes, empty strings, oversized strings).
- `test_validate_package_spec_valid`: Complete canonical Debian/Alpine package specification.
- `test_validate_package_spec_pm1_to_pm5_rejections`: Self-dependency, insecure HTTP URLs, zero-size installed packages, and malformed checksums.
- `test_validate_package_transaction`: Valid transactions, conflicting package actions, and malformed timestamps.
