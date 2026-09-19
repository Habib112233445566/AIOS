# T-01668: Security Policy Hardening

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Harden the Kernel Module Management Security Policy against failure modes, malformed inputs, symlink/FIFO attacks, and resource exhaustion.

## Hardening Implemented
1. **File Type & Size Bounds**:
   - Added explicit `metadata.is_file()` validation to reject FIFOs, device nodes, and directories.
   - Strictly enforced `MAX_POLICY_FILE_BYTES` (64 KiB) limit before and during reading (`reader.take(MAX_POLICY_FILE_BYTES + 1)`).
2. **Error Envelope Integrity**:
   - Ensured all failure paths in CLI and MCP return standardized error envelopes (`code`, `data`, `error` with `code` and `message`).
   - Every failure emits an honest audit row to the append-only audit ring via `classify_and_emit` or `dispatch::recorded_call`.
3. **Resource Leak Prevention**:
   - Zero persistent file descriptors or connection leaks across error paths.
   - Clean handling of corrupted or missing policy files.

## Verification
- Unit test suite verified via `cargo test -p aiosh-core --test test_kernel_module_policy` (all 6 passed).
- CLI/MCP integration verified via `test_kernel_module_policy_smoke.py`.
