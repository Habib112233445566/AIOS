# T-02561: Privilege Escalation Prevention Security Policy Research

- **Task**: `T-02561`
- **Sub-Epic**: Privilege Escalation Prevention / security policy
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Research Objectives
Establish facts, constraints, and prior art for declarative governance and security policy enforcement for the Privilege Escalation Prevention subsystem in `code/aiosh-rust/aiosh-core/src/privilege_policy.rs`.

## 2. Prior Art & Subsystem Analysis
Across the AIOS architecture, security policy components (`sandbox_policy.rs`, `pep_grant_security_policy.rs`, `network_policy.rs`):
1. **Tri-State Enforcement Modes**:
   - `Enforcing`: Strict fail-closed rejection with auditable errors.
   - `Permissive`: Permit with warnings emitted to telemetry.
   - `Disabled`: Bypass mode for emergency maintenance.
2. **Explicit Verdict Model**:
   - `Permit`, `PermitWithWarning { warning }`, `Deny { reason, code }`.
3. **Declarative Rule Vectors**:
   - Disallowed elevation targets (e.g. `SystemKernel` immutability).
   - Prohibited capabilities (e.g. `KernelModuleLoad` restricted by default).
   - Mandatory grant token enforcement.
   - Maximum allowable grant lifetime.
   - Per-actor tier ceiling maps.
4. **Persistence & Safety Bounds**:
   - File size capped at 64 KiB (`MAX_PRIVILEGE_SECURITY_POLICY_BYTES`).
   - Path traversal rejection prohibiting `..` path segments.

## 3. Facts vs Assumptions
- **Fact**: Policy evaluation must intercept every transition request before context mutation.
- **Fact**: Policy serialization must be standard JSON and support load/save via CLI and MCP.
- **Fact**: In enforcing mode, any attempt to violate actor tier ceiling or requested prohibited capability must return `PRIVESCPOL_ERR_DENIED`.

## 4. Key Decisions for Specification (T-02562)
1. Define `PrivilegeSecurityPolicy`, `PrivilegePolicyMode`, and `PrivilegePolicyVerdict` in `code/aiosh-rust/aiosh-core/src/privilege_policy.rs`.
2. Integrate `PrivilegeSecurityPolicy` into `PrivilegeService` with evaluation during `request_elevation()`.
3. Expose policy commands in `aiosh-cli` and tools in `aiosh-mcp`.
