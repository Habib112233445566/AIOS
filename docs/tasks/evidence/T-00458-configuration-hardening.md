# T-00458 — Documentation Index Control / configuration: Hardening

## 1. Hardening Scope
This task documents and verifies the defensive limits and resilience mechanisms of `DocIndexConfig` in `code/aiosh-rust/aiosh-core/src/doc_index_config.rs`.

## 2. Hardening Invariants
1. **64 KiB Read Bound (`MAX_CONFIG_BYTES`)**:
   - Enforced on all file read operations via `Read::take()`, preventing resource exhaustion on large/infinite streams.
2. **Root Directory Cap (50 Max)**:
   - Prevents memory spikes from deeply nested or unbounded directory configurations.
3. **Strict Path Sanitization**:
   - Disallows `..` in all directory entries.
4. **Descriptive Error Reporting**:
   - Clear, deterministic error messages for missing files, invalid JSON syntax, empty fields, and boundary violations.

## 3. Verification
- All negative test cases in unit test and CLI smoke test suites pass with expected error status codes.
