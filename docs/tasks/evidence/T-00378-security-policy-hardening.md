# T-00378 — Dependency & Toolchain Pinning / security policy: Hardening

## 1. Hardening Overview
This task hardens the security policy mechanisms for Dependency & Toolchain Pinning against resource leaks, silent failure modes, and improper bypass conditions.

## 2. Hardening Measures
1. **Zero-Allocation Pure Logic**:
   - `check_toolchain_policy` and `pep::is_irreversible` operate as pure functions over borrowed string references (`&str`, `Option<&str>`). They open no filesystem handles, child processes, or persistent database connections, making resource leaks impossible on the policy evaluation path.
2. **Deterministic Fail-Closed Error Reporting**:
   - Invocations lacking required cryptographic grants immediately return explicit, auditable error strings naming the denied action (e.g. `"Action 'aios.toolchain.set' is irreversible and requires an active PEP grant"`).
   - Silent failures or silent ignores are prohibited.
3. **Honest Audit Recording (ADR-0035 §F-2)**:
   - When policy checks fail, the refusal reason is captured losslessly and persisted directly into the immutable audit ring WAL as an auditable row with `outcome: "refused"` or `outcome: "error"`.

## 3. Acceptance Verification
- All failure modes produce explicit, structured error envelopes.
- No temporary file, process, or database connection leaks exist on error paths.
