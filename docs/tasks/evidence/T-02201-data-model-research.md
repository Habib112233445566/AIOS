# Task Evidence: T-02201 (Grant Lifecycle / data model: Research)

## 1. Context & Objectives
This task launches the **Grant Lifecycle Epic** (`T-02201..T-02300`) within **Phase 2 — Security Kernel & PEP Fabric**.
The objective is to establish authoritative facts, architectural constraints, prior art, and core data model invariants for managed authorization grants that govern agent and operator capabilities in AIOS.

## 2. Prior Art & Authoritative Sources
1. **RFC 6749 & RFC 7519 (OAuth 2.0 & JWT)**:
   - Authorization grants represent abstract credentials representing resource owner authorization.
   - Structured claims: `iss` (issuer), `sub` (subject), `aud` (audience), `exp` (expiration time), `nbf` (not before), `iat` (issued at), and `jti` (unique token ID).
2. **Birgisson et al., "Macaroons: Cookies with Contextual Caveats for Decentralized Authorization in the Cloud" (NDSS 2014)**:
   - Contextual caveats, monotonic attenuation, and cryptographic proof of delegation without server state sharing.
3. **NIST SP 800-162 & XACML 3.0 Delegation and Attribute Profiles**:
   - Explicit lifecycle state progression: Request -> Pending -> Active -> Suspended -> Revoked -> Expired.
   - Guarded administrative delegation: A subject cannot grant rights they do not possess.
4. **Existing AIOS Subsystems**:
   - `code/aiosh-rust/aiosh-core/src/capability.rs`: Established base `CapabilityRight`, `CapabilityScope`, and `CapabilityConstraints`.
   - `code/aiosh-rust/aiosh-core/src/pep_decision.rs`: Authoritative policy decision structures (`PepRequest`, `PepDecision`).
   - `code/aiosh-rust/aiosh-core/src/audit.rs`: Hash-chained SQLite ring audit trail.

## 3. Fact vs Assumption
| Topic | Fact | Assumption |
|-------|------|------------|
| **Lifecycle States** | Real grants require explicit states beyond binary active/revoked: `Pending`, `Active`, `Suspended`, `Revoked`, `Expired`. | Transitions must be strictly monotonic or validated via a formal state machine to prevent resurrected revoked grants. |
| **Delegation Depth** | Unconstrained delegation causes privilege propagation cascades. | Enforcing `max_delegation_depth` (e.g. max 4 hops) and monotonic scope attenuation prevents privilege escalation. |
| **Audit Requirement** | Invariant ADR-0035 / PEP: every grant mutation must record an audit row. | Grant state transitions (activation, suspension, revocation) are consequential security events. |
| **Persistence Schema** | Grant storage must serialize losslessly to canonical JSON and interface with SQLite audit buffer. | Standard JSON schema with typed error envelopes ensures cross-substrate parity with MCP and CLI. |

## 4. Unknowns & Architectural Decisions Needed
1. **State Machine Transitions**:
   - Allowed transitions:
     - `Requested` -> `Active` | `Rejected`
     - `Active` -> `Suspended` | `Revoked` | `Expired`
     - `Suspended` -> `Active` | `Revoked` | `Expired`
     - `Revoked` -> Terminal state (cannot transition)
     - `Expired` -> Terminal state (cannot transition)
   - Decision: Implement `can_transition_to(&self, target: GrantState) -> bool` enforcing terminal state invariants.
2. **Attenuated Delegation (`parent_grant_id`)**:
   - When a grant is created from a parent grant, its rights must be a subset of the parent's rights, its lifetime $\le$ parent's expiration, and its delegation depth $\le$ parent's depth minus 1.
3. **Revocation Cascading**:
   - Revoking a parent grant must invalidate all descendant child grants.

## 5. Acceptance Confirmation
- [x] Authoritative sources collected and cited.
- [x] Facts strictly separated from assumptions.
- [x] Architectural decisions and state machine transitions defined.
- [x] Zero code changed in this research task.
