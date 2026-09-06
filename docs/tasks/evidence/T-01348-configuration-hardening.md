# T-01348: Init & Service Supervision - Configuration: Hardening

## Metadata
- **Task ID:** `T-01348`
- **Subsystem:** `code/aiosh-rust`
- **Component:** Init & Service Supervision Configuration Subsystem Hardening
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Hardening Architecture & Implementations

### A. Strict Size Caps & Resource Limits
1. **Configuration File Ceiling (`SC7`)**:
   - Files read from disk are capped at `MAX_CONFIG_FILE_BYTES` (65,536 bytes / 64 KiB).
   - Ingestion checks `file.metadata()?.len()` before reading into memory.
   - Stream reading utilizes `file.take(MAX_CONFIG_FILE_BYTES + 1).read_to_end(&mut bytes)` to prevent decompression bombs or memory bloat.
2. **Filesystem Path Bounds (`SC1`)**:
   - `store_path` and `config_path` parameters are constrained to $\le 1024$ bytes.
   - ASCII control characters and null bytes (`\0`) are immediately rejected.
3. **Execution Timeout Bounds (`SC2`)**:
   - `default_timeout_start_secs` and `default_timeout_stop_secs` are validated within $[1 \dots 3,600]$ seconds.
   - Rejects zero-second instant timeouts and multi-day thread lockups.
4. **Store Sizing & Entity Bounds (`SC3`, `SC4`)**:
   - `max_store_size_bytes` is validated within $[65,536 \text{ bytes (64 KiB)} \dots 104,857,600 \text{ bytes (100 MiB)}]$.
   - `max_entity_count` is validated within $[10 \dots 100,000]$.
5. **Restart Throttling Bounds (`SC5`)**:
   - `restart_backoff_secs` is validated within $[1 \dots 300]$ seconds.
   - `max_restart_burst` is validated within $[1 \dots 50]$ attempts.

### B. Standard Result Envelopes & Error Visibility
1. **Operator CLI Surface**:
   - All errors return structured JSON envelopes with explicit error codes:
     - `INVALID_ARGUMENT`: Parameter length or control character violation (exit code 2).
     - `CONFIG_RESOLUTION_FAILED`: File access error, JSON syntax error, or invariant violation (`SC1..SC7`) (exit code 1).
   - CLI never fails silently.
2. **Autonomous Agent MCP Surface**:
   - Routed via `dispatch::recorded_call`, returning standard error envelope:
     ```json
     {
       "ok": false,
       "tool": "aios.service.config",
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
- `test_service_config_sc1_store_path_invariants` (path length, control chars, null bytes)
- `test_service_config_sc2_timeout_invariants` (sub-minimum, super-maximum, boundaries)
- `test_service_config_sc3_sc4_sc5_boundary_invariants` (store size, entity count, restart throttling)
- `test_service_config_sc6_env_resolution` (precedence and environment variable fallback)
- `test_service_config_sc7_file_roundtrip_and_size_cap` (64 KiB file cap enforcement)
- `test_mcp_service_tools` (MCP surface error handling and invalid config detection)
