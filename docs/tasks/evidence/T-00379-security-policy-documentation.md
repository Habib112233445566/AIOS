# T-00379 — Dependency & Toolchain Pinning / security policy: Documentation

## 1. Documentation Scope
This task updates user and operator documentation with the security policy rules, PEP grant requirements, and audit trail guarantees governing Dependency & Toolchain Pinning.

## 2. Documentation Updates
- **Document Updated**: `docs/README.md`
- **Section Added**: `Security Policy (PEP Gating & Audit)` under `## Dependency & Toolchain Pinning (T-00311..T-00380)`

### Summary of Documented Policies:
1. **PEP Gating**: State-mutating commands (`aios.toolchain.set`, `toolchain.set`) are classified as `is_irreversible` and require explicit cryptographic PEP grants.
2. **Audit Trail**: Read-only (`toolchain.check`, `toolchain.show`, `aios.toolchain.check`, `aios.toolchain.config.get`) and mutating commands emit immutable rows to the audit ring.
3. **Refusals**: Missing or invalid grant tokens trigger synchronous refusals with failure audit rows.

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
- `python tools/check_security_policy.py` -> PASS (S1..S5)
