# T-02461: Sandbox Enforcement Security Policy Research

## 1. Context & Objectives
This research establishes the facts, constraints, and prior art for declarative security policy enforcement over the Sandbox Enforcement subsystem in AIOS.

---

## 2. Established Facts vs Assumptions

### Established Facts
1. **Existing Policy Precedents**:
   - `AuditChainSecurityPolicy` (`audit_chain_policy.rs`) and `CapabilitySecurityPolicy` (`capability_policy.rs`) implement tri-mode policy governance (`Enforcing`, `Permissive`, `Disabled`), returning structured verdicts (`Permit`, `PermitWithWarning`, `Deny`).
2. **Containment Surface**:
   - `SandboxProfile` already models resource limits, filesystem paths, network isolation modes, and syscall filters.
   - However, currently, the decision whether a specific `SandboxExecutionRequest` is permitted to run under a given profile is determined only by individual profile validation and optional `enforce_pep_grants` flag.
3. **Defense-in-Depth Requirements (ADR-0035 / Zero Ambient Authority)**:
   - A system-level security policy (`SandboxSecurityPolicy`) must enforce global invariant rules across *all* execution requests regardless of profile.
   - Global rules include:
     - Disallowed binaries/interpreters (e.g., destructive system utilities).
     - Prohibited environment variable injection (e.g., `LD_PRELOAD`, `PYTHONPATH`).
     - Hard caps on wall time and memory allocation across any request.
     - Mandatory PEP capability grants for elevated profiles.

### Assumptions
- A default `SandboxSecurityPolicy` should be loaded from disk or initialize to secure defaults (`Enforcing` mode).
- Bounded file size constraint: `MAX_SANDBOX_SECURITY_POLICY_BYTES = 64 * 1024` (64 KiB), matching existing AIOS policies.

---

## 3. Decisions & Interface Contracts
- Module: `code/aiosh-rust/aiosh-core/src/sandbox_policy.rs`
- Invariants: `SANDBOXPOL1`..`SANDBOXPOL6`
- CLI command: `aiosh sandbox policy [show|check]`
- MCP tool: `aios.sandbox.policy`
