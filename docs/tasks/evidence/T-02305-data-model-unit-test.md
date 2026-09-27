# T-02305: Audit Chain Extensions Data Model Unit Test

## Overview
This task implements automated unit tests for the extended audit chain data model (`ExtendedAuditRow`, `AuditProvenance`, `AuditCausalLink`, `AuditSignature`) in `code/aiosh-rust/aiosh-core/tests/test_audit_chain_ext.rs`.

## Test Execution Evidence

```text
cargo test --test test_audit_chain_ext
   Compiling aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.75s
     Running tests\test_audit_chain_ext.rs (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\deps\test_audit_chain_ext-0ad6df571ad04c22.exe)

running 6 tests
test test_extended_audit_row_invalid_provenance_and_signatures ... ok
test test_extended_audit_row_json_serde_roundtrip ... ok
test test_extended_audit_row_legacy_compatibility ... ok
test test_extended_audit_row_bounds_enforcement ... ok
test test_extended_audit_row_tamper_detection ... ok
test test_extended_audit_row_with_provenance_and_causality ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## Coverage Summary
1. `test_extended_audit_row_legacy_compatibility`: Verifies that converting between legacy `AuditRow` and `ExtendedAuditRow` produces identical `hash_proto` dictionaries, identical SHA-256 hashes, and lossless roundtrip conversions.
2. `test_extended_audit_row_with_provenance_and_causality`: Verifies correct integration of `AuditProvenance` (session, grant ID, trace ID), `AuditCausalLink` (DAG parent hash), `AuditSignature` (Ed25519), and extension key-values.
3. `test_extended_audit_row_tamper_detection`: Verifies that modifying any field (e.g. `command`) invalidates the cryptographic hash and is rejected by `verify_hash()`.
4. `test_extended_audit_row_bounds_enforcement`: Asserts that rows exceeding the maximum allowed causal links (> 16) fail validation with `AUDIT_EXT_ERR_BOUNDS`.
5. `test_extended_audit_row_json_serde_roundtrip`: Verifies full JSON serialization and deserialization fidelity with preserved hash verification.
6. `test_extended_audit_row_invalid_provenance_and_signatures`: Asserts that provenance containing control characters or whitespace and invalid signatures fail validation.
