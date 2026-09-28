# Task Evidence: T-02361 (Audit Chain Extensions / security policy: Research)

## 1. Metadata
- **Task ID:** `T-02361`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Security Policy Research
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 7: Security Policy (1/4) — Research

---

## 2. Research Findings & Prior Art Analysis

### 2.1 Prior Art & Existing AIOS Policy Patterns
In AIOS Phase 2 (Security Kernel & PEP Fabric), several subsystems enforce formal declarative security policies:
1. **PEP Decision Engine (`pep_security_policy.rs`)**:
   - Implements `PepEnforcementMode` (`Enforcing`, `Permissive`, `Disabled`).
   - Enforces temporal validity (`valid_from_epoch_secs`, `valid_until_epoch_secs`).
   - Restricts resource prefixes (`sys:`, `sec:`, `kernel:`).
   - Validates configuration byte size bounds (`MAX_PEP_SECURITY_POLICY_BYTES = 64 KiB`).
2. **Network Security Policy (`network_policy.rs`)**:
   - Implements `NetworkPolicyMode` (`Enforcing`, `Audit`, `Permissive`).
   - Enforces interface allowlists/blocklists and DNS server restrictions.
   - Resource quotas on interfaces, routes, and sensitive address redaction.
3. **Audit Chain Subsystem (`audit_chain_service.rs`, `audit_chain_config.rs`)**:
   - Currently, `AuditChainService` supports append, query, and cryptographic verification, but lacks a dedicated `AuditChainSecurityPolicy` to govern *who* can record events, *which* tools require cryptographic Ed25519 signatures, *what* causal links are permitted, and *how* suspicious provenance is rejected.

### 2.2 Facts vs. Assumptions

| Item | Status | Details |
| :--- | :---: | :--- |
| **Fact** | Confirmed | Audit Chain captures provenance and cryptographic SHA-256 state chains across all agent and kernel actions. |
| **Fact** | Confirmed | Without a formal security policy, untrusted or unauthenticated actors could emit arbitrary provenance events, simulate causal parents, or overwhelm the DAG with circular or extreme fan-out causal links. |
| **Fact** | Confirmed | Subsystems in AIOS follow a standard tri-mode enforcement pattern: `Enforcing` (fail-closed, denies violations), `Permissive` (evaluates and logs warnings without denying), and `Disabled` (bypasses policy checks). |
| **Fact** | Confirmed | Core error codes must follow a uniform taxonomy: `AUDITPOL_ERR_VALIDATION`, `AUDITPOL_ERR_DENIED`, `AUDITPOL_ERR_TEMPORAL`, and `AUDITPOL_ERR_IO`. |
| **Assumption** | Validated | Defining `AuditChainSecurityPolicy` in `code/aiosh-rust/aiosh-core/src/audit_chain_policy.rs` with event evaluation methods (`evaluate_event`, `evaluate_causal_link`) will integrate seamlessly with `AuditChainService` and maintain zero runtime regressions. |

### 2.3 Policy Vectors & Requirements (`AUDITPOL1..AUDITPOL6`)
The security policy specification will cover the following key vectors:
- **`AUDITPOL1` (Enforcement Modes)**: Support `Enforcing` (default, fail-closed), `Permissive` (audit-only), and `Disabled`.
- **`AUDITPOL2` (Actor & Tool Access Rules)**: Prohibit anonymous provenance (`actor` or `tool` cannot be empty); provide allowlist/blocklist mechanisms for actor IDs and sensitive tool namespaces.
- **`AUDITPOL3` (Cryptographic Signature Mandate)**: Require valid Ed25519 signatures for high-impact tool executions (e.g. tools prefixed with `kernel:`, `sec:`, `admin:`, or `pep:`).
- **`AUDITPOL4` (Causal Topology Limits)**: Enforce maximum causal parent links per event (default: 32) and maximum lineage traversal depth (default: 64) to prevent resource exhaustion attacks.
- **`AUDITPOL5` (Temporal & Monotonicity Constraints)**: Validate event timestamps against temporal bounds and prevent clock-skew spoofing (> 300s into future).
- **`AUDITPOL6` (Persistence & File Bounds)**: Safe JSON serialization/deserialization with maximum policy file size of 64 KiB.

---

## 3. Decisions & Next Steps
1. **Decision**: Implement `AuditChainSecurityPolicy` in `code/aiosh-rust/aiosh-core/src/audit_chain_policy.rs` with `AuditPolicyMode` enum (`Enforcing`, `Permissive`, `Disabled`) and policy evaluation structs.
2. **Decision**: Integrate policy verification directly into `AuditChainService`, allowing optional custom security policies while providing a secure default policy.
3. **Next Step**: Proceed to `T-02362` to formally author the specification artifact `docs/tasks/evidence/T-02362-security-policy-specification.md`.
