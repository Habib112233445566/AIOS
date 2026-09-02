# T-00558 — Evidence & Audit Trail / configuration: Hardening

## 1. Hardening Scope
This task hardens the configuration subsystem for Evidence & Audit Trail against memory exhaustion, malformed JSON, and filesystem resource leaks.

## 2. Hardening Measures
1. **Bounded I/O (`MAX_CONFIG_BYTES`)**:
   - `EvidenceConfig::from_path` inspects metadata file size prior to reading, rejecting any configuration file > 64 KiB.
2. **Deterministic Fallbacks**:
   - `EvidenceConfig::from_env` falls back safely to default in-memory configuration if environmental files are absent or unreadable.
3. **Strict Path Invariants**:
   - `EvidenceConfig::validate` rejects non-relative paths, colon characters, and `..` parent directory traversal tokens.
4. **Clean Error Reporting**:
   - Deserialization and validation errors provide specific diagnostic messages including the invalid path and violated invariant.

## 3. Test Verification
- `cargo test -p aiosh-core evidence_config::tests` -> 8/8 tests pass.
