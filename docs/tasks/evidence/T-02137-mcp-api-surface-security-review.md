# Task Evidence: T-02137 (MCP/API Surface: Security Review)

## Overview
- **Task ID**: `T-02137`
- **Task Name**: MCP/API surface: Security Review
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 4: MCP / API Surface
- **Timestamp**: 2026-09-21T00:53:30+05:00
- **Status**: COMPLETED

## Threat Modeling & Abuse Scenarios (`THREAT-PEPMCP-01..06`)

### `THREAT-PEPMCP-01`: Policy Store Path Traversal
- **Vector**: An attacker sends `store_path: "../../../etc/shadow.json"` or null-byte infused paths to read or overwrite arbitrary system files.
- **Analysis**: The MCP handlers invoke `aiosh_core::pep_decision_service::validate_pep_service_path(path)` prior to any filesystem operation. The path is rejected if it contains `..` components, control characters, is $> 1024$ chars, or lacks a `.json` extension.
- **Verdict**: MITIGATED. Tested in `test_pep_decision_smoke.py`.

### `THREAT-PEPMCP-02`: Parameter & Control Character Injection
- **Vector**: An attacker provides rule IDs containing ANSI escapes, null bytes, or line breaks to corrupt logs or deceive operators.
- **Analysis**: Handlers enforce strict validation: `id` must be non-empty, $\le 128$ characters, and contain zero control characters (`c.is_control()`). `effect` is strictly checked against `"permit"` and `"deny"`.
- **Verdict**: MITIGATED.

### `THREAT-PEPMCP-03`: Unaudited Policy Modification
- **Vector**: A rogue subagent invokes `aios.pep.rule_add` or `aios.pep.rule_remove` without an audit record being created.
- **Analysis**: All MCP tool endpoints route through `dispatch::recorded_call`. Every invocation writes an immutable SHA-256 hash-chained row to the SQLite audit ring before returning the JSON-RPC response.
- **Verdict**: MITIGATED.

### `THREAT-PEPMCP-04`: Authorization Bypass via Ambiguous Rules
- **Vector**: An evaluator request is submitted with missing or unparseable rules expecting default permit.
- **Analysis**: PEP decision engine strictly implements default-deny (`PEPDEC1`). Unmatched requests always return `effect: "deny"` and `allowed: false`. Combining algorithms (`deny_overrides`, `permit_overrides`, `first_applicable`) are deterministic.
- **Verdict**: MITIGATED.

### `THREAT-PEPMCP-05`: DoS via Policy Store Memory Exhaustion
- **Vector**: An adversary submits thousands of rules to exhaust server RAM.
- **Analysis**: `PepDecisionService::add_rule` enforces a hard ceiling of `MAX_RULES_IN_SERVICE = 5000`. Exceeding this returns an explicit error (`PEPSERV_ERR_CAPACITY`) without allocating additional memory.
- **Verdict**: MITIGATED.

### `THREAT-PEPMCP-06`: Corrupt Store Deserialization Panic
- **Vector**: An attacker corrupts the `.json` file on disk to cause `aiosh-mcp` to crash on subsequent calls.
- **Analysis**: Handlers use `PepDecisionService::load_or_recover(path)`, which catches deserialization failures, moves the corrupted file to a secure timestamped quarantine (`.bak.<timestamp>` with mode `0600`), and initializes a clean store without crashing or panicking.
- **Verdict**: MITIGATED.

## Conclusion
Zero open policy bypasses or unmitigated threat vectors remain. The MCP/API surface is secure and adheres to all AIOS Zero Trust requirements.
