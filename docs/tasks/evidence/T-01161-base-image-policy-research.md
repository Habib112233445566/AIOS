# T-01161 — Base Image Build / Security Policy: Research

**Date:** 2026-09-04
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Executive Summary & Objective
The Base Image Build subsystem constructs root filesystems, kernel configurations, and bootable images. A formal security policy engine is required to enforce invariants on image definitions prior to build plan synthesis or disk generation.

## 2. Facts vs Assumptions
### Facts (Observed in Codebase & Linux Security Standards):
1. **Kernel Command Line Parameters**: Disabling kernel address space layout randomization (`nokaslr`), disabling LSMs (`selinux=0`, `apparmor=0`), or switching init to interactive shells (`init=/bin/sh`, `init=/bin/bash`) severely weakens system integrity.
2. **Package Security & Insecure Daemons**: Legacy unencrypted remote access tools (`telnet`, `rsh-client`, `rsh-server`, `rlogin`, `rexec`, `nis`, `yp-tools`) present critical vulnerabilities if baked into production base images.
3. **Filesystem Whitelist**: Only verified and journaled or immutable Linux filesystems (`ext4`, `squashfs`, `btrfs`, `erofs`, `xfs`) provide the required security guarantees for root filesystems.
4. **Current Implementation**: `code/aiosh-rust/aiosh-core/src/base_image.rs` contains basic validation (`I1..I6`) in `validate_base_image_manifest()`, but lacks dedicated policy enforcement profiles (e.g. Enforcing vs Permissive vs Audit modes).

### Assumptions:
1. Security policy should be configurable per build target (Strict/Production vs Permissive/Development).
2. Failures in Enforcing mode must fail-closed and reject plan generation with an explicit audit rejection code.
3. In Audit mode, violations should emit structured audit warning records into the AIOS AuditRing without terminating the build.

## 3. Policy Rule Specifications (P1..P6)
- **P1: Kernel Mitigation Invariant**: Prohibit `mitigations=off`, `nokaslr`, `pti=off`, `spec_store_bypass_disable=prctl`.
- **P2: LSM Invariant**: Prohibit `selinux=0`, `apparmor=0`, `enforcing=0`.
- **P3: Root Shell / Init Bypass**: Prohibit `init=/bin/sh`, `init=/bin/bash`, `init=/bin/dash`, `single`, `emergency`.
- **P4: Prohibited Package Invariant**: Reject packages in `PROHIBITED_PACKAGES` list (`telnet`, `rsh-client`, `rsh-server`, `rlogin`, `rexec`, `nis`).
- **P5: Mandatory Package Invariant**: In Strict mode, enforce inclusion of core integrity packages (`base-files`, `coreutils`).
- **P6: Architecture Whitelist**: Approved architectures are `x86_64`, `aarch64`, `riscv64`.

## 4. Decisions Needed & Next Steps
- Implement policy module in `code/aiosh-rust/aiosh-core/src/base_image_policy.rs`.
- Proceed to T-01162 for formal Specification of `BaseImagePolicy`, `PolicyViolation`, and enforcement rules.
