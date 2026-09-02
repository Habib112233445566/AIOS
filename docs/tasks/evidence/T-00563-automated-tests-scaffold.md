# T-00563 — Evidence & Audit Trail / automated tests: Scaffold

## 1. Scaffold Scope
This task scaffolds the automated testing harness for Evidence & Audit Trail, including the CI validation tool `tools/check_evidence.py` and the Rust end-to-end integration test file `code/aiosh-rust/aiosh-core/tests/test_evidence_e2e.rs`.

## 2. Scaffold Contents
- Created `tools/check_evidence.py` with E1..E4 criteria checks.
- Created `code/aiosh-rust/aiosh-core/tests/test_evidence_e2e.rs` with `test_evidence_full_lifecycle_e2e`.

## 3. Test Verification
- `python tools/check_evidence.py` -> PASS (E1..E4).
- `cargo test --test test_evidence_e2e` -> PASS.
