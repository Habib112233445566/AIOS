# T-01324: Init & Service Supervision - CLI Surface: Implementation

## Metadata
- **Task ID:** `T-01324`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Implementation
- **Status:** Complete

## 1. Implementation Details
- **Command Dispatcher (`cmd_service`) in `code/aiosh-rust/aiosh-cli/src/main.rs`**:
  - `validate`: Enforces 1 MiB size limit on spec files and inline strings. Validates SS1 naming syntax and SS1..SS5 specification invariants.
  - `list`: Supports query filtering by `--pattern`, `--state`, `--mode`, `--limit`, and `--store`.
  - `show` / `status`: Retrieves and renders service specification and runtime status.
  - `action`: Dispatches lifecycle transitions (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`), enforces FSM rules, and updates store atomically.
  - Direct Action Shortcuts: `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask` forward cleanly with arguments preserved and honest audit rows written.
  - `order`: Computes topological startup order via Kahn's algorithm with cycle detection.
  - Standardized JSON responses when `--json` is supplied (`{"code": <n>, "data": ..., "error": ...}`).
- **Audit Logging**:
  - Every subcommand unconditionally logs to the audit WAL ring / `audit.log` via `classify_and_emit` on all paths (success, validation failure, store failure).
- **Unit & Integration Test Coverage**:
  - `task_cli_tests::test_cmd_service_flow` tests `validate`, `list`, `show`, `status`, `action`, `start`, `stop`, `restart`, `reload`, `disable`, `enable`, `order`, `--help`, and invalid inputs.

## 2. Test Verification Output
```
   Compiling aiosh-cli v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s)
     Running unittests src\main.rs

running 1 test
test task_cli_tests::test_cmd_service_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; finished in 0.91s
```
```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```
