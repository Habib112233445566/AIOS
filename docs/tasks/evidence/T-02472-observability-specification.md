# T-02472: Sandbox Enforcement Observability Specification

## 1. Specification Overview
This specification details the observability metrics, report structure, and validation invariants for Sandbox Enforcement telemetry.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `SANDBOXOBS1` | **Temporal Integrity** | Every observability report is timestamped with `generated_at_utc` in RFC 3339 format. |
| `SANDBOXOBS2` | **Sanitized Telemetry** | All text fields (profile names, commands, errors) are stripped of ANSI/control characters and clamped to 256 characters. |
| `SANDBOXOBS3` | **Bounded Cardinality** | Distribution maps (`executions_by_outcome`, `executions_by_profile`) are capped at 128 entries to prevent unbounded memory growth. |
| `SANDBOXOBS4` | **Non-Intrusive Sampling** | Telemetry aggregation is strictly read-only and acquires shared read transactions without blocking active sandboxed processes. |
| `SANDBOXOBS5` | **Health Diagnostics** | Evaluates whether registered factory profiles exist, whether host capabilities match platform expectations, and reports `is_healthy: bool`. |
| `SANDBOXOBS6` | **Cross-Substrate Parity** | Telemetry report serializes to canonical JSON schema across CLI (`aiosh sandbox stats`) and MCP (`aios.sandbox.stats`). |

---

## 3. Data Structures & API Contract

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SandboxObservabilityReport {
    pub generated_at_utc: String,
    pub total_profiles_registered: usize,
    pub total_executions_recorded: usize,
    pub executions_by_outcome: HashMap<String, usize>,
    pub executions_by_profile: HashMap<String, usize>,
    pub policy_mode: String,
    pub host_capabilities: HostSandboxCapabilities,
    pub is_healthy: bool,
}

impl SandboxObservabilityReport {
    pub fn generate(service: &SandboxService) -> Result<Self, String>;
    pub fn validate(&self) -> Result<(), String>;
}
```
