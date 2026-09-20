# T-01835: Network Bootstrap / MCP/API Surface: Unit Test

## 1. Overview
- **Task ID**: `T-01835`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Add focused unit tests for the MCP/API surface of Network Bootstrap.

---

## 2. Test Implementation
Added `test_network_mcp_surface` in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
1. **Schema Discovery (`NMCP1`)**:
   - Asserts `tool_manifest` contains all 7 network tools: `aios.network.list`, `show`, `routes`, `dns`, `state`, `up`, `down`.
2. **Path Hygiene Validation (`NMCP2`)**:
   - Verifies `sysfs_path` length > 1024 fails (`ok: false`).
   - Verifies `procfs_path` containing control characters fails (`ok: false`).
   - Verifies `resolv_path` containing control characters fails (`ok: false`).
3. **Input & Argument Validation (`NMCP3`)**:
   - Missing required `interface` argument for `show`, `up`, `down` fails (`ok: false`).
   - Invalid interface names (e.g. `eth0;evil`) failing `validate_interface_name` return `ok: false`.
4. **Mock Execution Flow**:
   - Constructs isolated mock sysfs (`eth0`), procfs (`route`), and `resolv.conf`.
   - Tests `aios.network.list`, `show`, `routes`, `dns`, `state` verifying returned payloads.
   - Tests `aios.network.up` and `aios.network.down` verifying state transitions.
   - Verifies interface not found (`eth99`) returns `ok: false`.

## 3. Verification
- Test executed via `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp -- test_network_mcp_surface`.
