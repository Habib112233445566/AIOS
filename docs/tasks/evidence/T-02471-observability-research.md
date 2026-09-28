# T-02471: Sandbox Enforcement Observability Research

## 1. Context & Objectives
This research establishes the telemetry models, performance metrics, and reporting constraints for the Sandbox Enforcement Observability subsystem in AIOS.

---

## 2. Established Facts vs Assumptions

### Established Facts
1. **Precedent Implementations**:
   - `AuditChainObservabilityReport` (`audit_chain_observability.rs`) and `PepGrantObservabilityReport` (`pep_grant_observability.rs`) establish the canonical telemetry pattern in AIOS: UTC timestamping, bounded cardinality, text sanitization (`MAX_TELEMETRY_TEXT_LEN = 256`), and boolean health indicators.
2. **Telemetry Sources**:
   - `SandboxService` holds in-memory registered profile state and host containment probe capabilities (`HostSandboxCapabilities`).
   - SQLite WAL `audit_ring` records historical executions with tools `sandbox`, `aios.sandbox.exec`, outcomes (`ok`, `error`, `denied`), and elapsed execution times.
3. **Core Observability Requirements (ADR-0035 §A F-2)**:
   - Operators and automated supervisors must be able to inspect real-time sandbox execution statistics, error distributions, profile utilization, and containment health without impacting active executions.

### Assumptions
- Generating an observability report should query both current service memory and the audit ring database.
- Bounded processing: Limits queries to recent audit entries (e.g. up to 10,000 rows) to ensure predictable latency and prevent heap exhaustion.

---

## 3. Decisions & Interface Contracts
- Module: `code/aiosh-rust/aiosh-core/src/sandbox_observability.rs`
- Invariants: `SANDBOXOBS1`..`SANDBOXOBS6`
- CLI command: `aiosh sandbox stats`
- MCP tool: `aios.sandbox.stats`
