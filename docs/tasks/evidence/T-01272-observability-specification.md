# T-01272: Package Management / Observability - Specification

## 1. Scope & Objective
This specification defines the exact interface, data schemas, mathematical invariants, error modes, and operational guarantees for the **AIOS Package Management Observability Subsystem**.

The subsystem enables autonomous agents, continuous testing runners, and human operators to inspect system package health, inventory distribution, storage consumption, and security compliance in a single, high-performance, deterministic query.

---

## 2. Invariants Specification (PO1..PO6)

| ID | Invariant Title | Formal Definition & Enforcement Rules |
|---|---|---|
| **PO1** | Inventory Completeness | $\text{total\_packages} = \sum \text{state\_breakdown.values()} = \sum \text{format\_breakdown.values()} = \sum \text{architecture\_breakdown.values()}$. If store is empty, $\text{total\_packages} = 0$. |
| **PO2** | Categorical Distributions | Breakdown maps (`state_breakdown`, `format_breakdown`, `architecture_breakdown`) use canonical string representations and sorted keys (`BTreeMap<String, usize>`) for deterministic serialization. |
| **PO3** | Footprint Telemetry | $\text{total\_installed\_size\_bytes} = \sum \{ \text{p.installed\_size\_bytes} \mid \text{p.state} \in \{\text{Installed}, \text{Upgraded}\} \}$. $\text{average\_package\_size\_bytes} = \lfloor \frac{\sum \text{p.installed\_size\_bytes}}{\text{total\_packages}} \rfloor$ (or $0$ if $\text{total\_packages} = 0$). |
| **PO4** | Dependency Histogram | Partitions package dependency counts into four distinct, exhaustive buckets: `"0"`, `"1-5"`, `"6-10"`, and `"11+"`. The sum of all bucket counts equals $\text{total\_packages}$. |
| **PO5** | Security Policy Compliance | Evaluates all packages in the store against `PackageSecurityPolicy`. Computes: $\text{policy\_compliant\_count}$, $\text{policy\_violations\_count}$, and $\text{prohibited\_packages\_found}$ (sorted list of unique names). |
| **PO6** | Read-Only Determinism | Generation is strictly read-only; no mutation occurs on `PackageStore` or security configuration. Emits ISO-8601 UTC timestamp (`generated_at`). |

---

## 3. Data Schema & Rust Structures

Module: `code/aiosh-rust/aiosh-core/src/package_observability.rs`

```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::package_service::PackageStore;
use crate::package_policy::PackageSecurityPolicy;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageObservabilityReport {
    pub total_packages: usize,
    pub state_breakdown: BTreeMap<String, usize>,
    pub format_breakdown: BTreeMap<String, usize>,
    pub architecture_breakdown: BTreeMap<String, usize>,
    pub total_installed_size_bytes: u64,
    pub average_package_size_bytes: u64,
    pub dependency_distribution: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_packages_found: Vec<String>,
    pub generated_at: String,
}

impl PackageObservabilityReport {
    pub fn generate(
        store: &PackageStore,
        policy_opt: Option<&PackageSecurityPolicy>,
    ) -> Self;
}
```

---

## 4. Operator CLI Interface Specification

Subcommand: `aiosh package stats`

### Syntax:
```bash
aiosh package stats [--store <path>] [--config <path>] [--json]
```

### Flags:
- `--store <path>`: Optional path to JSON package store state file. Defaults to active store.
- `--config <path>`: Optional path to policy configuration file for compliance evaluation. Defaults to standard policy resolution hierarchy.
- `--json`: Format output as indented JSON. Defaults to formatted human-readable summary.

### Exit Codes:
- `0`: Success, metrics report generated and output.
- `1`: I/O error reading store or configuration file.
- `2`: Invalid command-line arguments or syntax.

---

## 5. Model Context Protocol (MCP) Interface Specification

Tool Name: `aios.package.stats`

### Input Schema:
```json
{
  "type": "object",
  "properties": {
    "store_path": {
      "type": "string",
      "description": "Optional path to package store state file"
    },
    "config_path": {
      "type": "string",
      "description": "Optional path to custom policy JSON configuration file"
    }
  }
}
```

### Output Shape:
```json
{
  "ok": true,
  "tool": "aios.package.stats",
  "report": {
    "total_packages": 8,
    "state_breakdown": {
      "available": 6,
      "installed": 2
    },
    "format_breakdown": {
      "deb": 5,
      "tar_gz": 2,
      "raw_binary": 1
    },
    "architecture_breakdown": {
      "amd64": 5,
      "x86_64": 3
    },
    "total_installed_size_bytes": 14194304,
    "average_package_size_bytes": 4875000,
    "dependency_distribution": {
      "0": 5,
      "1-5": 3,
      "6-10": 0,
      "11+": 0
    },
    "policy_compliant_count": 8,
    "policy_violations_count": 0,
    "prohibited_packages_found": [],
    "generated_at": "2026-09-04T00:00:00Z"
  }
}
```

---

## 6. Audit & Logging Effects
Every invocation of `aiosh package stats` and `aios.package.stats` triggers a recorded call via the PEP ring buffer (`dispatch::recorded_call`), logging:
- Tool name: `aios.package.stats`
- Actor: `operator` or `agent:mcp@aiosh-mcp`
- Execution parameters: `store_path`, `config_path`
- Result code: `ok: true/false`
- Hash chain sequence ID in SQLite WAL `audit.db`
