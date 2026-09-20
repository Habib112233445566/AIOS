# Task Evidence: T-02007 - Capability Model / data model: Security Review (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02007`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Perform comprehensive security review and threat modeling for the Capability Model data model.

---

## 2. Threat Model Analysis (THREAT-CAP-01..06)

| Threat ID | Threat Description | Attack Vector / Trigger | Severity | Mitigation Strategy |
|---|---|---|---|---|
| `THREAT-CAP-01` | **Privilege Escalation via Attenuation** | Adversary attempts to derive a child capability granting rights not held by the parent or expanding the resource scope. | Critical | Verify that all child rights exist in parent (`CAP3`); enforce `matches_scope` on child scope against parent scope; require `CapabilityRight::Delegate` on parent. |
| `THREAT-CAP-02` | **Subject / Identifier Injection** | Attacker crafts `subject` or `issuer` containing control characters (`\0`, `\n`, `\r`) to corrupt audit logs or spoof identifiers. | High | Validate that `issuer` and `subject` contain only alphanumeric and safe punctuation (`:`, `_`, `-`, `.`), length $\le 128$, no control characters. |
| `THREAT-CAP-03` | **Filesystem Scope Path Traversal** | Attacker specifies a path containing `..` or relative paths in `CapabilityScope::Filesystem` to escape directory boundaries. | High | Validate filesystem paths: enforce absolute paths, reject `..` components, reject control characters. |
| `THREAT-CAP-04` | **Malformed RFC3339 Temporal Bounds** | Adversary supplies unparseable or malicious timestamps for `not_before` or `expires_at`. | Medium | Validate RFC3339 timestamps during capability construction and attenuation, rejecting invalid formats upfront. |
| `THREAT-CAP-05` | **Quota Exhaustion & Overflow** | Invocation or byte consumption arithmetic overflows, wrapping around to zero. | High | Enforce `saturating_add` on all quota tracking; strictly check against quota ceilings before and after operations. |
| `THREAT-CAP-06` | **Lineage Forgery & Disconnected Revocation** | Attacker creates or modifies a capability claiming an arbitrary `parent_id` to evade revocation tracking. | High | Require Security Kernel cryptographic signature/token on capability issuance and validate parent existence during derivation. |

---

## 3. Hardening Plan for T-02008
1. **Input Validation**:
   - In `Capability::new` and `attenuate`, validate `issuer` and `subject` (disallow control characters, newlines, NUL bytes).
   - Validate `CapabilityScope::Filesystem` paths (require absolute paths, reject `..`).
   - Validate `CapabilityConstraints` timestamp strings (`not_before`, `expires_at`) using `chrono::DateTime::parse_from_rfc3339`.
2. **Scope Matching Hardening**:
   - In `matches_scope` for filesystem, ensure path prefix comparison properly accounts for directory boundary (e.g. `/var/data` must not match `/var/data_secret`).
3. **Quota Monotonicity**:
   - Ensure child capabilities cannot have quota limits larger than parent remaining quotas.
