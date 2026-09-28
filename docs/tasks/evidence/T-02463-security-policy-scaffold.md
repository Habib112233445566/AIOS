# T-02463: Sandbox Enforcement Security Policy Scaffold

## 1. Scaffold Overview
This task establishes the module skeleton and interface contracts for `SandboxSecurityPolicy` in `code/aiosh-rust/aiosh-core/src/sandbox_policy.rs` and re-exports it in `aiosh-core/src/lib.rs`.

---

## 2. Types & Interfaces Scaffolding
- `SandboxPolicyMode`: Tri-mode enum (`Enforcing`, `Permissive`, `Disabled`).
- `SandboxPolicyVerdict`: Evaluation result enum (`Permit`, `PermitWithWarning`, `Deny`).
- `SandboxSecurityPolicy`: Struct with validation, evaluation, load/save capabilities.
- Error constants: `SANDBOXPOL_ERR_VALIDATION`, `SANDBOXPOL_ERR_DENIED`, `SANDBOXPOL_ERR_IO`, `SANDBOXPOL_ERR_PARSE`.
- Capacity constants: `MAX_PROHIBITED_COMMANDS`, `MAX_PROHIBITED_ENV_VARS`, `MAX_PEP_MANDATED_PROFILES`, `MAX_SANDBOX_SECURITY_POLICY_BYTES`.

---

## 3. Build Status
- Workspace compiles with 0 warnings and 0 errors (`cargo check --workspace`).
