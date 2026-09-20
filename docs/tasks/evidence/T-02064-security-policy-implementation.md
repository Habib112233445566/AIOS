# Evidence: T-02064 - security policy: Implementation

## Task Overview
- **Task ID**: `T-02064`
- **Sub-Epic**: Sub-Epic 7: Security Policy (`T-02061`..`T-02070`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Implement minimal working behavior for Capability Security Policy (`CAPSEC1..CAPSEC6`) and integrate with `CapabilityService`.

## Implementation Details
1. **`CapabilitySecurityPolicy` (`code/aiosh-rust/aiosh-core/src/capability_policy.rs`)**:
   - `validate(&self) -> Result<(), String>`:
     - Enforces $1 \le \text{max\_attenuation\_depth} \le 128$.
     - Rejects empty strings and path traversals (`..`) in `prohibited_path_prefixes`.
     - Validates positive `max_validity_duration_seconds`.
   - `evaluate_issuance(...) -> CapabilityPolicyVerdict`:
     - Evaluates prohibited path prefixes (`/etc`, `/proc`, `/sys`, `/dev`, `/root`, `/var/run`, `C:\Windows`, etc.).
     - Evaluates prohibited network hosts (`169.254.169.254`, `metadata.google.internal`).
     - Evaluates prohibited tools (`raw_syscall`, `kernel_module_load`, `reboot`).
     - Evaluates disallowed rights per subject prefix (`untrusted:*`, `guest:*`).
     - Evaluates temporal constraints (`require_temporal_bounds`, `max_validity_duration_seconds`).
     - Evaluates quota ceilings (`max_invocations_ceiling`, `max_bytes_ceiling`).
   - `evaluate_attenuation(...) -> CapabilityPolicyVerdict`:
     - Invokes issuance checks on child capability parameters.
     - Enforces derivation depth bound (`current_depth <= max_attenuation_depth`).
2. **`CapabilityService` Integration (`code/aiosh-rust/aiosh-core/src/capability_service.rs`)**:
   - Added `policy: CapabilitySecurityPolicy` to `CapabilityService`.
   - Added builder and accessor methods: `with_policy()`, `policy()`, `policy_mut()`, and `get_derivation_depth()`.
   - Enforced `evaluate_issuance` in `issue_root_capability`.
   - Enforced `evaluate_attenuation` in `attenuate_capability`.

## Compilation & Verification
- `cargo check -p aiosh-core` passed with zero errors and zero warnings.
