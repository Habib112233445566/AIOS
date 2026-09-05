# T-01248: Package Management - Configuration: Hardening

## Metadata
- **Task ID:** `T-01248`
- **Subsystem:** `code/aiosh-rust`
- **Component:** Package Management Configuration Subsystem Hardening
- **Status:** Complete

## 1. Hardening Architecture & Implementations

### A. Strict Size Caps & Resource Limits
1. **Configuration File Ceiling (`PC6`)**:
   - Files read from disk are capped at `MAX_CONFIG_FILE_BYTES` (65,536 bytes / 64 KiB).
   - Ingestion inspects `file.metadata()?.len()` before reading into memory.
   - Stream reading utilizes `file.take(MAX_CONFIG_FILE_BYTES + 1).read_to_end(&mut bytes)` to prevent decompression or streaming memory attacks.
2. **Filesystem Path Bounds (`PC1`)**:
   - `store_path` and `config_path` parameters are constrained to $\le 1024$ bytes.
   - ASCII control characters and null bytes (`\0`) are immediately rejected.
3. **Store Sizing & Entity Bounds (`PC2`, `PC3`)**:
   - `max_store_size_bytes` is validated within $[65,536 \text{ bytes (64 KiB)} \dots 104,857,600 \text{ bytes (100 MiB)}]$.
   - `max_entity_count` is validated within $[10 \dots 100,000]$.

### B. Standard Result Envelopes & Error Visibility
1. **Operator CLI Surface**:
   - All errors return structured JSON envelopes with explicit error codes:
     - `INVALID_ARGUMENT`: Parameter length or control character violation.
     - `CONFIG_RESOLUTION_FAILED`: File access error, JSON syntax error, or invariant violation (`PC1..PC6`).
   - CLI exits with non-zero status code (`1` for operational errors, `2` for validation/argument errors), never failing silently.
2. **Autonomous Agent MCP Surface**:
   - Routed via `dispatch::recorded_call`, returning standard error envelope:
     ```json
     {
       "ok": false,
       "tool": "aios.package.config",
       "reason": "<error_message>",
       "audit_id": <id>
     }
     ```

### C. Resource Cleanup & Leak Prevention
- File descriptors opened via `std::fs::File::open` are wrapped in RAII constructs and drop cleanly upon function exit or error return.
- No child processes, background threads, or long-lived network sockets are spawned.
- All temporary test files are cleaned up via `std::fs::remove_file`.

### D. Honest Audit Emission on Failure (ADR-0035 §F-2)
- Any failure during configuration resolution writes an honest audit row to the SQLite WAL `AuditRing` detailing the error reason and target path.

---

## 2. Test Verification
All failure modes, boundary limits, and error paths are verified by:
- `test_package_config_pc1_store_path_invariants` (path length, control chars, null bytes)
- `test_package_config_pc2_store_size_invariants` (sub-minimum, super-maximum, boundaries)
- `test_package_config_pc3_entity_count_invariants` (sub-minimum, super-maximum, boundaries)
- `test_package_config_pc4_repository_security` (protocol enforcement)
- `test_package_config_pc6_file_roundtrip_and_size_cap` (64 KiB file cap enforcement)
- Criterion `PM5` in `tools/test_package_suites.py`.
