# T-01465: User Session Bootstrap — Security Policy: Unit Test

## Metadata
- **Task ID:** `T-01465`
- **Subsystem:** `code/aiosh-rust/aiosh-core`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`
- **Component:** User Session Bootstrap Security Policy Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Unit Test Deliverables

Executed standalone unit test suites validating positive and negative security policy boundaries across all layers:

1. **`code/aiosh-rust/aiosh-core/tests/test_session_policy.rs` (8 tests)**:
   - `test_default_policy_validation`: Verifies default policy struct conforms to all structural bounds.
   - `test_ssp1_root_and_greeter_rules`: Tests rejection of UID 0 root sessions and unprivileged greeter sessions, and acceptance of authorized greeter.
   - `test_ssp2_session_type_and_class_rules`: Tests rejection of Agent class with non-agent session type, greeter without display, and greeter with remote host.
   - `test_ssp3_seat_and_display_rules`: Tests rejection of remote session on console `seat0`, allowance of remote session on `seat-remote`, and rejection of invalid VT numbers (> 64).
   - `test_ssp4_environment_sanitization`: Tests rejection of `LD_PRELOAD`, allowance of standard variables (`LANG`, `TERM`).
   - `test_ssp5_concurrency_quotas`: Tests per-user active session quota enforcement across store.
   - `test_ssp6_agent_sandboxing`: Tests rejection of privileged AI agent execution (UID 0) and allowance of non-root agent execution.
   - `test_ssp7_policy_modes_and_file_caps`: Verifies mode behavior differences across `Enforcing`, `Audit`, and `Permissive`.

2. **`code/aiosh-rust/aiosh-cli/src/main.rs` (`test_cmd_session_flow`)**:
   - Tested `aiosh session policy` (default store evaluation: exit code 0).
   - Tested `aiosh session policy --json` (JSON envelope output).
   - Tested `aiosh session policy --spec <valid_json>` (exit code 0).
   - Tested `aiosh session policy --spec <root_json>` (exit code 1 on policy violation).

3. **`code/aiosh-rust/aiosh-mcp/src/main.rs` (`test_mcp_session_validate_tools`)**:
   - Tested `aios.session.policy` tool discovery.
   - Tested store evaluation over MCP (returns `ok: true, allowed: true`).
   - Tested spec evaluation with conforming session (returns `allowed: true`).
   - Tested spec evaluation with root session (returns `allowed: false`).

## 2. Command Output Summary
```text
cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_session_policy
running 8 tests
test test_default_policy_validation ... ok
test test_ssp1_root_and_greeter_rules ... ok
test test_ssp2_session_type_and_class_rules ... ok
test test_ssp3_seat_and_display_rules ... ok
test test_ssp4_environment_sanitization ... ok
test test_ssp5_concurrency_quotas ... ok
test test_ssp6_agent_sandboxing ... ok
test test_ssp7_policy_modes_and_file_caps ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```
