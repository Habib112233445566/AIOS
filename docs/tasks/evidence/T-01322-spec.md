# T-01322: Init & Service Supervision - CLI Surface: Specification

## Metadata
- **Task ID:** `T-01322`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Specification
- **Status:** Complete

## 1. CLI Dispatch Specification (`aiosh service`)

### 1.1 Command Interface & Entry Point
```rust
fn cmd_service(args: &[String]) -> i32
```
Dispatched in `aiosh-cli/src/main.rs`:
```rust
Some("service") => cmd_service(&args[1..]),
```

### 1.2 Subcommands & Contract

#### 1. `validate`
- **Syntax**: `aiosh service validate (--name <name> | --spec <file_or_json>) [--json]`
- **Inputs**: Service unit name or serialized `ServiceSpec` (from file path or inline JSON string).
- **Validation**: Enforces 1 MiB payload ceiling. Validates SS1 naming syntax or SS1..SS5 specification invariants.
- **Output**:
  - Text: `VALID: ...` or `INVALID: ...` with detailed invariant error list.
  - JSON: Result envelope `{"code": 0|2, "data": ..., "error": ...}`.
- **Exit Codes**: `0` on valid, `1` on file read error, `2` on validation violation or missing arguments.
- **Persistence**: Read-only; zero disk side effects.

#### 2. `list`
- **Syntax**: `aiosh service list [--pattern <pat>] [--state <state>] [--mode <mode>] [--limit <n>] [--store <path>] [--json]`
- **Inputs**: Query filters (name pattern, lifecycle state, startup mode, maximum result limit) and optional store path.
- **Output**:
  - Text: Formatted table with `NAME`, `TYPE`, `STATE`, `DESCRIPTION`.
  - JSON: Array of serialized `ServiceSpec` objects.
- **Exit Codes**: `0` on success, `1` on store loading failure, `2` on invalid argument.
- **Persistence**: Read-only.

#### 3. `show` / `status`
- **Syntax**: `aiosh service show <name> [--store <path>] [--json]`
- **Alias (AIOS-specific)**: `aiosh service status <name> [--store <path>] [--json]`
- **Inputs**: Service unit name identifier and optional store path.
- **Output**:
  - Text: Multi-line attribute display (unit, description, type, state, startup mode, exec_start, PID, health).
  - JSON: Result envelope `{"code": 0, "data": {"service": ..., "status": ...}, "error": null}`.
- **Exit Codes**: `0` on found, `1` on not found or store read error, `2` on missing/invalid name argument.
- **Persistence**: Read-only.

#### 4. `action`
- **Syntax**: `aiosh service action <name> <action> [--store <path>] [--json]`
- **Allowed Actions**: `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`.
- **Shortcuts (AIOS-specific convenience aliases)**:
  - `aiosh service start <name> [--store <path>] [--json]`
  - `aiosh service stop <name> [--store <path>] [--json]`
  - `aiosh service restart <name> [--store <path>] [--json]`
  - `aiosh service reload <name> [--store <path>] [--json]`
  - `aiosh service enable <name> [--store <path>] [--json]`
  - `aiosh service disable <name> [--store <path>] [--json]`
  - `aiosh service mask <name> [--store <path>] [--json]`
  - `aiosh service unmask <name> [--store <path>] [--json]`
- **Inputs**: Service unit name and target lifecycle action.
- **Output**:
  - Text: Action outcome summary showing previous state to new state transition.
  - JSON: Serialized `ServiceActionReport` envelope.
- **Exit Codes**: `0` on success, `1` on FSM rejection or persistence failure, `2` on invalid action or argument.
- **Persistence**: Updates in-memory store and atomically persists to disk when `--store` is specified.

#### 5. `order`
- **Syntax**: `aiosh service order <name> [--store <path>] [--json]`
- **Inputs**: Target service name.
- **Validation**: Verifies target exists and evaluates dependency closure using Kahn's topological sort with cycle detection.
- **Output**:
  - Text: Numbered sequential startup execution plan.
  - JSON: Result envelope `{"code": 0, "data": {"target": name, "order": [...]}, "error": null}`.
- **Exit Codes**: `0` on success, `1` on cyclic dependency or missing dependency, `2` on missing argument.
- **Persistence**: Read-only.

### 1.3 Reuse vs. AIOS-Specific Extensions
- **Reused Upstream Components**:
  - `aiosh-core::service`: `ServiceSpec`, `ServiceStatus`, `ServiceHealth`, `ServiceQuery`, `ServiceAction`, and validation invariants `SS1..SS5`.
  - `aiosh-core::service_service::ServiceStore`: Registry and lifecycle engine (`CS1..CS5`).
  - `classify_and_emit`: System-wide structured audit emission to SQLite WAL ring (`audit.db`).
- **AIOS-Specific Extensions**:
  - Subcommand aliases (`status`, `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`) mapped to unified `show` and `action` operations.
  - Standardized JSON envelope format with numeric exit code, typed data object, and structured error payloads.

### 1.4 Audit Trail Guarantee (ADR-0035)
Every subcommand execution unconditionally calls:
```rust
classify_and_emit(
    &mut ctx,
    "service",
    subcommand_name,
    json_params,
    outcome,
    target_name,
    detail,
    "operator",
    None,
);
```
Guaranteeing immutable audit row logging across both success and failure branches.
