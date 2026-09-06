# T-01318 — Init & Service Supervision / Core Service: Hardening

## 1. Hardening Deliverables
- **Store File Size Ceiling (`10 MiB`) & Entity Ceiling (`10,000`)**:
  - Enforced a hard 10 MiB (`10 * 1024 * 1024` bytes) ceiling on filesystem payload size in `ServiceStore::load_from_path` before reading into memory, guarding against memory exhaustion and decompression bombs.
  - Imposed a 10,000 maximum service entity count limit on deserialized stores to prevent unbounded memory allocation and CPU starvation.
- **Defensive Tempfile Cleanup & Atomic Persistence**:
  - `ServiceStore::save_to_path` writes to a PID-isolated temporary file (`<path>.tmp.<pid>`).
  - Implemented explicit error handling to un-link and remove incomplete temp files on any write or rename failure, preventing dangling files or filesystem leaks.
  - Applied atomic rename to ensure readers never observe partially written service states.
  - Applied strict file permissions (`0o644` on Unix) on the written store file.
- **Bounded Cycle Detection & Execution Time**:
  - In Kahn's topological sort (`plan_service_order`), cycle detection is handled iteratively without recursion. In-degree tracking halts immediately upon discovering an unresolved cycle and returns an explicit error (`invariant CS3 violated: cyclic dependency detected in service graph`).
  - Unit timeout parameters (`timeout_start_secs` and `timeout_stop_secs`) are constrained to $[1 \dots 86,400]$ seconds to prevent unkillable or hung service states.
- **Standardized Error Envelopes & Audit Guarantees**:
  - All operations return explicit `Result<T, String>` error envelopes with structured error messages (no silent failures, no panics, no unhandled unwraps).
  - Every CLI invocation records an honest audit row to `audit.log`.
  - Every MCP tool call executes via `dispatch::recorded_call`, writing SHA-256 hash-chained records to SQLite WAL audit storage for both success and error branches (satisfying fail-open audit requirement ADR-0035 §F-2).

## 2. Test Verification Output
```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```
