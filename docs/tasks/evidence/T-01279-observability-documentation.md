# T-01279: Package Management / Observability - Documentation

## 1. Overview & Architectural Goals
The **Package Management Observability Subsystem** (`PackageObservabilityReport`, `aiosh package stats`, and `aios.package.stats`) provides comprehensive inventory telemetry, storage footprint analytics, dependency structure distributions, and security policy compliance metrics across the AIOS package ecosystem.

```
+-------------------------------------------------------------------------+
|                         AIOS Operational Layer                         |
|    CLI (`aiosh package stats`)    |    MCP Tool (`aios.package.stats`)  |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|              PackageObservabilityReport Engine (`PO1..PO6`)             |
|  - Invariant PO1: Total Package & Inventory Completeness                |
|  - Invariant PO2: State, Format, and Architecture Breakdowns            |
|  - Invariant PO3: Total Installed Footprint & Average Package Sizing    |
|  - Invariant PO4: Dependency Distribution Histogram Bounding            |
|  - Invariant PO5: Security Policy & Prohibited Package Detection        |
|  - Invariant PO6: Read-Only Deterministic Telemetry Emission            |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                      Structured Telemetry Envelope                      |
|  - Human-readable formatted summary or structured JSON                 |
|  - Non-repudiation audit logging in SQLite WAL ring (ADR-0035)          |
+-------------------------------------------------------------------------+
```

---

## 2. Invariant Specifications (PO1..PO6)

| ID | Title | Description & Metric Calculation Rules |
|---|---|---|
| **PO1** | Inventory Completeness | Reports exact package count (`total_packages`); sums of breakdowns match `total_packages`. Handles empty stores safely with zeroed metrics. |
| **PO2** | Multi-Dimensional Breakdowns | Aggregates packages into discrete distribution maps: `state_breakdown`, `format_breakdown`, and `architecture_breakdown`. |
| **PO3** | Footprint & Sizing | Measures active storage footprint `total_installed_size_bytes` (sum of `Installed` and `Upgradable` packages) and `average_package_size_bytes` using saturation arithmetic (`u64::saturating_add`). |
| **PO4** | Dependency Histogram | Classifies packages into bounded categorical dependency buckets: `"0"`, `"1-5"`, `"6-10"`, and `"11+"`. |
| **PO5** | Compliance & Prohibited Detection | Evaluates all packages against `PackageSecurityPolicy`, reporting `policy_compliant_count`, `policy_violations_count`, and listing detected `prohibited_packages_found`. |
| **PO6** | Deterministic Serialization | Generates immutable, read-only telemetry reports with ISO timestamps and standard error envelopes. |

---

## 3. Operator CLI Usage (`aiosh package stats`)

### Syntax:
```bash
aiosh package stats [--store <path>] [--config <path>] [--json]
```

### Options:
- `--store <path>`: Path to a custom package store JSON file (defaults to active system store).
- `--config <path>`: Path to a custom package security policy file (defaults to system policy resolution).
- `--json`: Output raw structured JSON for automated pipelines and agent consumers.

### Example Human-Readable Invocation:
```bash
aiosh package stats
```
**Output:**
```
AIOS Package Management Observability & Telemetry Report:
  Total Packages:               8
  Total Installed Footprint:    27262976 bytes
  Average Package Size:         3407872 bytes
  Policy Compliant Packages:    8/8
  Policy Violations Detected:   0
  State Breakdown:
    available        3
    installed        5
  Format Breakdown:
    apk              3
    deb              5
  Architecture Breakdown:
    amd64            5
    x86_64           3
  Dependency Distribution:
    0                4
    1-5              4
    6-10             0
    11+              0
```

### Example Structured JSON Invocation:
```bash
aiosh package stats --json
```
**Output:**
```json
{
  "code": 0,
  "data": {
    "total_packages": 8,
    "total_installed_size_bytes": 27262976,
    "average_package_size_bytes": 3407872,
    "policy_compliant_count": 8,
    "policy_violations_count": 0,
    "prohibited_packages_found": [],
    "state_breakdown": {
      "available": 3,
      "installed": 5
    },
    "format_breakdown": {
      "apk": 3,
      "deb": 5
    },
    "architecture_breakdown": {
      "amd64": 5,
      "x86_64": 3
    },
    "dependency_distribution": {
      "0": 4,
      "1-5": 4,
      "6-10": 0,
      "11+": 0
    },
    "generated_at": "2026-09-04T00:00:00Z"
  },
  "error": null
}
```

---

## 4. MCP Agent Surface (`aios.package.stats`)

Autonomous AI agents invoke the `aios.package.stats` MCP tool to query system package health before or after mutations.

### Tool Request:
```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "method": "tools/call",
  "params": {
    "name": "aios.package.stats",
    "arguments": {}
  }
}
```

### Tool Response:
```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "result": {
    "ok": true,
    "tool": "aios.package.stats",
    "report": {
      "total_packages": 8,
      "total_installed_size_bytes": 27262976,
      "average_package_size_bytes": 3407872,
      "policy_compliant_count": 8,
      "policy_violations_count": 0,
      "prohibited_packages_found": [],
      "state_breakdown": { "available": 3, "installed": 5 },
      "format_breakdown": { "apk": 3, "deb": 5 },
      "architecture_breakdown": { "amd64": 5, "x86_64": 3 },
      "dependency_distribution": { "0": 4, "1-5": 4, "6-10": 0, "11+": 0 },
      "generated_at": "2026-09-04T00:00:00Z"
    }
  }
}
```

---

## 5. Constraints & Honest Limitations
1. **Read-Only Telemetry**: Generating an observability report does not modify package state or perform package repairs.
2. **Coarse Dependency Buckets**: Dependency distribution is bucketed into fixed categorical intervals (`"0"`, `"1-5"`, `"6-10"`, `"11+"`) for $O(1)$ memory consumption rather than maintaining full cyclic dependency graph representations.
3. **In-Memory Aggregation**: Metrics are computed dynamically in-memory over the `PackageStore` slice in $O(N)$ time.
4. **Input Ceilings**: Custom store files are bounded to 100 MiB; policy files are bounded to 64 KiB; file paths are capped at 1024 characters with zero control characters permitted.

---

## 6. Task Evidence Chain
- Research: [docs/tasks/evidence/T-01271-observability-research.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01271-observability-research.md)
- Specification: [docs/tasks/evidence/T-01272-observability-specification.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01272-observability-specification.md)
- Scaffold: [docs/tasks/evidence/T-01273-observability-scaffold.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01273-observability-scaffold.md)
- Implementation: [docs/tasks/evidence/T-01274-observability-implementation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01274-observability-implementation.md)
- Unit Tests: [docs/tasks/evidence/T-01275-observability-unit-test.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01275-observability-unit-test.md)
- Integration: [docs/tasks/evidence/T-01276-observability-integration.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01276-observability-integration.md)
- Security Review: [docs/tasks/evidence/T-01277-observability-security-review.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01277-observability-security-review.md)
- Hardening: [docs/tasks/evidence/T-01278-observability-hardening.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01278-observability-hardening.md)
- Documentation: [docs/tasks/evidence/T-01279-observability-documentation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01279-observability-documentation.md)
