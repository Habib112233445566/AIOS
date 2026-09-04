# T-01165 — Base Image Build / Security Policy: Unit Test

**Date:** 2026-09-04
**Type:** Unit Test
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Test Suite Summary
- Implemented `code/aiosh-rust/aiosh-core/tests/test_base_image_policy.rs` testing 7 criteria:
  - `test_p1_kernel_hardening_invariants`: validates rejection of `nokaslr`, `mitigations=off`, `pti=off`.
  - `test_p2_lsm_invariants`: validates rejection of `selinux=0`, `apparmor=0`, `enforcing=0`.
  - `test_p3_init_bypass_invariants`: validates rejection of `init=/bin/sh`, `single`, `emergency`.
  - `test_p4_package_blacklist`: validates rejection of legacy cleartext packages (`telnet`, `rsh-client`, etc.).
  - `test_p5_arch_and_fs_whitelists`: validates rejection of non-whitelisted architectures and filesystems.
  - `test_p6_enforcement_modes_and_policy_override`: validates `Enforcing` vs `Audit` vs `Permissive` modes and env-based configuration ingestion.
  - `test_p7_store_policy_filtering`: validates batch policy checking and compliant filtering over an `ImageStore`.
- All 7 tests executed cleanly and passed: `test result: ok. 7 passed; 0 failed`.
