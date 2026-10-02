# T-02571: Privilege Escalation Prevention Observability Research

- **Task**: `T-02571`
- **Sub-Epic**: Privilege Escalation Prevention / observability
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Research Objectives
Establish facts, architectural patterns, telemetry schemas, and health heuristics for the Privilege Escalation Prevention observability subsystem in `code/aiosh-rust/aiosh-core/src/privilege_observability.rs`.

## 2. Prior Art & Subsystem Analysis
Across `sandbox_observability.rs`, `pep_grant_observability.rs`, and `pep_observability.rs`:
1. **Report Structure**: Point-in-time snapshot with `generated_at_utc`, active entity counts, categorical breakdown maps (`HashMap<String, usize>`), policy state, and an overall boolean `is_healthy`.
2. **Audit Telemetry Aggregation**: When an `AuditRing` instance is accessible, recent rows filtered by `tool == "privilege"` are tallied for outcomes and transition types.
3. **Cardinality & Sanitization Hardening**:
   - `sanitize_telemetry_text`: Control characters filtered out, string truncated to 256 characters.
   - Distribution maps capped at `MAX_OUTCOME_DISTRIBUTION_ENTRIES` (128) to prevent memory ballooning under high-cardinality attacks.
4. **Health Check Heuristics**:
   - Registered contexts within configured capacity bounds.
   - Non-empty timestamp.

## 3. Facts vs Assumptions
- **Fact**: Observability report must be serializable to JSON and deserializable without precision loss.
- **Fact**: Telemetry text strings must be sanitized to eliminate ANSI escape injection and control characters.
- **Fact**: The subsystem must integrate into CLI (`aiosh privilege stats` / `observability`) and MCP (`aios.privilege.observability`).

## 4. Key Design Decisions for Specification (T-02572)
1. Define `PrivilegeObservabilityReport` in `code/aiosh-rust/aiosh-core/src/privilege_observability.rs`.
2. Implement `PrivilegeService::generate_observability_report()`.
3. Add CLI subcommand and MCP tool dispatch.
