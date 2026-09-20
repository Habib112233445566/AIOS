# Evidence: T-02065 - security policy: Unit Test

## Task Overview
- **Task ID**: `T-02065`
- **Sub-Epic**: Sub-Epic 7: Security Policy (`T-02061`..`T-02070`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Add focused automated unit tests for Capability Security Policy (`CAPSEC1..CAPSEC6`).

## Unit Test Coverage
Created `code/aiosh-rust/aiosh-core/tests/test_capability_policy.rs` with 8 comprehensive unit tests:
1. `test_policy_validation`: Tests policy structural invariants (attenuation depth bounds, prohibited paths without traversal or empty strings, positive duration bounds).
2. `test_policy_prohibited_filesystem_paths`: Tests blocking of sensitive paths (`/etc/shadow`, `/proc`, `/sys`, `/dev`, `/root`, `C:\Windows\System32`) across `Enforcing`, `Audit`, and `Permissive` modes.
3. `test_policy_prohibited_network_hosts`: Tests blocking cloud metadata IP (`169.254.169.254`) and hostname (`metadata.google.internal`) while allowing legitimate endpoints.
4. `test_policy_prohibited_tools`: Tests blocking privileged tools (`raw_syscall`, `kernel_module_load`, `reboot`) while allowing normal tools (`grep`).
5. `test_policy_disallowed_rights_by_subject`: Tests subject prefix rules (blocking `Admin`, `Delegate`, `Delete` for `untrusted:*`, blocking `Write` for `guest:*`).
6. `test_policy_temporal_and_quota_constraints`: Tests missing expiration rejection, excessive validity duration rejection, and invocation ceiling enforcement.
7. `test_policy_attenuation_depth_limit`: Tests attenuation depth limits on custom policies (depth 1..3 succeed, depth 4 rejected with `CAPSEC_DEPTH_EXCEEDED`).
8. `test_capability_service_policy_enforcement`: Tests end-to-end policy integration with `CapabilityService` for both root issuance and child attenuation.

## Test Results
```text
running 8 tests
test test_policy_disallowed_rights_by_subject ... ok
test test_policy_prohibited_filesystem_paths ... ok
test test_capability_service_policy_enforcement ... ok
test test_policy_attenuation_depth_limit ... ok
test test_policy_prohibited_tools ... ok
test test_policy_prohibited_network_hosts ... ok
test test_policy_temporal_and_quota_constraints ... ok
test test_policy_validation ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
