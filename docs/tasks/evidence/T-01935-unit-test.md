# Task Evidence: T-01935 - System Update / MCP/API surface: Unit Test

- **Task**: `T-01935`
- **Sub-Epic**: `Sub-Epic 4: Model Context Protocol (MCP) & API Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Authored and executed unit tests for the System Update MCP tools in `code/aiosh-rust/aiosh-mcp/src/main.rs` (`test_system_update_mcp_tools`):
- **Tool Advertisement**: Verified presence of all 6 update tools (`aios.update.status`, `aios.update.slots`, `aios.update.check`, `aios.update.apply`, `aios.update.confirm`, `aios.update.rollback`) in `tool_manifest()`.
- **Path Hygiene Rejection**: Verified refusal of `state_dir` with control characters (`bad\x07state`) returning `ok: false`.
- **Status Query**: Verified `aios.update.status` returning `ok: true`, `state: "idle"`, `active_slot: "slot_a"`.
- **Slots Query**: Verified `aios.update.slots` returning `ok: true`, `current_slot: "slot_a"`, `target_slot: "slot_b"`.
- **Check Validation**: Verified missing parameters returning `ok: false`, and valid inline manifest returning `ok: true`, updating target version to `1.2.0` and state to `downloading`.
- **Confirm Validation**: Verified version length bound checking (`len > 64` returns `ok: false`), confirm in wrong state fails (`ok: false`), and confirm in `ReadyToReboot` succeeds (`ok: true`).
- **Rollback Execution**: Verified rollback in `ReadyToReboot` state restoring fallback slot (`slot_b`) with `ok: true`.

## Verification
- Executed `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp test_system_update_mcp_tools`.
- Output: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.28s`.
