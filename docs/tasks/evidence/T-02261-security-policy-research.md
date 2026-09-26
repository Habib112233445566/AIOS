# T-02261 Research: Grant Lifecycle Security Policy

**Task:** Establish facts, constraints, and prior art for the security policy of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. Executive Summary

This research establishes the formal constraints, standards, and design invariants for the **Grant Lifecycle Security Policy** (`PepGrantSecurityPolicy`) subsystem in AIOS. 

Whereas the generic PEP Decision Engine security policy (`pep_security_policy.rs`, PEPPOL1..6) manages decision modes (Enforcing, Permissive, Disabled) and rule obligations, the **Grant Lifecycle Security Policy** governs credential issuance, rights attenuation monotonicity, delegation depth limits, lifetime bounds, forbidden right combinations, and subject credential trust levels.

---

## 2. Established Facts vs Assumptions

### 2.1 Established Facts (Source Code & Invariants)
1. **Delegation Invariant (POL-FACT-1):** In `pep_grant.rs`, grants attenuation must be strictly monotonic: child rights must be a subset of parent rights (`child.rights ⊆ parent.rights`).
2. **Depth Invariant (POL-FACT-2):** Max delegation depth is globally bounded (`MAX_DELEGATION_DEPTH_LIMIT = 8` in `pep_grant.rs` and `MAX_DELEGATION_DEPTH = 10` in `pep_grant_config.rs`).
3. **Identifier Format (POL-FACT-3):** Grant IDs, subject IDs, and issuer IDs are restricted to alphanumeric and `_ - : .` with max length 128 chars.
4. **State Machine (POL-FACT-4):** Grant states are `active`, `suspended`, `revoked`, `expired`. Transitions from terminal states (`revoked`, `expired`) to `active` are permanently disallowed.
5. **Cascade Requirement (POL-FACT-5):** When a parent grant is revoked, cascading revocation must invalidate all descendant child grants recursively.

### 2.2 Assumptions Requiring Policy Formalization
1. **Maximum Validity Lifetime (POL-ASSUME-1):** Grants must not be issued with an infinite or excessively long expiration time. A maximum validity window (e.g., 30 days default, 90 days max) should be enforced by policy.
2. **Restricted High-Privilege Rights (POL-ASSUME-2):** Certain sensitive rights (e.g., `Admin`, `Delete`, `Execute`) should require explicit issuer authorization or minimum privilege levels before issuance or delegation.
3. **Suspension Governance (POL-ASSUME-3):** The security policy should specify whether suspended grants can be used during temporary grace periods or if suspension strictly acts as an immediate fail-closed gate.
4. **Cross-Subject Delegation Restrictions (POL-ASSUME-4):** Delegation across trust domain boundaries (e.g., from `kernel` to untrusted sandbox `agent:untrusted`) should be subject to explicit policy constraints.

---

## 3. Prior Art & Authoritative Standards

1. **RFC 7519 (JSON Web Token) & RFC 8693 (OAuth 2.0 Token Exchange):**
   - Temporal validation (`nbf`, `exp`) standards for delegated tokens.
   - Attenuation of scope and rights during token exchange.
2. **Capability-Based Computer Systems (Saltzer & Schroeder, 1975):**
   - Principle of Least Privilege.
   - Confinement Problem: Inability of an untrusted borrower to leak or escalate privileges beyond what was granted.
3. **Macaroons (Birgisson et al., 2014 - Cookies with Contextual Caveats):**
   - Monotonic attenuation: caveats can only restrict capabilities, never broaden them.
   - Proof of delegation via chained HMAC/cryptographic signatures.

---

## 4. Key Design Decisions for Specification

1. **Policy Architecture:** Implement dedicated `PepGrantSecurityPolicy` in `aiosh-core::pep_grant_security_policy` with JSON serde and file persistence.
2. **Enforcement Modes:** Support `Enforcing` (fail-closed, reject non-compliant grants), `AuditOnly` (log policy non-compliance but permit), and `Permissive` (allow with warnings).
3. **Constraints Enforced by Policy:**
   - `max_grant_lifetime_seconds`: Maximum allowable duration between `not_before` and `expires_at`.
   - `disallowed_delegation_rights`: Rights that can never be delegated (e.g., `Admin`).
   - `max_store_grants`: Upper bound on grant store cardinality.
   - `require_explicit_expiry`: Boolean flag prohibiting open-ended / non-expiring grants.
   - `restricted_subjects`: Blacklist or regex of subjects prohibited from receiving grants.

---

## 5. Acceptance Verification
- ✅ Authoritative standards cited (Saltzer & Schroeder, RFC 7519, Macaroons).
- ✅ Facts separated from assumptions.
- ✅ Decisions listed for upcoming specification (T-02262).
- ✅ Zero code modifications in this research phase.
