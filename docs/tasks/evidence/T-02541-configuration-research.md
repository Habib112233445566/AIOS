# T-02541: Privilege Escalation Prevention Configuration Research

- **Task**: `T-02541`
- **Sub-Epic**: Privilege Escalation Prevention / configuration
- **Date**: 2026-09-29
- **Status**: PASSED

## 1. Research Objectives
Establish the architecture, schema requirements, environment override semantics, and bounded validation rules for the Privilege Escalation Prevention configuration subsystem in `code/aiosh-rust/aiosh-core/src/privilege_config.rs`.

## 2. Prior Art & Subsystem Analysis
Across `aiosh-core`, configuration components (`sandbox_config.rs`, `pep_grant_config.rs`, `capability_config.rs`, `audit_chain_config.rs`) adhere to a uniform, robust design:
1. **Bounded File Size**: Config files capped at $\le 64\text{ KiB}$ to prevent memory exhaustion / DoS.
2. **Deterministic Error Taxonomy**: Dedicated constants (`PRIVESCCONF_ERR_IO`, `PRIVESCCONF_ERR_PARSE`, `PRIVESCCONF_ERR_VALIDATION`, `PRIVESCCONF_ERR_BOUNDS`).
3. **Environment Overrides**: Overrides prefixed with `AIOS_PRIVILEGE_*` with strict bounds checking.
4. **Validation Guardrails**:
   - `max_active_contexts`: Clamped between 1 and 16,384 (default 1,024).
   - `max_grant_duration_seconds`: Clamped between 1 and 86,400 (default 3,600).
   - `max_capabilities_per_context`: Clamped between 1 and 64 (default 32).
   - `store_path`: Absolute or relative path without `..` directory traversal patterns.

## 3. Facts vs Assumptions
- **Fact**: Configuration must serialize cleanly to JSON and deserialize with zero data loss.
- **Fact**: Default configuration must be secure-by-default (e.g. `audit_all_transitions: true`, `enforce_grant_signatures: true`).
- **Fact**: Deserializing must reject unsupported schema versions.

## 4. Key Design Decisions for Specification (T-02542)
1. Implement `PrivilegeConfig` struct in `code/aiosh-rust/aiosh-core/src/privilege_config.rs`.
2. Re-export in `aiosh_core::lib.rs`.
3. Provide full CLI and MCP configuration inspection endpoints.
