# T-02667: Secrets Handling Security Policy Security Review

- **Task**: `T-02667`
- **Sub-Epic**: Phase 2 — Security Kernel & PEP Fabric / Secrets Handling / security policy
- **Date**: 2026-10-03
- **Status**: PASSED

## 1. Security Review Scope
Audited the security policy subsystem of Secrets Handling across:
- Core declarative policy definitions: `code/aiosh-rust/aiosh-core/src/secret_policy.rs`
- Vault service policy enforcement: `code/aiosh-rust/aiosh-core/src/secret_service.rs`
- Userland CLI interface: `code/aiosh-rust/aiosh-cli/src/main.rs`
- Machine Context Protocol (MCP) dispatch: `code/aiosh-rust/aiosh-mcp/src/main.rs`

## 2. Abuse Scenarios & Mitigations

| Abuse Scenario | CWE ID | Threat Description | Mitigation / Defense |
|:---|:---|:---|:---|
| Path Traversal Injection | CWE-22 | Attacker supplies traversal tokens (`../`, `..\`) in `policy_path` to read or overwrite arbitrary system files. | Enforced in `SecretSecurityPolicy::load_from_path()`, `save_to_path()`, CLI parser, and MCP dispatcher; paths containing `..` are rejected fail-closed with `SECPOL_ERR_VALIDATION`. |
| Resource Exhaustion via Giant Policy File | CWE-400 / CWE-770 | Attacker provides multi-megabyte JSON file causing high memory allocation or parser exhaustion. | `fs::metadata` stat check bounds file size to `MAX_SECRET_SECURITY_POLICY_BYTES` (64 KiB); oversized files are rejected prior to reading. |
| Scope Escalation via Global Secret Proliferation | CWE-285 | Untrusted actors store global secrets to bypass per-tenant or per-actor isolation boundaries. | Under `disallow_global_secrets = true`, `evaluate_store()` rejects global scope registration fail-closed with `SECPOL_ERR_GLOBAL_DISALLOWED`. |
| Prohibited Kind Injection | CWE-693 | Untrusted actors register dangerous or non-approved credential types (e.g. raw private keys). | `prohibited_kinds` list evaluates entry metadata in `evaluate_store()` and rejects unauthorized types with `SECPOL_ERR_KIND_PROHIBITED`. |
| Unauthenticated / Masked Bypass Retrieval | CWE-200 | Retrieving plaintext secret values without explicitly opting into disclosure. | `evaluate_get()` requires `expose = true` under `require_expose_flag = true`, returning `SECPOL_ERR_EXPOSE_REQUIRED` when omitted. |
| Audit Evasion on Policy Alterations | CWE-778 | Operator or compromised agent modifies policy modes without an immutable record. | Every CLI invocation (`aiosh secret policy set-mode`) and MCP call (`aios.secret.policy`) emits a structured audit record into the append-only audit ring buffer with full actor attribution. |

## 3. Findings & Verdict
All evaluated abuse vectors are rigorously countered by compile-time type safety, bounds validation, traversal prevention, and fail-closed security invariants.
Verdict: **APPROVED FOR PRODUCTION**.
