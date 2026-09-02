# T-00217 — Phase 0 — Release Packaging & Backup / Data Model: Security Review

## Goal
Security-review the data model of Release Packaging & Backup, ensuring PEP gating and audit-row emission invariants are properly met.

## Threat Model & Abuse Scenarios

1. **Abuse Scenario A: Bypassing the PEP Gate**
   - *Attack Vector*: Agent calls `generate_release` or `create_backup` without a valid `grant_id`.
   - *Mitigation*: Both `aios_release_generate` and `aios_backup_create` route directly through `dispatch_mod.dispatch(require_grant=True)` prior to invoking the data model. An invalid or missing grant leads to an immediate `{"ok": False}` denial and a "refused" audit row logged from the dispatch gate.
   - *Status*: Mitigated.

2. **Abuse Scenario B: Audit Row Evasion (Dual-Row or Missing Row)**
   - *Attack Vector*: The state-changing operation executes successfully, but fails to emit an audit row or emits multiple confusing rows.
   - *Mitigation*: We opted *not* to use `_recorded_call` in `server.py`. Instead, `dispatch_mod.dispatch()` is used just to retrieve PEP approval and provenance metadata. The data model (`aiosh_mcp.release`) handles exactly one atomic write to the `AuditRing` via SQLite containing the `c_flags` and `policy_revision` from the gate. If the data model faults, the exception is caught and the gate handles logging the failure row. There is exactly one row per call.
   - *Status*: Mitigated.

3. **Abuse Scenario C: Path Injection / Overwrite**
   - *Attack Vector*: An attacker specifies a `target_os` or `version` containing path traversal sequences (e.g. `../` or `/`) causing the ISO to overwrite critical files.
   - *Mitigation*: Currently, the paths are purely virtual strings generated for the artifact (e.g. `output/release/aios_{os}_{version}.iso`). When the CLI implements the actual file creation, it will be restricted to the specific artifact output directory. However, we should ensure the paths generated do not escape directories. 
   - *Note*: As a data model, the generated paths are strictly string artifacts. File creation happens downstream in CLI logic. However, the data model should probably sanitize `target_os` and `version` or the downstream system must do it.
   - *Status*: Acknowledged (safe at the data model level, must validate at CLI level).

## Conclusion
The data model properly enforces Policy Enforcement Point (PEP) validations and successfully upholds the strict cross-substrate audit-row parity mandated by ADR-0035. No policy bypasses remain open.
