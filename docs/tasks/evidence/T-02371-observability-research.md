# Task Evidence: T-02371 (Audit Chain Extensions / observability: Research)

## 1. Metadata
- **Task ID:** `T-02371`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Observability Research
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 8: Observability (1/10) — Research

---

## 2. Research Findings & Prior Art Analysis

### 2.1 Prior Art in AIOS Observability
Observability subsystems across AIOS (`pep_observability.rs`, `pep_grant_observability.rs`, `network_observability.rs`) follow consistent telemetry design rules:
1. **Point-in-Time Snapshot Generation**: Pure aggregation functions taking references to underlying services without side-effects or locking mutations.
2. **Cardinality & Distribution Aggregation**: Tracking unique actors, tools, sessions, traces, outcomes, and causal linkages.
3. **Telemetry Sanitization**: Stripping control characters and truncating strings (`sanitize_telemetry_text`, $\le 256$ chars) to prevent log injection or ANSI escape exploits.
4. **Health & Capacity Gauges**: Calculating capacity utilization percentages, disk storage footprints, and continuous SHA-256 chain integrity status.

### 2.2 Facts vs. Assumptions

| Item | Status | Details |
| :--- | :---: | :--- |
| **Fact** | Confirmed | `AuditChainService` currently provides raw `query_events` and `verify_integrity`, but lacks a structured observability report aggregating provenance statistics, causal linkage counts, and signature distributions. |
| **Fact** | Confirmed | Operators and monitoring sidecars require lightweight health metrics without executing full recursive DAG lineage traversals. |
| **Fact** | Confirmed | Metric text outputs must be sanitized to protect dashboards and terminal observers from log injection. |
| **Assumption** | Validated | Defining `AuditChainObservabilityReport` in `code/aiosh-rust/aiosh-core/src/audit_chain_observability.rs` provides full introspection over the extended audit ring with sub-millisecond generation time. |

### 2.3 Proposed Observability Metrics (`AUDITOBS1..AUDITOBS6`)
- **`AUDITOBS1` (Event & Provenance Metrics)**: Total extended rows, unique sessions, unique traces, unique actors, unique tools.
- **`AUDITOBS2` (Causal DAG Metrics)**: Total causal links, link count distribution, max causal fanout observed.
- **`AUDITOBS3` (Cryptographic Verification Metrics)**: Count of signed rows, unsigned rows, algorithm breakdown (`ed25519`).
- **`AUDITOBS4` (Policy & Decision Metrics)**: Active policy mode, count of denied vs permitted events.
- **`AUDITOBS5` (Storage & Resource Metrics)**: SQLite database file size in bytes, SQLite page count.
- **`AUDITOBS6` (Health Gauge)**: Overall boolean health status (`is_healthy`) evaluating database reachability and recent hash-chain integrity.

---

## 3. Decisions & Next Steps
1. **Decision**: Implement `AuditChainObservabilityReport` in `code/aiosh-rust/aiosh-core/src/audit_chain_observability.rs`.
2. **Decision**: Wire into CLI (`aiosh audit stats` / `aiosh audit telemetry [--json]`) and MCP (`aios.audit.stats`).
3. **Next Step**: Author specification in `T-02372` (`docs/tasks/evidence/T-02372-observability-specification.md`).
