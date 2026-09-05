# T-01268: Package Management / Security Policy - Hardening

## Overview
Task `T-01268` hardens the package security policy subsystem across bounds checking, file I/O boundaries, path traversal protections, stream limiting, and error propagation:
1. **File I/O Boundary Defense**:
   - `from_file` path bounds checking: restricts path strings to <= 1024 characters and rejects ASCII control characters.
   - Strict size ceiling: enforced via `file.take(MAX_POLICY_FILE_BYTES + 1)` ensuring that files larger than 64 KiB (`MAX_POLICY_FILE_BYTES = 65_536`) cannot cause unbounded buffer allocations or memory exhaustion.
2. **Repository Whitelist Hardening**:
   - `allowed_repositories` bounded to a maximum of 256 entries.
   - Each entry bounded to <= 1024 characters with control character validation.
   - Invariant PP4 validation guarantees all repository URLs start with either `https://` or `file://`.
3. **Architecture & Prohibited Package Validation**:
   - Architectures strictly capped at 64 entries, max 32 characters per architecture string, no control characters.
   - Prohibited package list strictly capped at 1024 entries, max 128 characters per entry, no whitespace or control characters.
   - Package sizes constrained to valid domain $[10\text{ KiB} \dots 100\text{ GiB}]$.
   - Dependencies per package constrained to $[1 \dots 1024]$.
4. **Fail-Closed Semantic Enforcement**:
   - In `Enforcing` mode, any fatal violation (`fatal: true`) forces `allowed = false`.
   - Missing or malformed SHA-256 digests (not matching 64 hex characters) unconditionally fail as fatal violations.
   - Insecure transport (`http://`) unconditionally fails as a fatal violation.
5. **Comprehensive Hardening Unit Suite**:
   - Added `test_pp7_hardening_and_boundary_checks` in `code/aiosh-rust/aiosh-core/tests/test_package_policy.rs` verifying control character paths, oversized configuration files, and invalid repository prefixes.

## Test Verification Output
```
running 7 tests
test test_pp3_cryptographic_checksum_enforcement ... ok
test test_pp4_transport_protocol_and_repository_security ... ok
test test_pp2_prohibited_package_blocking ... ok
test test_pp1_policy_configuration_bounds_and_defaults ... ok
test test_pp5_architecture_format_and_sizing_limits ... ok
test test_pp6_policy_modes_and_transaction_evaluation ... ok
test test_pp7_hardening_and_boundary_checks ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```
All criteria PM1..PM7 verified passing in `tools/test_package_suites.py`.
