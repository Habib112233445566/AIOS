# T-01663: Security Policy Scaffold

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Create the module skeleton, data types, and function interfaces for the Kernel Module Management Security Policy in `code/aiosh-rust/aiosh-core/src/kernel_module_policy.rs` and register it in `lib.rs`.

## Deliverables
1. **Source File**: `code/aiosh-rust/aiosh-core/src/kernel_module_policy.rs`:
   - `pub const MAX_POLICY_FILE_BYTES: u64 = 65_536;`
   - `pub enum KernelModulePolicyMode`: `Enforcing`, `Audit`, `Permissive`.
   - `pub struct KernelModuleSecurityPolicy`: Configuration and rules fields.
   - `pub struct KernelModulePolicyViolation`: Detailed rule violation descriptor.
   - `pub struct KernelModulePolicyVerdict`: Comprehensive verdict and audit report.
   - Method signatures:
     - `validate(&self) -> Result<(), String>`
     - `evaluate_directive(&self, directive: &ModprobeDirective) -> KernelModulePolicyVerdict`
     - `evaluate_autoload(&self, module: &str) -> KernelModulePolicyVerdict`
     - `evaluate_store(&self, store: &KernelModuleStore) -> Vec<KernelModulePolicyVerdict>`
     - `from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>`
     - `from_source<F: Fn(&str) -> Option<String>>(lookup: F) -> Result<Self, String>`
2. **Module Export**: Added `pub mod kernel_module_policy;` to `code/aiosh-rust/aiosh-core/src/lib.rs`.

## Verification
- Verified compilation via `cargo check -p aiosh-core`.
- Artifacts generated in `docs/tasks/evidence/T-01663-security-policy-scaffold.md` and `T-01663-scaffold.md`.
