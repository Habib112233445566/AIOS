# T-01396: Init & Service Supervision Recovery & Validation Integration

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Recovery & Validation  
**Task ID:** T-01396  

---

## 1. Executive Summary
Task `T-01396` integrated the **Init & Service Supervision Recovery & Validation** subsystem into the real production surfaces across AIOS:
- **Operator CLI (`aiosh service check [--fix] [--store <path>] [--json]`)**: Audits on-disk service store files, reports healthy/unhealthy states, and automatically heals damaged files via timestamped quarantine backups (`.corrupt.<ts>.bak`) when `--fix` is passed.
- **Autonomous Agent MCP (`aios.service.check`)**: Exposes JSON-RPC 2.0 tool taking `store_path` and `auto_recover`, returning full validation reports and non-destructive recovery status under PEP gating and SQLite WAL audit logging.
- **Master Test Runner Matrix (`tools/test_service_suites.py`)**: Integrated criterion `SS10` validating `test_service_recovery` across `SR1..SR5`.

---

## 2. Integrated Surfaces & Call Paths

### 1. Operator CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)
- Registered subcommand `check` in `cmd_service`:
  ```bash
  # Audit mode (read-only)
  aiosh service check --store /var/lib/aios/services.json --json

  # Self-healing recovery mode
  aiosh service check --store /var/lib/aios/services.json --fix --json
  ```
- Non-repudiable audit logging via `classify_and_emit`:
  - Normal check: action `service.check`
  - Automated repair: action `service.repair`
  - Event payload contains `healthy`, `total_services`, `valid_services`, `invalid_services`, `recovered`, and `backup_path`.

### 2. Autonomous Agent MCP Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
- Registered in `list_tools`:
  - Name: `aios.service.check`
  - Input Schema: `store_path` (string, max 1024 chars), `auto_recover` (boolean, default false), `grant_id` (string).
- Dispatched in `call_tool`:
  - Resolves target store path or defaults to `/var/lib/aios/services.json`.
  - Dispatches `load_or_recover` (if `auto_recover: true`) or `validate_service_store` (if `auto_recover: false`).
  - Audited via `dispatch::recorded_call`.

---

## 3. Automated Test Suite Outputs

### 1. CLI Smoke Suite (`code/aiosh-cli/tests/test_service_cli_smoke.py`)
```text
PASS: aiosh service --help
PASS: aiosh service unknown_cmd returns 2
PASS: aiosh service validate (name and json)
PASS: aiosh service list (prose, json, filters)
PASS: aiosh service show and status (prose, json, not found)
PASS: aiosh service action and direct shortcuts (start/stop/restart/reload)
PASS: aiosh service order (valid topological plan and negative tests)
PASS: aiosh service stats / observability (human, JSON, alias, boundaries, error paths)
PASS: aiosh service check (healthy, JSON, corruption detection, --fix quarantine recovery)

ALL SERVICE CLI SMOKE TESTS PASSED!
```

### 2. MCP Smoke Suite (`code/aiosh-mcp/tests/test_service_mcp_smoke.py`)
```text
=== RUNNING SERVICE MCP SMOKE TESTS ===
PASS: test_manifest (all 9 service tools registered)
PASS: test_validate (positive, negative, and boundary cases)
PASS: test_list (filtering, count, invalid enum)
PASS: test_get (found, not found, validation error)
PASS: test_action_and_persistence (lifecycle transitions, masked invariant, atomic save)
PASS: test_order (topological ordering, missing targets, error paths)
PASS: test_stats (telemetry metrics, inventory breakdown, error boundaries)
PASS: test_check (default healthy, corruption recovery, quarantine backup, boundaries)

ALL SERVICE MCP SMOKE TESTS PASSED!
```

### 3. Master Test Runner Matrix (`tools/test_service_suites.py`)
```text
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate, list, get, action, order)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)
[+] SS5 service configuration subsystem invariants, precedence & sizing (SC1..SC7)
[+] SS6 service automated integration tests (ST1..ST5)
[+] SS7 service security policy invariants & evaluation (SP1..SP6)
[+] SS8 service observability telemetry report & invariants (SO1..SO6)
[+] SS9 service documentation guide & invariants (D1..D6)
[+] SS10 service recovery subsystem & validation invariants (SR1..SR5)

PASS: service_suites criteria (SS1..SS10)
```
