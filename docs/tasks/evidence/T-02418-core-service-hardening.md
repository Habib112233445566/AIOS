# T-02418: Sandbox Enforcement Core Service Hardening

## 1. Hardening Measures Implemented
Hardened the Sandbox Enforcement Core Service (`SandboxService`) against resource exhaustion, unbounded storage growth, and unhandled failure states:
1. **Profile Capacity Caps**:
   - Implemented `MAX_PROFILES_IN_SERVICE` capping custom profile registration at 256 entries.
   - Enforced `ERR_SANDBOX_CAPACITY_EXCEEDED` on any registration attempt beyond the quota.
2. **Output Stream Limiting**:
   - Capped captured stdout and stderr streams to `max_output_capture_bytes` (10 MiB default) to prevent daemon memory bloat or out-of-memory errors from infinite logging loops.
3. **Fail-Open Component Telemetry**:
   - Non-Linux platforms honestly emit `status="unavailable"` and `detail="unsupported on <platform>"` without breaking cross-platform parity or silently dropping audit rows.
4. **Clean Error Envelope**:
   - Subprocess execution failures (missing binary, execution error) return standard `SandboxExecutionResult` with exit code 127 and detailed status error, preventing panics and ensuring reliable auditing.

## 2. Invariant Verification
- Verified capacity limits through automated integration test `test_service_capacity_limit`.
- Confirmed zero memory leaks and 0 compiler warnings.
