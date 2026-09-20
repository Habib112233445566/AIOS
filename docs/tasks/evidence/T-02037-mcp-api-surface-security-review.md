# T-02037: Capability Model / MCP/API Surface — Security Review

**Task ID**: `T-02037`  
**Phase**: Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic**: Sub-Epic 4: Capability Model / MCP/API Surface  
**Status**: COMPLETED  
**Date**: 2026-09-20  

---

## 1. Executive Summary

This security review evaluates the attack surface and threat vectors introduced by exposing the Capability Model via the Model Context Protocol (`aiosh-mcp`). Because MCP tools are directly invocable by autonomous AI agents and external language models over `stdio` JSON-RPC, rigorous input sanitization, PEP gate enforcement, audit immutability, and state-isolation guarantees are critical.

---

## 2. Threat Vector Modeling & Abuse Scenarios

### THREAT-CAPMCP-01: Path Traversal & Backing Store Hijacking
- **Attack Vector**: An untrusted caller provides `store_path: "../../../etc/passwd"` or a symlink to an arbitrary sensitive file in tool arguments (`aios.capability.list/issue/etc.`).
- **Impact**: Arbitrary file read, unauthorized overwrite, or filesystem corruption.
- **Current Defense**: `validate_service_path` in `aiosh_core::capability_service`:
  - Enforces path length $\le 1024$ chars.
  - Strictly rejects `..` path components.
  - Rejects ASCII control characters (`< 32` or `\0`).
  - Requires `.json` file extension.
  - Verifies that the path is not a symlink.
- **Hardening Action Required**: Ensure that in `code/aiosh-rust/aiosh-mcp/src/main.rs`, any `store_path` argument is validated immediately upon extraction, rejecting paths containing control characters or traversal before passing to `CapabilityService`.

### THREAT-CAPMCP-02: Scope & Identifier Injection via Untrusted Content
- **Attack Vector**: An agent supplies control characters (`\n`, `\r`, ANSI escape sequences, null bytes) or unbounded strings in `subject`, `issuer`, `scope_type`, or `scope_target`.
- **Impact**: Log injection in the Audit Ring, terminal corruption during operator inspection, or bypass of scope matching.
- **Current Defense**: `validate_identifier` in `capability.rs` enforces length bounds ($\le 128$) and restricts characters to ASCII alphanumeric, `_`, `-`, `:`, `.`.
- **Hardening Action Required**: Add explicit string length bounds ($\le 128$ for IDs, $\le 256$ for subjects/issuers, $\le 1024$ for targets) and control character validation in MCP argument extraction before building closures.

### THREAT-CAPMCP-03: Unauthorized Root Capability Issuance
- **Attack Vector**: An unprivileged model calls `aios.capability.issue` with `issuer: "kernel"` or `issuer: "admin:root"` to forge root privileges.
- **Impact**: Total compromise of the capability security model.
- **Current Defense**: `CapabilityService::issue_root_capability` checks `issuer == "kernel" || issuer.starts_with("admin:")`.
- **Hardening Action Required**: The MCP server must verify that the caller possesses a valid PEP grant or trusted actor identity when calling `aios.capability.issue`. Any issuance without appropriate authority must be refused and logged with an honest audit row.

### THREAT-CAPMCP-04: Monotonic Attenuation Bypass & Privilege Escalation
- **Attack Vector**: A caller attempts to attenuate a child capability claiming rights not held by the parent (e.g. parent has `["read"]`, child requests `["read", "admin"]`) or expanding resource scope.
- **Impact**: Privilege escalation across delegation boundaries.
- **Current Defense**: `Capability::attenuate` enforces strict subset validation on rights (`child.rights ⊆ parent.rights`) and verifies `parent.has_right(CapabilityRight::Delegate)`.
- **Finding**: Monotonic attenuation is enforced at the core model level; tested in `test_capability_mcp_tools`.

### THREAT-CAPMCP-05: Quota Evasion & Resource Exhaustion (DoS)
- **Attack Vector**: A caller submits extremely large values or negative integers for `max_invocations` or `quota_bytes`, or attempts to flood the store with millions of capabilities.
- **Impact**: Memory exhaustion (OOM), integer overflow, or quota bypass.
- **Current Defense**: `MAX_CAPABILITIES_IN_REGISTRY = 10_000` and `MAX_CAPABILITY_STORE_SIZE = 10 MB` enforced in `CapabilityService`. Numeric arguments are parsed as `u64`.
- **Hardening Action Required**: Ensure MCP handlers enforce non-zero positive bounds on quotas and handle overflows safely.

### THREAT-CAPMCP-06: Audit Evasion & Silent Failures
- **Attack Vector**: Mutating actions (`issue`, `attenuate`, `revoke`, `prune`) fail silently or bypass the audit ring.
- **Impact**: Inability to reconstruct security incidents or prove non-repudiation.
- **Current Defense**: All 7 tools route through `dispatch::recorded_call`, which emits a hash-chained audit row to SQLite WAL before returning.

---

## 3. Review Verdict

**Status**: Zero blocking vulnerabilities identified. Hardening actions (input bounds, explicit rejection of control characters in MCP argument parsing) will be applied in `T-02038`.
