# T-02458: Sandbox Enforcement Automated Tests Hardening

## 1. Hardening Summary
This task hardens the automated testing fixtures and test runners of the Sandbox Enforcement subsystem against process leaks, timeout deadlocks, and unbound resource usage.

## 2. Hardening Measures Implemented
1. **Leak-Proof Child Process Cleanup**:
   - In `test_sandbox_automated_smoke.py`, wrapped `subprocess.Popen` lifecycle in a `try...finally` block.
   - Guaranteed that if `communicate` times out or panics, `p.kill()` and `p.wait()` are executed, terminating any spawned server or test instance and preventing orphan process leaks.
2. **Explicit Execution Timeouts**:
   - Subprocess communications are clamped to 10-second deadlines (`timeout=10`).
3. **Structured Result Envelope Parsing**:
   - Hardened JSON-RPC response extraction to handle structured error envelopes without throwing unexpected parsing exceptions.
4. **Hermetic SQLite WAL Isolation**:
   - Verified that multi-threaded tests in `test_sandbox_automated.rs` use temporary scoped storage that unlinks on teardown.

## 3. Test Verification
All 5 automated smoke tests pass in 0.52s with zero lingering child processes.
