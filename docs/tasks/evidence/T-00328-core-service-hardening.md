# T-00328 — Dependency & Toolchain Pinning: core service Hardening

## 1. Overview
This task hardens the `toolchain_service` core module to protect against failure modes involving external process execution. 

## 2. Hardening Measures Implemented
- **Timeouts**: Added a `run_with_timeout` helper that wraps `Command::spawn` instead of relying on the unbounded `Command::output`. The helper polls `child.try_wait()` with a 5000 ms (5 second) timeout.
- **Size Caps**: (Already implemented in T-00323) The `ToolchainManifest::from_source` explicitly uses `take(65_536)` to prevent large config files from exhausting memory.
- **Resource Cleanup**: In `run_with_timeout`, if the timeout is exceeded or `try_wait` returns an error, the child process is immediately reaped using `child.kill()` and `child.wait()`. This prevents zombie processes.
- **Standard Envelope**: All process execution errors or timeouts are returned as `Err(String)` wrapped in the standard envelope.
- **Audit Row**: The integration at the CLI layer (implemented in T-00326) relies on `aiosh-cli`'s top-level error handling, which uses `emit()` on both success and error branches (fail-open) and always generates an honest audit row.

## 3. Verification
Unit tests in `toolchain_service::tests` (including mismatched binary names which trigger wait errors and timeouts) were run via `cargo test`.
Results: `test result: ok. 90 passed; 0 failed`

The service is now properly hardened against hung tools (e.g. if `rustc -V` hangs indefinitely due to filesystem locks).
