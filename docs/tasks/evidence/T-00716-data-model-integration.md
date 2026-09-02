# T-00716 — Secrets & Access Hygiene / data model: Integration

## 1. Integration Scope
This task integrates the `secrets` data model into the core library and initializes the standalone automated test runner `tools/test_secrets_suites.py` validating criteria `K1`.

## 2. Integration Deliverables
- Registered `pub mod secrets;` into `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Created standalone test harness `tools/test_secrets_suites.py` to evaluate secrets hygiene criteria (`K1..K7`).
- Verified `test_k1_data_model_integrity` asserting data model serialization and invariant checks.

## 3. Test Verification Output
```text
[+] K1 data model integrity

PASS: secrets_suites criteria (K1)
```
