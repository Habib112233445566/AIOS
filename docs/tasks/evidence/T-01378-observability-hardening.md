# T-01378: Init & Service Supervision / Observability - Hardening

## Overview
Task `T-01378` hardens the Init & Service Supervision observability subsystem across path bounds checking, input sanitization, buffer ceilings, integer overflow protections, fail-open audit compliance, and structured error reporting.

---

## Hardening Protections & Implementation Details

1. **Path Boundary & Control Character Defense**:
   - `generate_from_paths` enforces length boundaries ($\le 1024$ characters) and explicitly rejects control characters (`c.is_control()`, such as null bytes `\0`, bell `\x07`, etc.) on both `store_path` and `policy_path`.
   - Both CLI (`aiosh service stats`) and MCP (`aios.service.stats`) enforce identical validation before performing filesystem calls.
   - Non-existent or inaccessible file paths return explicit, descriptive error strings instead of panicking.

2. **Resource & Allocation Ceilings**:
   - File reads delegate to `ServiceStore::load_from_path` and `ServiceSecurityPolicy::from_file`, which strictly enforce file size limits ($10\text{ MiB}$ and $64\text{ KiB}$ respectively).
   - Telemetry distribution maps use static histogram buckets (`"0"`, `"1-2"`, `"3-5"`, `"6+"`), ensuring bounded memory allocation regardless of service dependency graph complexity.

3. **Arithmetic Saturation Defense**:
   - Accumulation of service process restarts uses saturated arithmetic (`total_restarts.saturating_add(...)`), preventing integer overflow panics in environments with long service uptime or repeated flapping.

4. **Resource Cleanup & State Isolation**:
   - File handles opened during store and policy loading are scoped with automatic RAII closure upon exit.
   - Telemetry calculation is completely read-only; no temporary files, background threads, or child processes remain after generation.

5. **Structured Error Envelopes & Honest Audit Emission**:
   - All errors adhere to the standard result envelope:
     - CLI returns code 1 or 2 with structured JSON: `{"code": ..., "data": null, "error": {"code": "...", "message": "..."}}`.
     - MCP returns standard JSON-RPC 2.0 error payloads (`{"ok": false, "error": ...}`).
   - Failure paths unconditionally write an honest audit record to the SQLite WAL audit ring via `classify_and_emit` or `dispatch::recorded_call`, satisfying ADR-0035 §F-2.

---

## Verification & Test Evidence
```text
running 7 tests
test test_so1_inventory_completeness_and_empty_store ... ok
test test_so2_categorical_distributions ... ok
test test_so3_health_and_restart_telemetry ... ok
test test_so4_dependency_distribution_histogram ... ok
test test_so5_security_policy_compliance ... ok
test test_so6_serialization_and_string_helpers ... ok
test test_so7_hardening_and_path_boundaries ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
```
All criteria `SS1..SS8` pass in `tools/test_service_suites.py`.
