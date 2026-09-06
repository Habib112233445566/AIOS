# T-01325: Init & Service Supervision - CLI Surface: Unit Tests

## Metadata
- **Task ID:** `T-01325`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Unit Tests
- **Status:** Complete

## 1. Unit Test Deliverables
Created standalone CLI smoke test suite `code/aiosh-cli/tests/test_service_cli_smoke.py` and expanded Rust unit test suite in `code/aiosh-rust/aiosh-cli/src/main.rs` (`test_cmd_service_flow`):
- **`help` & Command Dispatch**:
  - `aiosh service --help`: Validates exit code 0 and presence of command taxonomy.
  - `aiosh service unknown_cmd`: Validates exit code 2 and structured error.
- **`validate`**:
  - Valid service name (`aios-securityd.service`) -> Exit code 0
  - Valid service name with `--json` -> Exit code 0, parses valid boolean
  - Invalid service name (`bad/name.service`) -> Exit code 2
  - Missing flags -> Exit code 2
  - Valid inline `ServiceSpec` JSON -> Exit code 0
  - Invalid inline `ServiceSpec` JSON (self-dependency) -> Exit code 2
- **`list`**:
  - Default prose output -> Exit code 0
  - Canonical JSON output (`--json`) -> Exit code 0, validates array length $\ge 5$
  - Filter by lifecycle state (`--state active`) -> Exit code 0
  - Invalid state filter rejection (`--state bogus_state`) -> Exit code 2
  - Filter by startup mode (`--mode enabled`) -> Exit code 0
  - Invalid startup mode rejection (`--mode invalid_mode`) -> Exit code 2
  - Filter by substring pattern (`--pattern audit`) -> Exit code 0
  - Invalid limit value (`--limit 0`) -> Exit code 2
- **`show` & `status` Alias**:
  - Existing service (`auditd.service`) -> Exit code 0
  - Existing service with `--json` -> Exit code 0, validates parsed JSON
  - Status alias (`aiosh service status auditd.service --json`) -> Exit code 0, identical schema
  - Missing service name -> Exit code 2
  - Nonexistent service -> Exit code 1
  - Control characters in service name (`bad\0name`) -> Exit code 2
- **`action` & Direct Shortcuts**:
  - Action execution (`action auditd.service stop --store <path> --json`) -> Exit code 0, new state `inactive`
  - Direct shortcut `start` (`start auditd.service --store <path> --json`) -> Exit code 0, new state `active`
  - Direct shortcut `restart` (`restart auditd.service --store <path>`) -> Exit code 0
  - Direct shortcut `reload` (`reload auditd.service --store <path>`) -> Exit code 0
  - Direct shortcut `disable` (`disable auditd.service --store <path>`) -> Exit code 0
  - Direct shortcut `enable` (`enable auditd.service --store <path>`) -> Exit code 0
  - Missing argument on shortcut (`aiosh service start`) -> Exit code 2
  - Invalid action name (`invalid_act`) -> Exit code 2
- **`order`**:
  - Valid dependency ordering (`aiosh service order aios-securityd.service --json`) -> Exit code 0, verifies `auditd.service` and `dbus.service` precede `aios-securityd.service` in topological sequence
  - Nonexistent service ordering -> Exit code 1
  - Missing service name -> Exit code 2

## 2. Test Execution Output
```text
running 1 test
test task_cli_tests::test_cmd_service_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.91s
```
```text
PASS: aiosh service --help
PASS: aiosh service unknown_cmd returns 2
PASS: aiosh service validate (name and json)
PASS: aiosh service list (prose, json, filters)
PASS: aiosh service show and status (prose, json, not found)
PASS: aiosh service action and direct shortcuts (start/stop/restart/reload)
PASS: aiosh service order (valid topological plan and negative tests)

ALL SERVICE CLI SMOKE TESTS PASSED!
```
