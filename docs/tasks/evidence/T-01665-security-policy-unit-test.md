# T-01665: Security Policy Unit Test

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Add focused automated tests for the Kernel Module Management Security Policy in `code/aiosh-rust/aiosh-core/tests/test_kernel_module_policy.rs`.

## Unit Test Coverage
1. **`test_sp_km1_policy_bounds_and_disjointness`**:
   - Validates default policy and limits.
   - Asserts disjointness: fails if any module is both prohibited and protected.
   - Asserts module name syntax validation.
   - Asserts install command absolute path and traversal checks.
   - Asserts parameter length limits.
2. **`test_sp_km2_prohibited_module_enforcement`**:
   - Asserts blacklisting a prohibited module is allowed.
   - Asserts configuring options or aliases for a prohibited module is rejected.
   - Asserts autoloading a prohibited module is rejected.
   - Asserts autoloading an allowed module succeeds.
3. **`test_sp_km3_protected_module_guard`**:
   - Asserts blacklisting protected modules (`ext4`, `dm_mod`) is blocked.
   - Asserts disabling protected modules (`overlay`) via install `/bin/false` or `/bin/true` is blocked.
   - Asserts non-protected modules can be blacklisted normally.
4. **`test_sp_km4_install_command_sanitization`**:
   - Asserts approved install commands (`/bin/true`) pass.
   - Asserts unapproved binaries (`/usr/bin/python3`) are blocked.
   - Asserts command injection attempts (`;`, `&`, `|`, `` ` ``, `$`, `..`) are blocked.
5. **`test_sp_km5_parameter_inspection_and_bounds`**:
   - Asserts disallowed parameter keys (`panic=1`, `init=`, `rdinit=`) are blocked.
   - Asserts parameter length bounds enforcement.
   - Asserts dangerous substring / metacharacter detection.
   - Asserts valid module options pass cleanly.
6. **`test_sp_km6_tri_state_modes_and_store_evaluation`**:
   - Asserts `Enforcing` mode blocks on fatal violations.
   - Asserts `Audit` mode allows mutations while capturing structured violation records.
   - Asserts `Permissive` mode allows non-critical violations while continuing to block protected module destruction and command injection.
   - Asserts full store evaluation across rules and autoload entries.
   - Asserts environment variable parsing via `from_source`.

## Verification
- Run via `cargo test -p aiosh-core --test test_kernel_module_policy`.
- All 6 tests pass.
