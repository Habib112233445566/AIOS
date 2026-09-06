# T-01323: Init & Service Supervision - CLI Surface: Scaffold

## Metadata
- **Task ID:** `T-01323`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Scaffold
- **Status:** Complete

## 1. Scaffold Deliverables
- **CLI Subcommand Surface**:
  - `code/aiosh-rust/aiosh-cli/src/main.rs` extended with command dispatcher `cmd_service`:
    - `validate`: Enforcing 1 MiB payload limit and calling `validate_service_name` or `validate_service_spec`.
    - `list`: Providing query matrix parsing (`--pattern`, `--state`, `--mode`, `--limit`, `--store`, `--json`).
    - `show` / `status`: Retrieving specification and runtime state by service name.
    - `action`: FSM lifecycle state transitions (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`).
    - Direct action shortcuts: `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask` forwarding cleanly with preserved options and honest audit emission.
    - `order`: Topological startup execution planning via Kahn's algorithm.
- **Audit Logging Integration**:
  - Every subcommand routes through `classify_and_emit`, capturing actor, action, parameters, and outcome into the structured audit log.
- **Test Stub Coverage**:
  - `task_cli_tests::test_cmd_service_flow` tests all subcommands (`help`, `unknown`, `validate`, `list`, `show`, `status`, `action`, `start`, `stop`, `order`) with both human and `--json` modes.

## 2. Compilation & Verification Output
```
   Compiling aiosh-cli v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s)
     Running unittests src\main.rs

running 1 test
test task_cli_tests::test_cmd_service_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; finished in 0.79s
```
