# T-00595 — Evidence & Audit Trail / documentation: Unit Test

## 1. Unit Test Scope
This task implements and executes unit tests for `format_evidence_summary` in `code/aiosh-rust/aiosh-core/src/evidence_service.rs`.

## 2. Test Cases & Coverage
1. **Multi-Record Populated Manifest**:
   - Asserts header rendering, epic name, ISO 8601 timestamp, and individual records with task ID, step, relative path, short 8-char hash, and pass status.
2. **Short Hash Boundary**:
   - Asserts hashes shorter than 8 characters are rendered safely without out-of-bounds slicing panic.
3. **Empty Manifest Boundary**:
   - Asserts empty manifests render `(no evidence records)` without panics.

## 3. Test Verification Output
```text
running 1 test
test evidence_service::tests::test_format_evidence_summary ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 153 filtered out; finished in 0.00s
```
