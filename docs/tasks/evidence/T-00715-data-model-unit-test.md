# T-00715 — Secrets & Access Hygiene / data model: Unit Test

## 1. Test Scope
This task verifies unit tests for the Secrets & Access Hygiene data model in `code/aiosh-rust/aiosh-core/src/secrets.rs` covering:
- Redaction boundary behavior (`test_redact_secret_value`).
- Clean scan report initialization and invariant assertion (`test_secret_scan_report_clean`).
- Multi-finding scan report severity calculation (`test_secret_scan_report_with_findings`).
- Validation failure on invariant corruption (`test_validate_secret_report_invalid`).
- Full JSON serialization and deserialization roundtrip (`test_secret_scan_report_serde_roundtrip`).

## 2. Test Verification Output
```text
running 5 tests
test secrets::tests::test_redact_secret_value ... ok
test secrets::tests::test_secret_scan_report_clean ... ok
test secrets::tests::test_secret_scan_report_with_findings ... ok
test secrets::tests::test_validate_secret_report_invalid ... ok
test secrets::tests::test_secret_scan_report_serde_roundtrip ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 176 filtered out; finished in 0.00s
```
