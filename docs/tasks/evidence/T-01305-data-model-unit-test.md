# T-01305: Init & Service Supervision - Data Model: Unit Test

## Metadata
- **Task ID:** `T-01305`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision Data Model Unit Tests (`code/aiosh-rust/aiosh-core::tests::test_service_data_model`)
- **Status:** Complete

---

## 1. Test Suite Overview
Created dedicated standalone automated integration test suite in `code/aiosh-rust/aiosh-core/tests/test_service_data_model.rs` validating invariants `SS1..SS5` across boundary conditions, negative failure modes, and serialization round-trips.

### Invariants Tested:
1. **`SS1` (Service Name Syntax & Boundaries)**:
   - `test_ss1_service_name_boundary_and_syntax`: Validates standard service names (`aios-securityd`, `dbus`, `auditd.service`), 1-char min boundary, 128-char max boundary. Asserts rejection of 129-char names, empty strings, leading symbols (`-service`, `.service`), whitespace, path separators (`/`, `\`), null bytes, and shell metacharacters (`;`, `&`, `|`, `>`, `<`, `$`).

2. **`SS2` (Execution Commands & Paths)**:
   - `test_ss2_exec_commands_and_working_dir`: Validates happy path execution string, 4096-char ceiling, rejection of empty `exec_start`, rejection of empty `exec_stop`/`exec_reload`, Unix/Windows absolute working directories, and rejection of relative paths and path traversal sequences (`..`).

3. **`SS3` (Dependency Hygiene & Acyclicity)**:
   - `test_ss3_dependency_hygiene`: Validates diverse dependency types (`Requires`, `Wants`, `After`, `Before`, `Conflicts`), rejection of self-dependency (`spec.name == dep.name`), rejection of duplicate dependencies, rejection of invalid dependency names, and 128 items dependency count limit.

4. **`SS4` (Resource & Field Limits)**:
   - `test_ss4_resource_and_field_limits`: Validates timeout bounds ($1 \le t \le 86400$), rejection of $t=0$ or $t > 86400$, description size limits (4096 bytes), environment map limits ($\le 256$ entries), empty key rejection, '=' key rejection, and Unix user/group name syntax.

5. **`SS5` (Status & Lifecycle Consistency)**:
   - `test_ss5_service_status_and_lifecycle_consistency`: Validates active running status, inactive stopped status, rejection of `state == Failed` when `health.healthy == true`, and rejection of `startup_mode == Masked` when `state == Active`.

6. **Serialization & Deserialization**:
   - `test_service_data_model_serde_roundtrip`: Verifies JSON round-trip equality for `ServiceSpec`, `ServiceStatus`, `ServiceQuery`, and snake_case `ServiceAction`.

---

## 2. Test Execution Results
```
running 6 tests
test test_service_data_model_serde_roundtrip ... ok
test test_ss2_exec_commands_and_working_dir ... ok
test test_ss1_service_name_boundary_and_syntax ... ok
test test_ss3_dependency_hygiene ... ok
test test_ss5_service_status_and_lifecycle_consistency ... ok
test test_ss4_resource_and_field_limits ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
