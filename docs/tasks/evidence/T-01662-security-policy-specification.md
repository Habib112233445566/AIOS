# T-01662: Security Policy Specification

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Specify the exact formal contract, data structures, evaluation algorithms, error conditions, and audit effects for the Kernel Module Management Security Policy (`KernelModuleSecurityPolicy`).

## 1. Scope & Placement
- **Location**: `code/aiosh-rust/aiosh-core/src/kernel_module_policy.rs`
- **Export**: Exported via `code/aiosh-rust/aiosh-core/src/lib.rs`
- **Reused Interfaces**:
  - `crate::kernel_module::{ModprobeDirective, ModprobeAction, validate_module_name, MAX_MODULE_NAME_LEN}`
  - `crate::kernel_module_service::KernelModuleStore`
- **New Interfaces (AIOS-Specific)**:
  - `enum KernelModulePolicyMode`: `Enforcing`, `Audit`, `Permissive`
  - `struct KernelModuleSecurityPolicy`: Configuration rules and evaluator
  - `struct KernelModulePolicyViolation`: Individual rule failure record
  - `struct KernelModulePolicyVerdict`: Aggregate decision and audit envelope

## 2. Invariants (SP-KM1 .. SP-KM6)
| Invariant | Title | Description | Fatal? |
|---|---|---|---|
| **SP-KM1** | Parameter & Identifier Hygiene | Names must match `^[a-zA-Z0-9_]+$` ($\le 64$ chars). Parameter keys must be alphanumeric/underscore ($\le 128$ chars). Parameter values must not contain shell metacharacters or control characters ($\le 1024$ chars). | Yes |
| **SP-KM2** | Mandatory Blacklist Enforcement | Modules in `prohibited_modules` (e.g. `cramfs`, `dccp`, `firewire-core`) must never be added to autoload or configured with options. | Yes |
| **SP-KM3** | Protected Module Guard | Modules in `protected_modules` (e.g. `ext4`, `xfs`, `overlay`, `crypto`, `dm_mod`) must never be blacklisted or disabled via install `/bin/false`. | Yes |
| **SP-KM4** | Install Command Sanitization | `install` directives may only execute binaries explicitly listed in `allowed_install_commands` (default: `/bin/true`, `/bin/false`, `/usr/bin/true`, `/usr/bin/false`). Arbitrary commands are rejected. | Yes |
| **SP-KM5** | Parameter Whitelist / Blacklist | Specific parameter keys in `disallowed_parameter_keys` or values matching dangerous patterns (e.g., command execution, binary paths) are blocked. | Yes |
| **SP-KM6** | Tri-State Evaluation & Size Ceiling | Policy files must not exceed `MAX_POLICY_FILE_BYTES` (64 KiB). Evaluation follows `Enforcing`, `Audit`, or `Permissive` semantics. | Enforcing: all fatal block; Audit: none block; Permissive: only SP-KM3/SP-KM4 block. |

## 3. Data Structures

```rust
pub const MAX_POLICY_FILE_BYTES: u64 = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelModulePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelModuleSecurityPolicy {
    pub mode: KernelModulePolicyMode,
    pub prohibited_modules: Vec<String>,
    pub protected_modules: Vec<String>,
    pub allowed_install_commands: Vec<String>,
    pub disallowed_parameter_keys: Vec<String>,
    pub disallowed_parameter_patterns: Vec<String>,
    pub max_parameter_value_len: usize,
    pub max_rules_per_module: usize,
    pub enforce_strict_naming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelModulePolicyViolation {
    pub rule_id: String,
    pub module_name: String,
    pub description: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelModulePolicyVerdict {
    pub module_name: String,
    pub allowed: bool,
    pub mode: KernelModulePolicyMode,
    pub violations: Vec<KernelModulePolicyViolation>,
    pub evaluated_at: String,
}
```

## 4. Evaluator API Contract

1. `validate(&self) -> Result<(), String>`:
   - Verifies limits: `prohibited_modules.len() <= 1024`, `protected_modules.len() <= 256`.
   - Verifies disjointness: `prohibited_modules` and `protected_modules` MUST NOT overlap. If any module appears in both, returns `Err("invariant SP1 violated: module '<name>' is both prohibited and protected")`.
   - Verifies `allowed_install_commands`: all must be absolute paths without whitespace or traversal.
2. `evaluate_directive(&self, directive: &ModprobeDirective) -> KernelModulePolicyVerdict`:
   - Evaluates a single `ModprobeDirective` against SP-KM1..SP-KM5.
   - Computes `allowed` based on `self.mode` and recorded violations.
3. `evaluate_autoload(&self, module: &str) -> KernelModulePolicyVerdict`:
   - Checks whether `module` violates SP-KM1 or is present in `prohibited_modules` (SP-KM2).
4. `evaluate_store(&self, store: &KernelModuleStore) -> Vec<KernelModulePolicyVerdict>`:
   - Evaluates all directives in `store.directives` and all modules in `store.autoload_modules`.
5. `from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>`:
   - Opens policy file bounded by `MAX_POLICY_FILE_BYTES` (64 KiB) and deserializes.
6. `from_source<F: Fn(&str) -> Option<String>>(lookup: F) -> Result<Self, String>`:
   - Reads environment overrides (e.g. `AIOS_KERNEL_MODULE_POLICY_MODE`).

## 5. Audit & Error Semantics
Every evaluation produces a structured `KernelModulePolicyVerdict` containing an immutable record of all violations discovered, whether fatal or non-fatal, and the precise policy mode active at evaluation time.
