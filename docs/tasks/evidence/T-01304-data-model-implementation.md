# T-01304: Init & Service Supervision - Data Model: Implementation

## Metadata
- **Task ID:** `T-01304`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision Data Model Implementation (`code/aiosh-rust/aiosh-core::service`)
- **Status:** Complete

---

## 1. Implementation Summary
Implemented complete validation logic for the Init & Service Supervision data model in `code/aiosh-rust/aiosh-core/src/service.rs`, enforcing invariants `SS1..SS5`.

### Implemented Validation Functions:
1. **`validate_service_name(name: &str) -> Result<(), String>`**:
   - Enforces `SS1`: Non-empty, $\le 128$ characters, begins with ASCII alphanumeric, only allowed characters `[a-zA-Z0-9_.-]`.
   - Rejects spaces, path slashes, null bytes, and shell metacharacters.

2. **`validate_service_spec(spec: &ServiceSpec) -> Result<(), Vec<String>>`**:
   - Validates `SS1` identifier syntax on `spec.name`.
   - Validates `SS2` execution commands and working directory (non-empty `exec_start`, bounds $\le 4096$, absolute path requirement, traversal sequence `..` rejection).
   - Validates `SS3` dependency hygiene (bounds $\le 128$, acyclic self-dependency rejection, duplicate rejection, dependency name syntax).
   - Validates `SS4` resource limits (timeouts in `[1, 86400]`, env map $\le 256$ entries, env keys free of `=` and null bytes, descriptions $\le 4096$, user/group names $\le 64$).
   - Validates `SS5` lifecycle consistency (masked service cannot be enabled).

3. **`validate_service_status(status: &ServiceStatus) -> Result<(), Vec<String>>`**:
   - Validates `status.name` per `validate_service_name`.
   - Rejects contradictory state where `state == ServiceState::Failed` but `health.healthy == true`.
   - Rejects contradictory state where `startup_mode == ServiceStartupMode::Masked` but `state == ServiceState::Active`.

---

## 2. Test Verification
All unit tests in `service.rs` passed with 0 failures:
- `test_valid_service_spec_happy_path ... ok`
- `test_service_naming_syntax_ss1 ... ok`
- `test_exec_commands_and_working_dir_ss2 ... ok`
- `test_dependency_hygiene_ss3 ... ok`
- `test_resource_bounds_ss4 ... ok`
- `test_status_consistency_ss5 ... ok`
- Zero regressions across existing test suite (97 tests passing).
