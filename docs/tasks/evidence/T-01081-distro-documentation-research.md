# T-01081 — Distro Selection & Justification / Documentation: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Documentation

## 1. Documentation Scope Research
Research the documentation requirements for the Linux Distro Selection & Justification component:
- **Core Narrative & Justification**:
  - Detailed architectural rationale for selecting Debian 12 Minimal (`bookworm`) as the tier-1 production base system.
  - Justification factors: glibc compatibility (for PyTorch, Rust toolchains, binary agents), 5-year security updates, apt/deb ecosystem maturity, systemd service isolation, container base image compatibility.
  - Comparative analysis against alternatives:
    - Alpine 3.19 (musl libc binary compatibility challenges with complex AI runtimes, though excellent for minimal containers).
    - Arch Linux (rolling release instability risk for predictable production base OS).
    - Fedora CoreOS (immutability benefits vs deployment complexity).
- **Subsystems to Document**:
  1. Data Model (`DistroProfile`, `DistroFamily`, `CpuArch`, `DistroEvaluation`, `DistroScores`).
  2. Storage & Registry (`DistroStore`, `distro_store.json`, corruption recovery).
  3. CLI Commands (`aiosh distro list|show|evaluate|recommend|config|policy|stats`).
  4. MCP Server Tools (`aios.distro.list|show|evaluate|recommend|policy|stats`).
  5. Configuration (`config/distro.json`, environment variables, precedence resolution).
  6. Automated Test Battery (D1..D5 criteria, U01..U10 assertions).
  7. Security Policy (`DistroSecurityPolicy`, score floors, family blacklists).
  8. Observability & Telemetry (`DistroObservabilityReport`, invariants O1..O4).
- **Target Location**: `docs/distro_selection.md` linked to `docs/README.md`.
- **Integrity Criteria**: Compliance with `tools/check_task_docs.py` (C1..C6).
