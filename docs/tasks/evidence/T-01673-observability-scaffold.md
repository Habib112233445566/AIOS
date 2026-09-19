# T-01673: Observability Scaffold

## Sub-Epic
Kernel Module Management / Observability

## Objective
Create the module skeleton, data types, helper functions, and report generator interface for Kernel Module Management Observability in `code/aiosh-rust/aiosh-core/src/kernel_module_observability.rs` and register it in `lib.rs`.

## Deliverables
1. **Source File**: `code/aiosh-rust/aiosh-core/src/kernel_module_observability.rs`:
   - `pub fn module_state_to_str(state: ModuleState) -> &'static str`
   - `pub fn rule_type_to_str(rule: &ModprobeRule) -> &'static str`
   - `pub struct KernelModuleObservabilityReport`
   - `KernelModuleObservabilityReport::generate` method signature
2. **Library Registration**: Added `pub mod kernel_module_observability;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## Verification
- Verified compilation via `cargo check -p aiosh-core`.
- Artifacts: `docs/tasks/evidence/T-01673-observability-scaffold.md` and `T-01673-scaffold.md`.
