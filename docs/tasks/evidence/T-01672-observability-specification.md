# T-01672: Observability Specification

## Sub-Epic
Kernel Module Management / Observability

## Objective
Specify the exact data structures, telemetry metrics, helper functions, and report generation contract for Kernel Module Management Observability (`KernelModuleObservabilityReport`).

## 1. Scope & Placement
- **Location**: `code/aiosh-rust/aiosh-core/src/kernel_module_observability.rs`
- **Export**: Exported via `code/aiosh-rust/aiosh-core/src/lib.rs`
- **Reused Interfaces**:
  - `crate::kernel_module::{ModprobeRule, ModuleInfo, ModuleState}`
  - `crate::kernel_module_service::{KernelModuleService, KernelModuleStore}`
  - `crate::kernel_module_policy::KernelModuleSecurityPolicy`
- **New Interfaces (AIOS-Specific)**:
  - `struct KernelModuleObservabilityReport`
  - `pub fn module_state_to_str(state: ModuleState) -> &'static str`
  - `pub fn rule_type_to_str(rule: &ModprobeRule) -> &'static str`

## 2. Invariants (KO1..KO6)
| Invariant | Title | Description |
|---|---|---|
| **KO1** | Accurate Module Aggregation | Accurately aggregates total count of loaded modules, store rules, and autoload modules without double counting. |
| **KO2** | Categorical Distributions | Provides deterministic breakdowns by module runtime state (`live`, `loading`, `unloading`) and modprobe rule type (`blacklist`, `alias`, `options`, `install`, `remove`, `softdep`). |
| **KO3** | Memory Footprint Telemetry | Sums `size_bytes` across all loaded modules, reporting kernel memory usage in ring 0. |
| **KO4** | Dependency & Ref Count Distribution | Groups module reference counts into buckets: `"0"`, `"1-2"`, `"3-5"`, `"6+"`. |
| **KO5** | Security Policy & Compliance | Computes compliance counts and violation totals against `KernelModuleSecurityPolicy`, identifying any configured prohibited modules. |
| **KO6** | Deterministic Serialization | Uses `BTreeMap` to ensure stable, sorted JSON keys across platforms. |

## 3. Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelModuleObservabilityReport {
    pub total_loaded_modules: usize,
    pub total_memory_bytes: u64,
    pub state_breakdown: BTreeMap<String, usize>,
    pub store_rules_count: usize,
    pub rule_type_breakdown: BTreeMap<String, usize>,
    pub autoload_modules_count: usize,
    pub ref_count_distribution: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_modules_configured: Vec<String>,
    pub protected_modules_configured: Vec<String>,
    pub generated_at: String,
}
```

## 4. API Contract

```rust
impl KernelModuleObservabilityReport {
    /// Generates an observability report from the provided service and optional security policy.
    pub fn generate(
        service: &KernelModuleService,
        policy_opt: Option<&KernelModuleSecurityPolicy>,
    ) -> Self;
}
```

- If `policy_opt` is `None`, the default `KernelModuleSecurityPolicy` is used.
- If live module inspection fails or `/proc/modules` is unavailable, `total_loaded_modules` and `total_memory_bytes` default to 0 without panicking; store rules and autoload metrics are reported from the store.
