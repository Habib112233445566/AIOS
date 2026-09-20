# T-01834: Network Bootstrap / MCP/API Surface: Implementation

## 1. Overview
- **Task ID**: `T-01834`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Implement the minimal working behavior for the MCP/API surface of Network Bootstrap.

---

## 2. Implementation Details
1. **Service Resolution & Path Hygiene**:
   - Implemented `resolve_network_service` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Validates that `sysfs_path`, `procfs_path`, and `resolv_path` do not exceed 1024 characters and contain no control characters.
   - Defaults to `/sys/class/net`, `/proc/net`, `/etc/resolv.conf` when omitted.
2. **Tool Handlers in `call_tool`**:
   - `aios.network.list`: Scans interfaces and returns `{ interfaces, count }`.
   - `aios.network.show`: Validates `interface` with `validate_interface_name`, returns interface details or not found error.
   - `aios.network.routes`: Scans routing table and returns `{ routes, count }`.
   - `aios.network.dns`: Queries nameservers and search domains into `{ dns }`.
   - `aios.network.state`: Retrieves unified snapshot into `{ state }`.
   - `aios.network.up`: Validates interface name, calls `service.bring_up(name)`, returns `{ interface, status: "up" }`.
   - `aios.network.down`: Validates interface name, calls `service.bring_down(name)`, returns `{ interface, status: "down" }`.
3. **PEP & Audit Integration**:
   - All tool executions route through `dispatch::recorded_call(...)`, ensuring classifier evaluation, PEP checking, and audit row persistence.

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp` succeeded with 0 errors.
