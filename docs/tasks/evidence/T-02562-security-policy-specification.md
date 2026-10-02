# T-02562: Privilege Escalation Prevention Security Policy Specification

- **Task**: `T-02562`
- **Sub-Epic**: Privilege Escalation Prevention / security policy
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Specification Objectives
Define the contractual types, error taxonomies, evaluation algorithms, and persistence invariants for declarative security policy in `code/aiosh-rust/aiosh-core/src/privilege_policy.rs`.

## 2. Invariant Contracts
- `PRIVESCPOL1`: Kernel tier immutability enforcement.
- `PRIVESCPOL2`: 64 KiB configuration and policy file bounds.
- `PRIVESCPOL3`: Rejection of path traversal sequences in storage paths.
- `PRIVESCPOL4`: Tri-state policy modes (`Enforcing`, `Permissive`, `Disabled`).

See full details in `docs/SPEC-PRIVILEGE-POLICY.md`.
