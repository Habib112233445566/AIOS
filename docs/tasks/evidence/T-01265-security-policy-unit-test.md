# T-01265: Package Management - Security Policy: Unit Test

## Metadata
- **Task ID:** `T-01265`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Package Management / Security Policy
- **Status:** Complete
- **Date:** 2026-09-04

---

## 1. Unit Test Deliverables
Implemented standalone integration and unit tests in `code/aiosh-rust/aiosh-core/tests/test_package_policy.rs` covering criteria `PP1..PP6`:
1. `test_pp1_policy_configuration_bounds_and_defaults`:
   - Validates default policy validity.
   - Negative assertions: empty architectures array, empty formats array, and invalid size bounds.
2. `test_pp2_prohibited_package_blocking`:
   - Positive assertion: legitimate utilities (`aiosh-tools`) pass without violations.
   - Negative assertions: prohibited utilities (`telnet`, `rsh-server`) produce fatal violation `PP2-PROHIBITED-PACKAGE` and are rejected.
3. `test_pp3_cryptographic_checksum_enforcement`:
   - Negative assertion: missing SHA-256 checksum produces fatal violation `PP3-MISSING-CHECKSUM`.
   - Negative assertion: malformed checksum (not 64 hex characters) produces `PP3-INVALID-CHECKSUM`.
4. `test_pp4_transport_protocol_and_repository_security`:
   - Negative assertion: plaintext `http://` produces fatal violation `PP4-INSECURE-TRANSPORT`.
   - Positive assertion: `https://` and `file://` pass.
5. `test_pp5_architecture_format_and_sizing_limits`:
   - Negative assertion: non-whitelisted architecture (`mips64el`) produces `PP5-DISALLOWED-ARCH`.
   - Negative assertion: package size exceeding policy ceiling produces `PP5-SIZE-EXCEEDED`.
6. `test_pp6_policy_modes_and_transaction_evaluation`:
   - Mode `Audit`: fatal violations tracked, but `allowed = true`.
   - Mode `Enforcing`: transaction with prohibited package rejected (`allowed = false`).
   - File roundtrip: serialization and deserialization from disk with size limits.

---

## 2. Test Execution Output
```
running 6 tests
test test_pp2_prohibited_package_blocking ... ok
test test_pp3_cryptographic_checksum_enforcement ... ok
test test_pp4_transport_protocol_and_repository_security ... ok
test test_pp1_policy_configuration_bounds_and_defaults ... ok
test test_pp5_architecture_format_and_sizing_limits ... ok
test test_pp6_policy_modes_and_transaction_evaluation ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```
All assertions passed in standalone execution.
