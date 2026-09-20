# Task Evidence: T-01936 - System Update / MCP/API surface: Integration

- **Task**: `T-01936`
- **Sub-Epic**: `Sub-Epic 4: Model Context Protocol (MCP) & API Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Integrated the System Update MCP tools into the production `aiosh-mcp` service and verified cross-surface parity with the operator CLI:
- **Production Server Registration**: Exposed all 6 `aios.update.*` tools (`aios.update.status`, `aios.update.slots`, `aios.update.check`, `aios.update.apply`, `aios.update.confirm`, `aios.update.rollback`) via JSON-RPC `tools/list`.
- **JSON-RPC Dispatch**: Integrated `resolve_update_service()` and `call_tool()` handlers in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- **Integration Smoke Suite**: Authored `code/aiosh-mcp/tests/test_system_update_mcp_smoke.py`:
  1. Discovery of all 6 update tools via `tools/list`.
  2. Input sanitization: rejection of control characters in `state_dir`, missing manifest in `aios.update.check`, oversized version strings (> 64 chars) in `aios.update.confirm`.
  3. Status and Slots queries against isolated state directory.
  4. State transition to `downloading` via `aios.update.check` with inline manifest.
  5. Enforcement of state prerequisites: rejection of `confirm` in non-ReadyToReboot state.
  6. Transition to `reboot_pending` upon valid `confirm` in `ReadyToReboot`.
  7. Safe rollback from `ReadyToReboot` returning fallback slot (`slot_a`).
  8. Cross-surface state parity: CLI `aiosh update status` and MCP `aios.update.status` return identical `state` and `active_slot`.

## Verification
- Executed `python code/aiosh-mcp/tests/test_system_update_mcp_smoke.py`:
  - `ALL 7 SYSTEM UPDATE MCP SMOKE CHECKS PASSED.`
- Executed `python -m pytest code/aiosh-mcp/tests/test_system_update_mcp_smoke.py`:
  - `1 passed in 0.65s` (100% pass).
