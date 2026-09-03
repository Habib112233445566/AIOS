# T-01061 — Distro Selection & Justification / Security Policy: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Security Policy

## 1. Objectives & Threat Landscape
Research security policy controls for the Linux Distro Selection & Justification subsystem in AIOS:
- **Supply Chain Integrity**: Distro profiles define base images and package sources. Malicious or compromised profiles pointing to unauthenticated mirror URLs or vulnerable packages represent high-severity supply chain attack vectors.
- **Production Baseline Hardening**: Distribution profiles flagged as production-ready or recommended must satisfy minimum mandatory security criteria:
  1. Signed repositories / GPG verification enabled.
  2. Minimal attack surface (footprint score $\ge 0.50$, security score $\ge 0.70$).
  3. No unverified third-party binary repositories.
  4. Memory corruption defenses and modern Linux security modules (SELinux / AppArmor / Landlock / seccomp).
- **Actor Authorization & Policy Enforcement Point (PEP)**:
  - Read-only operations (`list`, `show`, `evaluate`, `recommend`) accessible to standard callers.
  - Store modifications (`register`, `delete`, store path mutations) restricted to `operator` or `admin` actors.
- **Audit Immutability**: Every security policy evaluation or policy rejection generates an auditable structured event in the SQLite WAL audit trail.

## 2. Policy Invariants vs. Assumptions
| Control | Status | Invariant / Requirement |
|---|---|---|
| Repository Signature Enforcement | Fact | Recommended profiles must enforce signed repository verification. |
| Production Security Floor | Fact | `security_score >= 0.70` required for any profile marked `recommended = true`. |
| Memory & Landlock Sandbox | Fact | Target distributions must support Landlock and seccomp-bpf. |
| Audit Row Invariant | Fact | Security policy checks emit structured audit records via PEP. |
