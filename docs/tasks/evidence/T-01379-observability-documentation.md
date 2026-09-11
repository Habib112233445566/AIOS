# T-01379: Init & Service Supervision / Observability - Documentation

## Metadata
- **Task ID:** `T-01379`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Observability
- **Status:** Complete
- **Date:** 2026-09-09

---

## 1. Overview & Shipped Artifacts
This document establishes operational and developer documentation for the Init & Service Supervision Observability subsystem (`ServiceObservabilityReport`, `service_observability.rs`, CLI command `aiosh service stats`, and MCP tool `aios.service.stats`), implementing criteria `SO1..SO6`.

### Shipped Invariants & Features
- **SO1 (Inventory Completeness)**: Exact tracking of total supervised services; mathematical conservation across all distribution buckets.
- **SO2 (Categorical Distributions)**: Breakdowns across service state (`active`, `inactive`, `failed`, etc.), startup mode (`enabled`, `disabled`, `masked`, `static`), service type (`simple`, `forking`, `oneshot`, `notify`), and restart policy (`always`, `on_failure`, `no`).
- **SO3 (Health & Flapping Telemetry)**: Precise counts for healthy vs. unhealthy services, names of failed services, and process restart counters aggregated with saturation arithmetic.
- **SO4 (Dependency Histogram)**: Bounded partition buckets (`"0"`, `"1-2"`, `"3-5"`, `"6+"`) providing $O(1)$ memory usage regardless of dependency graph depth.
- **SO5 (Security Policy Compliance)**: Cross-evaluation with `ServiceSecurityPolicy` reporting compliant services, violation counts, and prohibited service detections (e.g. `telnet.service`).
- **SO6 (Deterministic Formatting & Audit Trail)**: Stable ISO timestamps, standardized JSON result envelopes, and immutable SHA-256 hash-chained audit logging to SQLite WAL.

---

## 2. Copy-Pasteable Invocation Examples

### 1. Operator CLI Surface (`aiosh service stats`)

#### A. Human-Readable Telemetry Overview
```bash
aiosh service stats
```
*Output:*
```text
AIOS Init & Service Supervision Observability Report:
  Total Services:         5
  Healthy Services:       5
  Unhealthy Services:     0
  Total Process Restarts: 0
  Policy Compliant:       5
  Policy Violations:      0
  State Breakdown:
    active         4
    inactive       1
  Startup Mode Breakdown:
    disabled       1
    enabled        4
  Service Type Breakdown:
    simple         5
  Restart Policy Breakdown:
    always         3
    on_failure     2
  Dependency Distribution:
    0              3
    1-2            2
    3-5            0
    6+             0
```

#### B. Structured JSON Output
```bash
aiosh service stats --json
```
*Output:*
```json
{
  "code": 0,
  "data": {
    "total_services": 5,
    "healthy_count": 5,
    "unhealthy_count": 0,
    "total_restarts": 0,
    "failed_services": [],
    "policy_compliant_count": 5,
    "policy_violations_count": 0,
    "prohibited_services_found": [],
    "state_breakdown": {
      "active": 4,
      "inactive": 1
    },
    "startup_mode_breakdown": {
      "disabled": 1,
      "enabled": 4
    },
    "service_type_breakdown": {
      "simple": 5
    },
    "restart_policy_breakdown": {
      "always": 3,
      "on_failure": 2
    },
    "dependency_distribution": {
      "0": 3,
      "1-2": 2,
      "3-5": 0,
      "6+": 0
    },
    "generated_at": "2026-09-06T00:00:00Z"
  },
  "error": null
}
```

#### C. Custom Store & Policy Evaluation
```bash
aiosh service stats --store .aios/service_store.json --policy .aios/service_policy.json --json
```

---

### 2. Autonomous Agent MCP Tool Calls (`aios.service.stats`)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.service.stats",
    "arguments": {
      "store_path": ".aios/service_store.json"
    }
  }
}
```

---

## 3. Constraints & Known Limitations
1. **Path Bounds**: Custom `store_path` and `policy_path` strings must not exceed 1024 characters and must not contain ASCII control characters.
2. **File Size Caps**: Store files are bounded at 10 MiB; policy files are bounded at 64 KiB.
3. **Read-Only Guarantee**: Observability queries cannot alter service state, mutate store contents, or trigger lifecycle changes.
4. **Audit Requirement**: Every query writes an immutable SHA-256 hash-chained audit event to the SQLite WAL ring buffer.

---

## 4. Evidence Trail & Sub-Task References
- **Research (T-01371)**: [`T-01371-observability-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01371-observability-research.md)
- **Specification (T-01372)**: [`T-01372-observability-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01372-observability-specification.md)
- **Scaffold (T-01373)**: [`T-01373-observability-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01373-observability-scaffold.md)
- **Implementation (T-01374)**: [`T-01374-observability-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01374-observability-implementation.md)
- **Unit Test (T-01375)**: [`T-01375-observability-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01375-observability-unit-test.md)
- **Integration (T-01376)**: [`T-01376-observability-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01376-observability-integration.md)
- **Security Review (T-01377)**: [`T-01377-observability-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01377-observability-security-review.md)
- **Hardening (T-01378)**: [`T-01378-observability-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01378-observability-hardening.md)
