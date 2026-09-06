# T-01365: Init & Service Supervision - Security Policy: Unit Test

## Metadata
- **Task ID:** `T-01365`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Security Policy
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Scope & Objective
Create and execute focused automated integration and unit tests for `ServiceSecurityPolicy` in `code/aiosh-rust/aiosh-core/tests/test_service_policy.rs` covering all positive, negative, and boundary scenarios for criteria `SP1..SP6`.

---

## 2. Test Cases Implemented
1. `test_sp1_policy_configuration_bounds_and_defaults`:
   - Validates default policy initialization and parameter validation.
   - Negative boundary: empty `allowed_service_types`.
   - Negative boundary: `max_env_vars` out of bounds ($0$ or $> 1024$).
   - Negative boundary: `max_timeout_secs` out of bounds ($> 86400$).
   - Negative boundary: relative path in `prohibited_exec_paths`.
   - Negative boundary: illegal environment variable name (containing `=`).
2. `test_sp2_prohibited_service_blocking`:
   - Positive: legitimate system daemon (`aiosh-agent.service`) allowed with empty violation list.
   - Negative: `telnet.service` prohibited.
   - Negative: bare name `telnet` correctly matches `.service` suffix rule.
   - Negative: `rsh.service` and `tftp.service` prohibited.
3. `test_sp3_executable_path_and_working_dir_hygiene`:
   - Negative: relative binary path in `exec_start` rejected (`SP3-RELATIVE-PATH`).
   - Negative: binary residing in prohibited prefix `/tmp/` rejected (`SP3-PROHIBITED-PATH`).
   - Negative: directory traversal (`..`) in binary arguments rejected (`SP3-PATH-TRAVERSAL`).
   - Negative: directory traversal in `working_dir` rejected (`SP3-PATH-TRAVERSAL`).
   - Negative: `working_dir` in prohibited prefix `/dev/shm` rejected (`SP3-PROHIBITED-PATH`).
4. `test_sp4_user_privilege_and_root_hygiene`:
   - Enforces unprivileged execution when `require_service_user = true` (`SP4-UNPRIVILEGED-USER-REQUIRED`).
   - Verifies exemption for whitelisted `allowed_root_services` (e.g. `systemd-journald.service`).
   - Enforces root restriction when `disallow_root = true` (`SP4-ROOT-DISALLOWED`).
   - Verifies exemption for whitelisted `aios-securityd.service`.
5. `test_sp5_environment_and_parameter_sanitization`:
   - Negative: dangerous `LD_PRELOAD` injection rejected (`SP5-DANGEROUS-ENV-VAR`).
   - Negative: service timeouts exceeding policy limits rejected (`SP5-TIMEOUT-EXCEEDED`).
   - Negative: disallowed service type rejected (`SP5-DISALLOWED-TYPE`).
6. `test_sp6_policy_modes_store_evaluation_and_file_roundtrip`:
   - Mode `Audit`: fatal violations do not block `allowed` (`allowed == true`), violations reported.
   - Mode `Permissive`: suppresses non-fatal violations but blocks prohibited services.
   - Store evaluation: verifies multi-service store policy evaluation (`evaluate_store`).
   - JSON file serialization and loading (`from_file`).
   - Size limit hardening: rejects policy file exceeding 64 KiB.

---

## 3. Execution & Verification Output
```text
running 6 tests
test test_sp1_policy_configuration_bounds_and_defaults ... ok
test test_sp2_prohibited_service_blocking ... ok
test test_sp3_executable_path_and_working_dir_hygiene ... ok
test test_sp4_user_privilege_and_root_hygiene ... ok
test test_sp5_environment_and_parameter_sanitization ... ok
test test_sp6_policy_modes_store_evaluation_and_file_roundtrip ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```
