# Task T-02501 Evidence: Privilege Escalation Prevention Data Model Research

## Goal
Establish facts, constraints, threat models, and prior art for the data model of the Privilege Escalation Prevention subsystem in AIOS.

## Facts vs. Assumptions

| Domain | Facts (Authoritative Sources) | Assumptions |
|---|---|---|
| OS Privilege Tiers | Operating systems partition authority via user IDs (UID 0 vs non-zero on POSIX), capabilities (`capabilities(7)`), or Windows Access Tokens (`TOKEN_PRIVILEGES`). Unbounded ambient authority allows compromised workers to escalate to root/admin. | AIOS requires a unified, substrate-independent privilege data model that abstracts POSIX capabilities and Windows privileges into discrete privilege tiers and capabilities. |
| Monotonic Privilege Law | Principle of Least Privilege: processes must only acquire privileges necessary for their explicit task and must never elevate without an authentic PEP capability grant. | Privilege level transitions can be modeled as formal state-transitions evaluated against a strict transition lattice. |
| Inherent SUID/Token Risks | Classical privilege escalation vectors include SUID/SGID execution, `CAP_SYS_ADMIN`, `SeImpersonatePrivilege`, and parent token inheritance. | Gating capability additions and child process tokens fail-closed blocks ambient inheritance. |

## Authoritative References
1. **Linux Capabilities**: `capabilities(7)` — granular slicing of root powers (`CAP_NET_BIND_SERVICE`, `CAP_SYS_PTRACE`, `CAP_DAC_OVERRIDE`, `CAP_SETUID`).
2. **POSIX Access Control**: `setuid(2)`, `setgid(2)`, `prctl(PR_SET_NO_NEW_PRIVS)`.
3. **Windows Security Tokens**: Access Tokens, Token Privileges (`SeDebugPrivilege`, `SeImpersonatePrivilege`), and Restricted Tokens.
4. **AIOS Security Architecture**: ADR-0035 (Security Kernel & PEP Fabric, Policy Enforcement Point governance).

## Data Model Architecture & Decisions
1. **Module Name**: `code/aiosh-rust/aiosh-core/src/privilege_data_model.rs`.
2. **Key Structures**:
   - `PrivilegeLevel`: `Guest` (0), `User` (10), `Operator` (20), `Admin` (30), `SystemKernel` (40).
   - `PrivilegeCapability`: `ProcessSpawn`, `NetworkAccess`, `FilesystemWrite`, `MemoryInspect`, `SystemReboot`, `AuditLogAdmin`, `KernelModuleLoad`.
   - `PrivilegeContext`: `actor_id`, `current_level`, `granted_capabilities`, `elevation_token`, `created_at_utc`.
   - `PrivilegeTransitionRequest`: `from_level`, `target_level`, `requested_capabilities`, `grant_id`.
   - `PrivilegeEscalationVerdict`: `Allowed`, `Denied(String)`, `GrantRequired`.
