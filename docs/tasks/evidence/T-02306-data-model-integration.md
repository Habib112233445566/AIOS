# T-02306: Audit Chain Extensions Data Model Integration

## Overview
This task integrates the extended audit chain data model (`ExtendedAuditRow`, `AuditProvenance`, `AuditCausalLink`, `AuditSignature`) with SQLite persistence in `AuditRing`, supporting both legacy and extended rows via schema migration, `write_extended`, `tail_extended`, and extended verification in `aiosh-core`.

## Integration Summary
1. **Schema Migration**: Added `EXTENSION_COLUMNS` (`provenance_json`, `causal_links_json`, `signature_json`, `extensions_json`) to `audit.rs` and integrated idempotent schema migrations into `AuditRing::ensure_schema()`.
2. **Ring Operations**:
   - Implemented `ExtendedAuditRowInput` encapsulating base row attributes, provenance, DAG causal links, digital signatures, and extension payload dictionaries.
   - Implemented `AuditRing::write_extended` computing SHA-256 hash chaining over extended canonical JSON prototypes and storing JSON fields.
   - Implemented `row_to_extended_audit` and `AuditRing::tail_extended` to read extended rows with full backward compatibility for unextended legacy records.
   - Updated `AuditRing::verify` to verify both unextended and extended audit records without breaking historical chain hashes.
3. **Automated Integration Tests**: Added `test_audit_ring_extended_integration` in `tests/test_audit_chain_ext.rs` verifying sequential writes of legacy and extended records, tailing, and end-to-end cryptographic hash verification.

## Test Execution Evidence

```text
cargo test --test test_audit_chain_ext
   Compiling aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 28.60s
     Running tests\test_audit_chain_ext.rs (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\deps\test_audit_chain_ext-0ad6df571ad04c22.exe)

running 7 tests
test test_extended_audit_row_bounds_enforcement ... ok
test test_extended_audit_row_invalid_provenance_and_signatures ... ok
test test_extended_audit_row_json_serde_roundtrip ... ok
test test_audit_ring_extended_integration ... ok
test test_extended_audit_row_legacy_compatibility ... ok
test test_extended_audit_row_tamper_detection ... ok
test test_extended_audit_row_with_provenance_and_causality ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```
