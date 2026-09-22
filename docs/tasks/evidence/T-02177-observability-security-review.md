# Task Evidence: T-02177 - PEP Decision Engine: Observability: Security Review

## Task Metadata
- **Task ID**: `T-02177`
- **Sub-Epic**: Sub-Epic 8: Observability Subsystem
- **Component**: `aiosh-core::pep_observability`, `aiosh-cli`, `aiosh-mcp`
- **Date**: 2026-09-22
- **Status**: Completed

## 1. Threat Model & Abuse Scenarios

### Scenario 1: Telemetry Log / Terminal Injection (CWE-117)
- **Attack Vector**: Attacker registers a rule or passes a custom timestamp containing ANSI escape codes, null bytes, or carriage return/newline (`\r\n`) characters to corrupt operator terminals or split log lines.
- **Defense & Verification**:
  - `sanitize_telemetry_text` strips all ASCII control characters (`!c.is_control()`).
  - Terminal output is sanitized using `sanitize_terminal`.
  - Timestamp field is capped at 256 bytes.
  - Verified in `test_pep_observability_sanitization`.
- **Verdict**: PASS — Injection impossible.

### Scenario 2: Directory Traversal via Policy/Store Path (CWE-22)
- **Attack Vector**: Attacker provides `../../../../etc/shadow.json` via `--store` or `store_path` to leak or evaluate unauthorized system files.
- **Defense & Verification**:
  - `validate_pep_service_path` and `validate_pep_security_policy_path` reject any paths containing `..` or non-JSON extensions.
  - Returns exit code 2 on CLI and `{"ok": false}` on MCP.
  - Verified in `test_pep_path_hygiene` and `test_pep_persistent_lifecycle`.
- **Verdict**: PASS — Traversal blocked.

### Scenario 3: Memory Exhaustion via Metric Inflation (CWE-400)
- **Attack Vector**: Flooding the store with unbounded rules or large rule bodies to cause OOM during report generation.
- **Defense & Verification**:
  - `PepDecisionService` caps rule count at `MAX_RULES_IN_SERVICE` (5,000 rules).
  - Individual rule string fields are capped during addition (`MAX_PEP_SUBJECT_LEN = 256`, `MAX_PEP_RESOURCE_LEN = 512`, `MAX_PEP_ACTION_LEN = 64`).
  - Observability aggregation operates over references and pre-sized HashMaps without heavy heap allocations.
- **Verdict**: PASS — Memory strictly bounded.

### Scenario 4: Audit Circumvention (ADR-0035 §F-2)
- **Attack Vector**: Invoking `report` or `status` without an audit record being emitted.
- **Defense & Verification**:
  - CLI `cmd_pep` executes `classify_and_emit` on both success and failure paths.
  - MCP tool uses `dispatch::recorded_call` which unconditionally seals audit ring records.
  - Both surfaces maintain SHA-256 hash chain integrity.
- **Verdict**: PASS — Consequential and query operations fully audited.

## 2. Security Audit Summary
- Zero policy bypasses identified.
- Fail-closed behavior maintained.
- Memory and string bounds strictly respected.
