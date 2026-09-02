# T-00781 — Secrets & Access Hygiene / observability: Research

## 1. Prior Art & In-Tree Observability
- **Data Model Metrics**:
  - `SecretScanReport` tracks `scanned_files`, `scanned_bytes`, `duration_ms`, and `clean` boolean alongside `findings`.
  - Severity breakdown: `Critical`, `High`, `Medium`, `Low`, `Info` can be aggregated for observability.
- **Audit Logging**:
  - Consequential MCP calls (`aios.secrets.scan`, `aios.secrets.check`) route through `dispatch::recorded_call`, inserting structured audit rows into the SQLite WAL ring.
- **CLI Telemetry**:
  - CLI prints human-readable scan summaries to stderr/stdout with elapsed milliseconds, file counts, and findings breakdowns.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Scan Report Metrics | Fact | `SecretScanReport` encapsulates scan counters and duration. |
| Audit Trail | Fact | Audit ring persists tool execution events with parameters and result hashes. |
| Redaction Telemetry | Fact | Telemetry must never emit raw unredacted secret tokens. |

## 3. Decisions & Contracts Needed
1. Ensure `SecretScanReport` provides helper method `severity_counts() -> BTreeMap<String, usize>` or similar summary helper for telemetry.
2. Add criteria test `K8` to `tools/test_secrets_suites.py` validating observability summary metrics and audit compliance.
