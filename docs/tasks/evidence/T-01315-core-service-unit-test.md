# T-01315: Init & Service Supervision - Core Service: Unit Test

## Metadata
- **Task ID:** `T-01315`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service_service`
- **Component:** Init & Service Supervision Core Service Unit Tests
- **Status:** Complete

## 1. Test Suite Architecture
Created automated integration and unit test suite in [code/aiosh-rust/aiosh-core/tests/test_service_service.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/tests/test_service_service.rs) verifying invariants `CS1..CS5`:

1. **`test_service_store_seeding_and_lookup`**:
   - Asserts initial state of pre-seeded reference services (`auditd.service`, `dbus.service`, `systemd-journald.service`, `network-manager.service`, `aios-securityd.service`, `ssh.service`).
   - Verifies all units have valid specs and corresponding `Active` statuses with non-empty PIDs and healthy telemetry.
   - Validates that non-existent service queries return `None`.

2. **`test_service_store_cs1_uniqueness_and_lifecycle`**:
   - Registers valid custom service (`aios-telemetry.service`).
   - Verifies initial status defaults to `Inactive` with `pid: None` and matching `startup_mode`.
   - Rejects duplicate service registration with formal `CS1` violation error.
   - Rejects invalid specs failing syntax validation (e.g. spaces in service names).
   - Verifies `unregister_service` cleans up both spec and runtime status.
   - Verifies unregistering non-existent units errors gracefully.

3. **`test_service_store_cs2_fsm_lifecycle_actions`**:
   - `Start`: Rejects masked units; transitions `Inactive` $\to$ `Active`, assigns PID 1001, sets `started_at`.
   - `Stop`: Transitions `Active` $\to$ `Inactive`, clears PID and `started_at`.
   - `Restart`: Reboots unit to `Active`, assigns PID 1002, increments restart counter.
   - `Reload`: Succeeds on `Active` unit; errors on `Inactive` unit.
   - `Enable` / `Disable`: Modifies `startup_mode` on both `ServiceSpec` and `ServiceStatus`.
   - `Mask`: Strictly prohibited on `Active` units (errors); succeeds once stopped, setting `startup_mode = Masked`.
   - `Unmask`: Restores `Masked` units to `Disabled`.

4. **`test_service_store_cs3_topological_ordering_and_cycle_detection`**:
   - Resolves transitive dependencies for `aios-securityd.service` (`auditd.service` and `dbus.service` precede `aios-securityd.service`).
   - Resolves multi-tier chain for `ssh.service` (`dbus.service` $\to$ `network-manager.service` $\to$ `ssh.service`).
   - Detects unmet external dependencies with explicit error envelope.
   - Invariant `CS3` cycle detection: generates mutually dependent units (`cycle-a.service` and `cycle-b.service`) and confirms Kahn's algorithm detects the cycle and returns `invariant CS3 violated: cyclic dependency detected`.

5. **`test_service_store_query_matrix`**:
   - Evaluates queries filtering by `name_pattern` (name and description substrings).
   - Evaluates queries filtering by `startup_mode` (`Static`, `Enabled`).
   - Evaluates query result truncation with `limit`.

6. **`test_service_store_cs5_persistence_and_bounds`**:
   - Verifies atomic persistence via `save_to_path` and `load_from_path`.
   - Verifies round-trip equality of services and statuses.
   - Verifies missing files return explicit error.
   - Enforces 10 MiB disk read ceiling rejection on 11 MiB payload.

## 2. Test Execution & Output
Executed test suite standalone via `cargo test --test test_service_service`:
```text
running 6 tests
test test_service_store_cs2_fsm_lifecycle_actions ... ok
test test_service_store_cs3_topological_ordering_and_cycle_detection ... ok
test test_service_store_cs1_uniqueness_and_lifecycle ... ok
test test_service_store_seeding_and_lookup ... ok
test test_service_store_query_matrix ... ok
test test_service_store_cs5_persistence_and_bounds ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Integrated criterion `SS4` into master runner [tools/test_service_suites.py](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/tools/test_service_suites.py):
```text
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```
