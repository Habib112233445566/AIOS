# Task Evidence: T-01869 - Network Bootstrap / security policy: Documentation

## 1. Overview
- **Task ID**: `T-01869`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Document the Network Bootstrap Security Policy subsystem for operators and agents.

---

## 2. Documentation Deliverables
Authored Section 10 in `docs/network_bootstrap.md`:
- Architecture and overview of `NetworkSecurityPolicy`.
- Invariants `NPOL1..NPOL6` table.
- Supported modes: `Enforcing` (deny), `Audit` (audit), and `Permissive` (allow).
- Complete JSON configuration schema (`net_policy.json`) and limits.
- Copy-pasteable Rust programmatic example code.
- Explicit statements on limitations (userspace snapshot evaluation, no direct kernel eBPF datapath filter injection).
- Cross-references to task evidence files `T-01861` through `T-01868`.
