# T-00778 — Secrets & Access Hygiene / security policy: Hardening

## 1. Hardening Deliverables
- **Policy Invariant Gating**: Enforced strict validation across OpenSSF scorecard criteria S1..S5 in `tools/check_security_policy.py`.
- **Rot-Proof In-Tree Linking**: Added automated verification that every relative link in `SECURITY.md` resolves to a valid, committed file in the repository.
- **Fail-Closed Reporting**: Missing security files or policy violations immediately halt CI pipelines with non-zero exit codes.
