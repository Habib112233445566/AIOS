# Evidence: T-02063 - security policy: Scaffold

## Task Overview
- **Task ID**: `T-02063`
- **Sub-Epic**: Sub-Epic 7: Security Policy (`T-02061`..`T-02070`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Create module skeleton and interfaces for Capability Security Policy.

## Scaffolding Summary
- Created `code/aiosh-rust/aiosh-core/src/capability_policy.rs`:
  - `CapabilityPolicyMode` enum (`Enforcing`, `Audit`, `Permissive`).
  - `CapabilityPolicyViolation` struct (`rule_id`, `description`, `fatal`).
  - `CapabilityPolicyVerdict` struct (`allowed`, `mode`, `violations`, `evaluated_at`).
  - `CapabilitySecurityPolicy` struct:
    - `mode: CapabilityPolicyMode`
    - `max_attenuation_depth: usize`
    - `disallowed_rights_by_subject_prefix: HashMap<String, Vec<CapabilityRight>>`
    - `prohibited_path_prefixes: Vec<String>`
    - `prohibited_network_hosts: Vec<String>`
    - `prohibited_tools: Vec<String>`
    - `require_temporal_bounds: bool`
    - `max_validity_duration_seconds: Option<i64>`
    - `max_invocations_ceiling: Option<u64>`
    - `max_bytes_ceiling: Option<u64>`
  - Typed method signatures:
    - `validate(&self) -> Result<(), String>`
    - `evaluate_issuance(&self, issuer: &str, subject: &str, scope: &CapabilityScope, rights: &[CapabilityRight], constraints: &CapabilityConstraints) -> CapabilityPolicyVerdict`
    - `evaluate_attenuation(&self, parent: &Capability, new_subject: &str, scope: &CapabilityScope, rights: &[CapabilityRight], constraints: &CapabilityConstraints, current_depth: usize) -> CapabilityPolicyVerdict`
- Registered `pub mod capability_policy;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Validated compilation with `cargo check -p aiosh-core` (Finished successfully in 40.03s).
