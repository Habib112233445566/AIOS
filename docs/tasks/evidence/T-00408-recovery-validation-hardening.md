# T-00408 — Dependency & Toolchain Pinning / recovery & validation: Hardening

## 1. Hardening Scope
This task hardens the recovery and validation functionality of Dependency & Toolchain Pinning against resource exhaustion, malformed payloads, subprocess hangs, and memory leakage.

## 2. Hardening Measures
1. **Bounded Manifest File Reading**:
   - Manifest validation reads strictly at most 64KB (`take(65_536)`) from disk, preventing memory exhaustion and denial-of-service from massive configuration files.
2. **Subprocess Timeout and Process Reaping**:
   - Telemetry collection during reconciliation uses bounded 15-second timeouts with automatic child process termination and reap (`child.kill()`).
3. **Telemetry Output Clamping**:
   - Captured stdout/stderr streams from compiler/runtime binaries are clamped to 512 bytes with `[TRUNCATED]` markers to prevent audit ring and memory buffer inflation.
4. **Zero Silent Failures**:
   - Missing tools, version mismatches, and JSON syntax errors return explicit `Err(String)` envelopes with descriptive diagnostics.

## 3. Acceptance Verification
- All failure modes produce structured, explicit error envelopes.
- No memory leaks, dangling subprocesses, or infinite loops exist on error paths.
