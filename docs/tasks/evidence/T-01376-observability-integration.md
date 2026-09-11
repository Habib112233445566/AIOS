# T-01376: Init & Service Supervision - Observability: Integration

## Metadata
- **Task ID:** `T-01376`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Observability
- **Status:** Complete
- **Date:** 2026-09-09

---

## 1. Scope & Objective
Integrate `ServiceObservabilityReport` into the operator CLI surface (`aiosh service stats` / `aiosh service observability`) and the autonomous agent Model Context Protocol (MCP) server (`aios.service.stats`) with SQLite WAL audit row emissions, PEP authorization checks, and end-to-end smoke verification.

---

## 2. Integration Details

### 1. CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)
- Implemented `aiosh service stats` and alias `aiosh service observability`:
  - `--store <path>`: Allows operators to target custom service store files with strict validation ($\le 1024$ chars, rejection of control characters).
  - `--policy <path>` / `--config <path>`: Resolves security policy constraints to compute compliance and prohibited service metrics.
  - `--json`: Formats the telemetry report as standard JSON matching cross-substrate invariants.
  - Human-readable output formatting covering total services, health counts, restart telemetry, policy compliance, and categorical distribution breakdowns (state, mode, type, restart policy, dependencies).
  - Audit row emission: Invocations call `classify_and_emit` to record an immutable SHA-256 hash-chained audit record in the SQLite WAL ring buffer.
  - Help text updated to advertise the `stats` subcommand and its `observability` alias.
  - Added unit test cases to `task_cli_tests::test_cmd_service_flow` in `aiosh-cli`.

### 2. MCP Server (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
- Registered `aios.service.stats` tool schema in `list_tools`:
  - Inputs: `store_path` (optional string), `policy_path` (optional string), `grant_id` (optional PEP authorization).
- Implemented execution handler inside `call_tool`:
  - Path and argument validation: bounds checking, rejection of control characters.
  - Generates telemetry via `ServiceObservabilityReport::generate_from_paths`.
  - Wrapped within `dispatch::recorded_call` ensuring PEP gating and non-repudiable audit logging.
- Unit testing in `tests::test_mcp_service_tools` asserts tool registration, standard metrics generation, and error handling for malformed input paths.

### 3. Automated Smoke Suites & CI Matrix
- **CLI Smoke Suite (`code/aiosh-cli/tests/test_service_cli_smoke.py`)**:
  - Added `test_service_stats()` validating human-readable output, JSON formatting, the `observability` alias, boundary control-character handling (`\x07`), and non-existent store files.
- **MCP Smoke Suite (`code/aiosh-mcp/tests/test_service_mcp_smoke.py`)**:
  - Added `test_stats()` validating JSON-RPC stdio responses, metrics completeness, and path boundary rejections.
  - Updated `test_manifest()` to mandate `aios.service.stats` among the 8 registered service tools.
- **Master Test Runner Matrix (`tools/test_service_suites.py`)**:
  - Added criterion `SS8` (`test_ss8_observability`) validating `test_service_observability`. All criteria `SS1..SS8` pass.

---

## 3. Verification & Test Evidence

1. **Cargo Observability Unit Tests**:
   ```text
   running 7 tests
   test test_so1_inventory_completeness_and_empty_store ... ok
   test test_so2_categorical_distributions ... ok
   test test_so3_health_and_restart_telemetry ... ok
   test test_so4_dependency_distribution_histogram ... ok
   test test_so5_security_policy_compliance ... ok
   test test_so6_serialization_and_string_helpers ... ok
   test test_so7_hardening_and_path_boundaries ... ok

   test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
   ```

2. **Cargo CLI Integration Test**:
   ```text
   running 1 test
   test task_cli_tests::test_cmd_service_flow ... ok

   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 4.24s
   ```

3. **Cargo MCP Tool Integration Test**:
   ```text
   running 1 test
   test tests::test_mcp_service_tools ... ok

   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.20s
   ```

4. **CLI Smoke Suite**:
   ```text
   PASS: aiosh service --help
   PASS: aiosh service unknown_cmd returns 2
   PASS: aiosh service validate (name and json)
   PASS: aiosh service list (prose, json, filters)
   PASS: aiosh service show and status (prose, json, not found)
   PASS: aiosh service action and direct shortcuts (start/stop/restart/reload)
   PASS: aiosh service order (valid topological plan and negative tests)
   PASS: aiosh service stats / observability (human, JSON, alias, boundaries, error paths)

   ALL SERVICE CLI SMOKE TESTS PASSED!
   ```

5. **MCP Smoke Suite**:
   ```text
   === RUNNING SERVICE MCP SMOKE TESTS ===
   PASS: test_manifest (all 8 service tools registered)
   PASS: test_validate (positive, negative, and boundary cases)
   PASS: test_list (filtering, count, invalid enum)
   PASS: test_get (found, not found, validation error)
   PASS: test_action_and_persistence (lifecycle transitions, masked invariant, atomic save)
   PASS: test_order (topological ordering, missing targets, error paths)
   PASS: test_stats (telemetry metrics, inventory breakdown, error boundaries)

   ALL SERVICE MCP SMOKE TESTS PASSED!
   ```

6. **Master Service Test Matrix (`tools/test_service_suites.py`)**:
   ```text
   [+] SS1 service data model integrity & invariants (SS1..SS5)
   [+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
   [+] SS3 service MCP tool surface (validate, list, get, action, order)
   [+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)
   [+] SS5 service configuration subsystem invariants, precedence & sizing (SC1..SC7)
   [+] SS6 service automated integration tests (ST1..ST5)
   [+] SS7 service security policy invariants & evaluation (SP1..SP6)
   [+] SS8 service observability telemetry report & invariants (SO1..SO6)

   PASS: service_suites criteria (SS1..SS8)
   ```
