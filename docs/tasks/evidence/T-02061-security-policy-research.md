# Research: Capability Security Policy (T-02061)

## Executive Summary
This document establishes the theoretical foundations, system constraints, prior art, facts, assumptions, and key architectural decisions for the **Capability Security Policy** subsystem (`CAPSEC1..CAPSEC6`) within the AIOS Security Kernel.

---

## 1. Theoretical Foundations & Authoritative Sources

### 1.1 Object-Capability Model & Security Foundations
- **Dennis & Van Horn (1966)**, *"Programming Semantics for Multiprogrammed Computations"*: Defined the concept of capabilities as unforgeable tokens granting specific access rights to resources.
- **Saltzer & Schroeder (1975)**, *"The Protection of Information in Computer Systems"*:
  - *Principle of Least Privilege*: Every program and every privileged user of the system should operate using the least amount of privilege necessary to complete the job.
  - *Economy of Mechanism*: Keep the design as simple and small as possible.
  - *Fail-Safe Defaults*: Base access decisions on permission rather than exclusion (default deny).
  - *Complete Mediation*: Every access to every object must be checked for authority.
- **Miller, Yee, Shapiro (2003)**, *"Capability Myths Demolished"*:
  - Clarified that capability systems can enforce confinement and mandatory access control policies when augmented with membrane/attenuation policies.
- **seL4 Kernel Architecture (Klein et al., 2009)**:
  - Capability Spaces (CSpace) and Capability Derivation Trees (CDT). Explicit derivation depth tracking, untyped memory limits, and formal authority verification.

---

## 2. Existing System Architecture & Constraints

### 2.1 Existing Subsystems in AIOS
1. **Capability Data Model (`code/aiosh-rust/aiosh-core/src/capability.rs`)**:
   - `Capability`, `CapabilityRight` (`Read`, `Write`, `Execute`, `Delete`, `Admin`, `Delegate`), `CapabilityScope` (`Filesystem`, `Network`, `Tool`, `Process`, `Ipc`, `System`), `CapabilityConstraints`.
   - Invariants `CAP1..CAP6`: monotonic attenuation, temporal expiration, quota consumption.
2. **Capability Service (`code/aiosh-rust/aiosh-core/src/capability_service.rs`)**:
   - In-memory registry with indexing by subject and parent. Monotonic attenuation verification and cascade revocation.
3. **Existing Policy Implementations in AIOS**:
   - `BaseImageSecurityPolicy` (`base_image_policy.rs`): Enforcing/Audit/Permissive modes, validation rules, violation reports.
   - `DistroSecurityPolicy` (`distro_policy.rs`): Policy rules, violation diagnostics, severity levels.

---

## 3. Facts vs. Assumptions

### 3.1 Facts
1. **Fact 1**: A raw capability service enforces structural invariants (e.g. child rights $\subseteq$ parent rights), but does not by itself prevent a root issuer from granting dangerous rights (e.g., `/etc/shadow` write or unrestricted host network access) unless governed by an authoritative security policy.
2. **Fact 2**: High attenuation depth can lead to unbounded capability chains, recursion depth exhaustion, and denial-of-service in revocation tree walks.
3. **Fact 3**: In multi-tenant agent execution, certain subjects (e.g. untrusted third-party agents) must never receive `Admin` or `Delegate` rights, nor access sensitive paths (`/etc`, `/proc`, `/sys`, `/dev`, credential directories).
4. **Fact 4**: The system needs both `Enforcing` mode (blocks violations with errors) and `Audit` mode (logs violations but permits evaluation for debugging/staging), matching existing AIOS policies.

### 3.2 Assumptions
1. **Assumption 1**: A policy should govern both *issuance* (root capability creation) and *attenuation* (delegation to child capabilities).
2. **Assumption 2**: The policy should define a configurable maximum attenuation depth (default: 8, bounds: 1..=64) to prevent unbounded recursion while supporting realistic agent workflows.
3. **Assumption 3**: Sensitive filesystem paths can be expressed as prefix blacklists and prefix whitelists per subject tier (e.g. `system`, `service`, `untrusted_agent`).

---

## 4. Required Policy Capabilities & Invariants (`CAPSEC1..6`)

1. **`CAPSEC1` (Default Deny & Policy Modes)**: Policy evaluation must support `Enforcing`, `Audit`, and `Permissive` modes. In `Enforcing` mode, any rule violation halts issuance or attenuation.
2. **`CAPSEC2` (Attenuation Depth Bound)**: Derivation depth cannot exceed `max_attenuation_depth` (default 8, max 64).
3. **`CAPSEC3` (Sensitive Resource Restrictions)**: Scopes targeting sensitive resources (e.g., protected filesystem paths, restricted network destinations, privileged tools) must be explicitly denied unless the subject has authorized classification.
4. **`CAPSEC4` (Subject Disallowed Rights)**: Disallow specified rights (e.g. `Admin`, `Delegate`, `Delete`) for restricted or untrusted subject classes.
5. **`CAPSEC5` (Mandatory Temporal Bounds)**: When `require_temporal_bounds` is enabled, capabilities issued to non-system subjects must specify a valid `expires_at` within `max_validity_duration_seconds`.
6. **`CAPSEC6` (Auditability & Determinism)**: Every policy evaluation must return a deterministic `CapabilityPolicyVerdict` containing structured `CapabilityPolicyViolation` entries.

---

## 5. Decisions Needed Before Implementation

1. **Module Placement**: Create `code/aiosh-rust/aiosh-core/src/capability_policy.rs` and export it in `code/aiosh-rust/aiosh-core/src/lib.rs`.
2. **Integration with `CapabilityService`**:
   - `CapabilityService` can hold an `Option<CapabilitySecurityPolicy>` or a default `CapabilitySecurityPolicy`.
   - Provide `evaluate_issuance` and `evaluate_attenuation` methods on `CapabilitySecurityPolicy`.
   - Add optional policy check hooks to `CapabilityService::issue_root` and `CapabilityService::attenuate`.
3. **Serialization & Configuration**:
   - Support JSON serde for `CapabilitySecurityPolicy`, `CapabilityPolicyVerdict`, and `CapabilityPolicyViolation` so they can be loaded from config files or serialized over MCP.

---

## 6. Authoritative Citations
- Dennis, J. B., & Van Horn, E. C. (1966). Programming semantics for multiprogrammed computations. *Communications of the ACM*, 9(3), 143-155.
- Saltzer, J. H., & Schroeder, M. D. (1975). The protection of information in computer systems. *Proceedings of the IEEE*, 63(9), 1278-1308.
- Miller, M. S., Yee, K. P., & Shapiro, J. (2003). Capability myths demolished. *Technical Report SRL2003-02*, Johns Hopkins University.
