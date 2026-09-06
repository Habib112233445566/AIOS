# T-01366: Init & Service Supervision - Security Policy: Integration

## Metadata
- **Task ID:** `T-01366`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Security Policy
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Scope & Objective
Integrate `ServiceSecurityPolicy` into the production CLI surface (`aiosh service policy`) and the Model Context Protocol (MCP) server (`aios.service.policy`) with SQLite WAL audit row emissions and PEP authorization checks.

---

## 2. Integration Details

### 1. CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)
- Added `aiosh service policy` subcommand handling:
  - `--config <path>`: Allows operators to evaluate policies defined in custom JSON configuration files (bounded at 64 KiB).
  - `--service <name>` / `--name <name>`: Evaluates a target service specification against security policy rules and outputs violation details with `FATAL` vs `WARN` flags.
  - Policy inspection: When no service name is provided, dumps the resolved policy attributes and constraints in human-readable or structured JSON format.
  - Audit logging: Calls `classify_and_emit` to record an immutable SHA-256 hash-chained audit record in the SQLite WAL ring for every policy evaluation.
  - Updated `aiosh service --help` usage contract to advertise the `policy` command.

### 2. MCP Server (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
- Registered `aios.service.policy` tool schema in `list_tools`:
  - Inputs: `service_name` (optional string), `config_path` (optional string), `store_path` (optional string), `grant_id` (optional PEP authorization).
- Implemented execution handler inside `call_tool`:
  - Path and argument validation: bounded string lengths ($\le 1024$), rejection of control characters.
  - Prohibited services check: immediately flags prohibited services.
  - Store lookup and spec evaluation: passes target `ServiceSpec` through `policy.evaluate_spec()`.
  - Wrapped within `dispatch::recorded_call` ensuring PEP gating and audit ring persistence.
- Added comprehensive unit tests in `aiosh-mcp/src/main.rs` (testing tool registration, general policy inspection, and evaluation of prohibited services like `telnet.service`).

---

## 3. Verification & Test Evidence
1. **MCP Tests**:
   - Ran `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp`.
   - Result: 10 passed, 0 failed (including `test_mcp_service_tools`).
2. **CLI Tests**:
   - Ran `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli`.
   - Result: 22 passed, 0 failed (including `test_cmd_service_flow`).
