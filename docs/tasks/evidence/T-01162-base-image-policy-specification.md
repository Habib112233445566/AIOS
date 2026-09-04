# T-01162 — Base Image Build / Security Policy: Specification

**Date:** 2026-09-04
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Specification Contract: `BaseImageSecurityPolicy`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseImageSecurityPolicy {
    pub mode: PolicyMode,
    pub prohibited_kernel_params: Vec<String>,
    pub prohibited_packages: Vec<String>,
    pub allowed_architectures: Vec<String>,
    pub allowed_filesystems: Vec<String>,
    pub require_core_packages: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule_id: String,
    pub description: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub passed: bool,
    pub mode: PolicyMode,
    pub violations: Vec<PolicyViolation>,
}
```

## 2. Invariant Specifications (P1..P6)
- **P1 (Kernel Mitigation Invariant)**: Prohibits kernel parameters that disable hardware/software exploit mitigations (`nokaslr`, `mitigations=off`, `pti=off`, `spec_store_bypass_disable=prctl`).
- **P2 (LSM Integrity Invariant)**: Prohibits disabling Linux Security Modules (`selinux=0`, `apparmor=0`, `enforcing=0`).
- **P3 (Init / Shell Bypass Invariant)**: Prohibits parameters subverting systemd or init (`init=/bin/sh`, `init=/bin/bash`, `init=/bin/dash`, `single`, `emergency`).
- **P4 (Prohibited Package Invariant)**: Prohibits legacy, cleartext, or insecure packages (`telnet`, `rsh-client`, `rsh-server`, `rlogin`, `rexec`, `nis`, `yp-tools`).
- **P5 (Architecture Whitelist Invariant)**: Architectures must be one of `x86_64`, `aarch64`, or `riscv64`.
- **P6 (Filesystem Whitelist Invariant)**: Filesystem type must be one of `ext4`, `squashfs`, `btrfs`, `erofs`, or `xfs`.

## 3. Enforcement Behavior
- **Enforcing**: If any violation occurs, `PolicyEvaluationResult.passed` is `false`, and build plan generation or image creation must halt with `Err(PolicyViolation)`.
- **Audit**: Violations are collected, `PolicyEvaluationResult.passed` remains `true` (non-fatal), but violations are logged to the AIOS audit system.
- **Permissive**: No violations are emitted.
