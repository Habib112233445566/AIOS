# T-01062 — Distro Selection & Justification / Security Policy: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Security Policy

## 1. Data Contracts

### A. `DistroSecurityPolicy`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroSecurityPolicy {
    pub min_security_score: f32,
    pub min_binary_compatibility_score: f32,
    pub require_https_repositories: bool,
    pub require_signed_packages: bool,
    pub disallowed_distro_families: Vec<String>,
}
```

### B. `DistroPolicyVerdict`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroPolicyVerdict {
    pub profile_id: String,
    pub allowed: bool,
    pub violations: Vec<String>,
    pub evaluated_at: String,
}
```

## 2. Policy Enforcement Rules (P1..P5)
- **P1 (Score Floor)**: `evaluation.security_score >= policy.min_security_score`.
- **P2 (Binary Compatibility)**: `evaluation.binary_compatibility_score >= policy.min_binary_compatibility_score`.
- **P3 (Transport Security)**: Repository mirrors and sources must use `https://` schemes.
- **P4 (Package Verification)**: If `require_signed_packages` is true, package management must enforce cryptographically signed packages.
- **P5 (Family Whitelist/Blacklist)**: Distribution family must not be in `disallowed_distro_families`.
