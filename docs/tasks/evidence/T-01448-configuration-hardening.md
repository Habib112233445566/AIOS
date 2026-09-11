# T-01448: User Session Bootstrap — Configuration: Hardening

## Metadata
- **Task ID:** `T-01448`
- **Subsystem:** `code/aiosh-rust/aiosh-core`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`
- **Component:** User Session Bootstrap Configuration Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Hardening Measures Implemented

1. **Size Ceilings & Bounded Ingestion**:
   - Configuration file loading in `SessionConfig::from_file` enforces a hard ceiling of 64 KiB (`MAX_CONFIG_FILE_BYTES = 65_536`).
   - Sizing checked twice: first via `file.metadata()?.len()`, second via stream bounding with `file.take(MAX_CONFIG_FILE_BYTES + 1).read_to_end(&mut bytes)` preventing memory spike attacks.
   - Session store loading is capped at 100 MiB (`MAX_ALLOWED_STORE_SIZE_BYTES`).

2. **Strict Range Invariants**:
   - User capacity: bounded to $[1 \dots 128]$.
   - Store capacity: bounded to $[10 \dots 10,000]$.
   - Idle timeouts: bounded to $[10 \dots 86,400]$ seconds.
   - Store sizes: bounded to $[65,536 \dots 104,857,600]$ bytes.

3. **String Sanitization & Null-Byte Rejection**:
   - `store_path` and `config_path` arguments are strictly checked for length $\le 1024$ and rejection of ASCII control characters (`c.is_control()`) and null bytes (`\0`).

4. **Zero Silent Failure & Standard Envelopes**:
   - Every parsing, file I/O, or invariant validation error is explicitly captured and emitted in standardized envelopes (`{"code": 1|2, "data": null, "error": {"code": ..., "message": ...}}`).
   - Under no circumstances does the engine silently fall back or drop errors.

5. **Resource Cleanup & Atomic Persistence**:
   - Disk writes for session stores utilize PID + nanosecond temporary files with automatic removal on error paths.
   - Clean handling of file handles ensures no descriptor leaks.

6. **Honest Audit Emission**:
   - Every failure path, invalid argument error, and resolution event emits an audit row to SQLite WAL with non-repudiable monotonic ID.
