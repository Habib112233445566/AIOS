# T-01368: Init & Service Supervision / Security Policy - Hardening

## Overview
Task `T-01368` hardens the Init & Service Supervision security policy subsystem across bounds checking, file I/O boundaries, path traversal protections, stream limiting, and error propagation:

1. **File I/O Boundary Defense**:
   - `from_file` path validation: path strings are restricted to $\le 1024$ characters and reject ASCII control characters (such as null bytes `\0`).
   - Strict size ceiling: enforced via `file.take(MAX_POLICY_FILE_BYTES + 1)` ensuring that policy files larger than 64 KiB (`MAX_POLICY_FILE_BYTES = 65_536`) cannot cause unbounded buffer allocations or memory exhaustion.
2. **Prohibited Executable Paths & Path Traversal Prevention**:
   - Prohibited execution paths (`prohibited_exec_paths`) bounded to a maximum of 128 entries, each verified to be an absolute path.
   - Evaluates `exec_start`, `exec_stop`, `exec_reload`, and `working_dir` for directory traversal sequences (`..`), immediately emitting fatal violation `SP3-PATH-TRAVERSAL`.
   - Rejects execution from world-writable or transient mount namespaces (`/tmp`, `/var/tmp`, `/dev/shm`, `/run/user`).
3. **Environment & Privilege Hardening**:
   - Disallowed environment variables (`disallow_env_vars`) strictly capped at 128 entries, rejecting variables containing `=` or control characters.
   - Blocks dynamic linker injection vectors (`LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`).
   - Enforces least privilege via `require_service_user` and `disallow_root` with explicit whitelist exemptions in `allowed_root_services`.
4. **Fail-Closed Semantic Enforcement**:
   - In `Enforcing` mode, any fatal violation (`fatal: true`) forces `allowed = false`.
   - In `Permissive` mode, prohibited services (`SP2-PROHIBITED-SERVICE`) unconditionally force `allowed = false`.
   - Bounded timeouts ($[1..86400]$s) and maximum environment variables ($[1..1024]$) prevent process hang and allocation exhaustion.
5. **Comprehensive Hardening Unit Suite**:
   - Added `test_sp7_hardening_and_boundary_checks` in `code/aiosh-rust/aiosh-core/tests/test_service_policy.rs` verifying control character paths, oversized path lengths, fail-closed enforcement, and multi-argument binary parsing.

---

## Test Verification Output
```text
running 7 tests
test test_sp2_prohibited_service_blocking ... ok
test test_sp4_user_privilege_and_root_hygiene ... ok
test test_sp3_executable_path_and_working_dir_hygiene ... ok
test test_sp1_policy_configuration_bounds_and_defaults ... ok
test test_sp7_hardening_and_boundary_checks ... ok
test test_sp5_environment_and_parameter_sanitization ... ok
test test_sp6_policy_modes_store_evaluation_and_file_roundtrip ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```
