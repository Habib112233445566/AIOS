# T-01372: Init & Service Supervision / Observability - Specification

## 1. Scope & Objective
This specification establishes the interface, data schemas, mathematical invariants, error handling, and operational contracts for the **AIOS Init & Service Supervision Observability Subsystem** (`code/aiosh-rust/aiosh-core/src/service_observability.rs`).

The subsystem provides operators and autonomous AI agents with holistic, high-performance telemetry regarding service inventory, execution states, startup enablement modes, health telemetry, restart loop accounting, dependency complexity, and security policy compliance in a deterministic, read-only query.

---

## 2. Invariants Specification (SO1..SO6)

| ID | Invariant Title | Formal Definition & Enforcement Rules |
|---|---|---|
| **SO1** | Inventory Completeness | $\text{total\_services} = \sum \text{state\_breakdown.values()} = \sum \text{startup\_mode\_breakdown.values()} = \sum \text{service\_type\_breakdown.values()} = \sum \text{restart\_policy\_breakdown.values()}$. If store is empty, $\text{total\_services} = 0$. |
| **SO2** | Categorical Distributions | Breakdown maps use canonical string representations and sorted keys (`BTreeMap<String, usize>`) for deterministic JSON serialization across `state_breakdown`, `startup_mode_breakdown`, `service_type_breakdown`, and `restart_policy_breakdown`. |
| **SO3** | Health & Restart Telemetry | Aggregates operational health: $\text{healthy\_count} = \sum \{ 1 \mid \text{s.health.healthy} == \text{true} \}$, $\text{unhealthy\_count} = \sum \{ 1 \mid \text{s.health.healthy} == \text{false} \lor \text{s.state} == \text{Failed} \}$, $\text{total\_restarts} = \sum \text{s.health.restarts}$, and collects sorted list $\text{failed\_services}$. |
| **SO4** | Dependency Histogram | Partitions service dependency counts into four distinct, exhaustive buckets: `"0"`, `"1-2"`, `"3-5"`, and `"6+"`. The sum of all bucket counts strictly equals $\text{total\_services}$. |
| **SO5** | Security Policy Compliance | Evaluates all registered services against `ServiceSecurityPolicy`. Computes: $\text{policy\_compliant\_count}$, $\text{policy\_violations\_count}$, and $\text{prohibited\_services\_found}$ (sorted list of unique prohibited service names). |
| **SO6** | Read-Only Determinism | Generation is strictly read-only; no state mutations occur on `ServiceStore` or security configuration. Emits ISO-8601 UTC timestamp (`generated_at`). |

---

## 3. Data Schema & Rust Structures

Module: `code/aiosh-rust/aiosh-core/src/service_observability.rs`

```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::service_service::ServiceStore;
use crate::service_policy::ServiceSecurityPolicy;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceObservabilityReport {
    pub total_services: usize,
    pub state_breakdown: BTreeMap<String, usize>,
    pub startup_mode_breakdown: BTreeMap<String, usize>,
    pub service_type_breakdown: BTreeMap<String, usize>,
    pub restart_policy_breakdown: BTreeMap<String, usize>,
    pub healthy_count: usize,
    pub unhealthy_count: usize,
    pub total_restarts: u32,
    pub failed_services: Vec<String>,
    pub dependency_distribution: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_services_found: Vec<String>,
    pub generated_at: String,
}

impl ServiceObservabilityReport {
    pub fn generate(
        store: &ServiceStore,
        policy_opt: Option<&ServiceSecurityPolicy>,
    ) -> Self;

    pub fn to_json_pretty(&self) -> Result<String, String>;

    pub fn generate_from_paths<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
        store_path_opt: Option<P>,
        policy_path_opt: Option<Q>,
    ) -> Result<Self, String>;
}
```

---

## 4. Operator CLI Interface Specification

Subcommand: `aiosh service stats` (alias: `aiosh service observability`)

### Syntax:
```bash
aiosh service stats [--store <path>] [--policy <path>] [--json]
```

### Flags:
- `--store <path>`: Optional path to JSON service store state file. Defaults to active store.
- `--policy <path>`: Optional path to service policy configuration file for compliance evaluation. Defaults to standard policy resolution hierarchy.
- `--json`: Format output as indented JSON. Defaults to formatted human-readable summary.

### Exit Codes:
- `0`: Success, metrics report generated and output.
- `1`: I/O error reading store or configuration file.
- `2`: Invalid command-line arguments or syntax.

---

## 5. Model Context Protocol (MCP) Interface Specification

Tool Name: `aios.service.stats`

### Input Schema:
```json
{
  "type": "object",
  "properties": {
    "store_path": {
      "type": "string",
      "description": "Optional path to service store state file"
    },
    "policy_path": {
      "type": "string",
      "description": "Optional path to custom service policy JSON file"
    },
    "grant_id": {
      "type": "string",
      "description": "Optional PEP authorization grant ID"
    }
  },
  "additionalProperties": false
}
```

### Output Shape:
```json
{
  "ok": true,
  "tool": "aios.service.stats",
  "report": {
    "total_services": 4,
    "state_breakdown": {
      "active": 3,
      "inactive": 1
    },
    "startup_mode_breakdown": {
      "enabled": 3,
      "masked": 1
    },
    "service_type_breakdown": {
      "simple": 4
    },
    "restart_policy_breakdown": {
      "always": 4
    },
    "healthy_count": 3,
    "unhealthy_count": 0,
    "total_restarts": 0,
    "failed_services": [],
    "dependency_distribution": {
      "0": 2,
      "1-2": 2,
      "3-5": 0,
      "6+": 0
    },
    "policy_compliant_count": 4,
    "policy_violations_count": 0,
    "prohibited_services_found": [],
    "generated_at": "2026-09-06T00:00:00Z"
  }
}
```

---

## 6. Audit & Logging Effects
Every invocation of `aiosh service stats` and `aios.service.stats` triggers an audit row emission to the SQLite WAL ring (`classify_and_emit` / `dispatch::recorded_call`), logging:
- Subsystem: `service`
- Action: `stats`
- Actor: `operator` or autonomous agent
- Parameters: `store_path`, `policy_path`
- Status: `success` or `failure`
- Hash chain sequence ID in SQLite WAL `audit.db`
