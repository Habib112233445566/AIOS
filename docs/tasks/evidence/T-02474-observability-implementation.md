# T-02474: Sandbox Enforcement Observability Implementation

## 1. Implementation Overview
This task implements `SandboxObservabilityReport::generate` and integrates it into `SandboxService` via `service.generate_observability_report()`.

---

## 2. Key Components
1. **Telemetry Ingestion & Filtering**:
   - Aggregates executions from the SQLite WAL `audit_ring` where `tool == "sandbox"` or `tool.starts_with("aios.sandbox")`.
   - Populates `executions_by_outcome` and `executions_by_profile` with text sanitization (`sanitize_telemetry_text`).
2. **Capability & Profile Telemetry**:
   - Collects host platform containment capabilities via `probe_host_capabilities()`.
   - Records total registered profiles and active security policy enforcement mode.
3. **Health Evaluation**:
   - Asserts factory profile integrity (`standard`, `strict`, `permissive`) to derive `is_healthy: bool`.
4. **Service Integration**:
   - Exposed `generate_observability_report(&self)` on `SandboxService`.

---

## 3. Build Status
- Workspace compiles with 0 warnings and 0 errors (`cargo check --workspace`).
