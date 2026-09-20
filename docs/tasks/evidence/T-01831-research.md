# T-01831: Network Bootstrap / MCP/API Surface: Research

## 1. Overview
- **Task ID**: `T-01831`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Research facts, constraints, prior art, and architecture for the MCP/API surface of Network Bootstrap.

---

## 2. Research Findings: Facts vs Assumptions

### A. Authoritative Facts
1. **MCP Server Architecture in AIOS**:
   - `code/aiosh-rust/aiosh-mcp/src/main.rs` serves as the primary Rust MCP server handling JSON-RPC 2.0 requests over stdio.
   - Tools are listed in `handle_tools_list()` returning `name`, `description`, and `inputSchema`.
   - Tool execution in `handle_tools_call()` is governed by:
     - Input validation.
     - PEP (Policy Enforcement Point) gating.
     - Audit row emission via `emit_audit_event()`.
     - Standard MCP response formatting: `{"content": [{"type": "text", "text": "<json_string>"}]}`.
2. **Domain Service Capabilities (`NetworkService`)**:
   - `aiosh_core::network_service::NetworkService` provides:
     - `scan_interfaces()` -> `Vec<NetworkInterface>`
     - `get_interface(name)` -> `Option<NetworkInterface>`
     - `scan_routes()` -> `Vec<Route>`
     - `get_dns_config()` -> `DnsConfig`
     - `get_network_state()` -> `NetworkState`
     - `bring_up(name)` -> `Result<(), String>`
     - `bring_down(name)` -> `Result<(), String>`
   - Supports configurable mock filesystem paths via `NetworkService::with_paths(sysfs, procfs, resolv)`.
3. **Security Invariants & Prior Art**:
   - Similar subsystems (`hardware`, `kernel_module`, `service`) expose inspection tools as non-grant-required tools with full audit logging.
   - State-changing tools (`up`, `down`) must enforce strict interface name validation and emit consequential audit rows.

### B. Assumptions
1. Tool naming conventions should follow `aios.network.<action>`:
   - `aios.network.list`: Discover all network interfaces.
   - `aios.network.show`: Query detailed attributes for a single interface.
   - `aios.network.routes`: Query IPv4 routing table.
   - `aios.network.dns`: Query DNS nameservers and search domains.
   - `aios.network.state`: Full snapshot of host networking.
   - `aios.network.up`: Bring interface up.
   - `aios.network.down`: Bring interface down.
2. Tools should accept optional `sysfs_path`, `procfs_path`, `resolv_path` to support hermetic execution and testing in non-Linux or sandboxed environments.

---

## 3. Invariants for MCP/API Surface (`NMCP1..NMCP6`)

- **`NMCP1` (Schema Completeness)**: Every tool has a valid JSON Schema 2020-12 / Draft 7 inputSchema with explicit property types and descriptions.
- **`NMCP2` (Path Sanitization)**: Any provided path overrides (`sysfs_path`, `procfs_path`, `resolv_path`) must be $\le 1024$ characters and contain no control characters.
- **`NMCP3` (Interface Name Validation)**: Interface name inputs must satisfy `validate_interface_name` ($\le 15$ chars, `^[a-zA-Z0-9_.-]+$`).
- **`NMCP4` (PEP & Mutation Controls)**: Mutation operations (`up`, `down`) require explicit validation and are classified as consequential actions.
- **`NMCP5` (Audit Logging)**: Every invocation writes an audit record with timestamp, actor, tool name, arguments, and outcome.
- **`NMCP6` (Deterministic Serialization)**: Tool outputs return JSON payloads serialized from canonical Rust data structures.

---

## 4. Unknowns & Decisions Needed
1. **Grant Requirement for Link Mutation**:
   - *Decision*: In alignment with other hardware/network tools in development, `up` and `down` will emit consequential audit records and require valid interface names.
2. **Path Parameter Exposure**:
   - *Decision*: Expose `sysfs_path`, `procfs_path`, and `resolv_path` as optional parameters so automated test suites can verify the MCP surface offline.
