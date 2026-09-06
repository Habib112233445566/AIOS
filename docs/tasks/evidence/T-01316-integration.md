# T-01316 — Init & Service Supervision / Core Service: Integration

## 1. Production Surfaces Integrated
- **CLI Subcommand Surface (`aiosh service`)**:
  - `aiosh service list [--pattern <pat>] [--state <state>] [--mode <mode>] [--limit <n>] [--json] [--store <path>]`: Lists registered system services filtered by pattern, state, startup mode, and limit.
  - `aiosh service show <name> [--json] [--store <path>]`: Displays detailed service specification (unit metadata, dependencies, execution target, restart policy) and runtime status (state, startup mode, PID, restart count, timestamps).
  - `aiosh service action <name> <action> [--json] [--store <path>]`: Executes FSM state transition or administrative mode changes (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`), records structured audit rows to `audit.log`, and atomically persists store updates.
  - `aiosh service order <name> [--json] [--store <path>]`: Computes topological startup execution sequence for target service and all transitive dependencies via Kahn's algorithm with cycle detection.
  - `aiosh service validate <spec-file>`: Validates service specification files against naming and schema constraints.
- **MCP Server Tool Surface (`aiosh-mcp`)**:
  - `aios.service.list`: Dispatches query across registered services with input sanitization and PEP authorization via `dispatch::recorded_call`.
  - `aios.service.get`: Retrieves specific service specification and runtime status.
  - `aios.service.action`: Executes service action with lifecycle transitions, persistence, and audit logging.
  - `aios.service.order`: Returns calculated topological startup order.
  - `aios.service.validate`: Validates service specifications and names.

## 2. Automated Integration Test Coverage
- `test_cmd_service_flow`: Rust CLI integration test in `aiosh-cli` testing `list`, `show`, `action` (start, stop, restart, enable, mask), `order`, and `validate`.
- `test_mcp_service_tools`: Full MCP tool execution test in `aiosh-mcp` verifying tool dispatch, audit recording, and error handling.
- `tools/test_service_suites.py`: Master test runner validating all criteria (`SS1..SS4`).

## 3. Verification Output
```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```
