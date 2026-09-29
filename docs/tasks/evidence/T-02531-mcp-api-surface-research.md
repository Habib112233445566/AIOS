# T-02531: Privilege Escalation Prevention MCP/API Surface Research

- **Task**: `T-02531`
- **Sub-Epic**: Privilege Escalation Prevention / MCP/API surface
- **Date**: 2026-09-29
- **Status**: PASSED

## 1. Research Objectives
Establish authoritative technical facts, architectural patterns, input schemas, and security boundaries for exposing Privilege Escalation Prevention capabilities via Model Context Protocol (MCP) JSON-RPC 2.0 tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Existing Substrate & Prior Art Analysis
1. **Core Substrate**:
   - `aiosh_core::privilege` module implements the complete privilege data model (`PrivilegeLevel`, `PrivilegeCapability`, `PrivilegeContext`, `PrivilegeEvaluation`, `PrivilegeDecision`, `ElevationGrant`).
   - `aiosh_core::privilege_service` provides stateful context management (`PrivilegeService`) supporting multi-actor context storage, persistence, elevation validation, safe de-escalation, capability checks, and defense-in-depth kernel protection.
   - `aiosh-cli` provides shell commands under `aiosh privilege {status, elevate, drop, revoke, check, list}`.
2. **Missing MCP Surface**:
   - Autonomous LLM agents operating through Model Context Protocol (MCP) require programmatic RPC tools to inspect their active privilege tier, request authorized privilege elevations (with cryptographic PEP grant tokens), safely drop elevated privileges, revoke dynamic grants, and check specific capability holdings.
3. **MCP Server Integration Patterns in `aiosh-mcp`**:
   - Tools are declared in the `tools/list` schema handler in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Tool calls are routed through `dispatch::recorded_call` to guarantee audit trail emission to `audit_ring`.
   - Strict JSON Schema draft-07 input schemas validate inputs and reject unknown properties (`additionalProperties: false`).
   - Consistent JSON envelope returns: `{"code": i32, "data": Value, "error": Value}`.

## 3. Facts vs Assumptions
- **Fact**: MCP tools must be non-blocking, strictly bounded in memory, and reject malformed JSON.
- **Fact**: Any elevation request targeting `SystemKernel` MUST be immediately rejected with an explicit security violation error, preserving the invariant that `SystemKernel` is immutable and inaccessible to dynamic user-space elevations.
- **Fact**: Actor IDs and Grant IDs must be bounded in length ($\le 128$ bytes for actor, $\le 256$ bytes for grant) and free of control characters.
- **Fact**: Capability lists must be bounded to $\le 32$ items.
- **Assumption**: The MCP server maintains an in-memory/persisted `PrivilegeService` instance, defaulting to actor `"mcp-agent"` when no explicit actor ID is supplied.

## 4. Key Design Decisions for Specification (T-02532)
1. Expose 5 dedicated MCP tools under namespace `aios.privilege.*`:
   - `aios.privilege.status`: Query active context, tier, and capabilities.
   - `aios.privilege.elevate`: Request dynamic elevation with grant token validation.
   - `aios.privilege.drop`: Voluntarily de-escalate privilege level.
   - `aios.privilege.revoke`: Restore baseline tier and strip dynamic grants.
   - `aios.privilege.check`: Check whether a specific capability is held.
2. Standard error taxonomy:
   - `ERR_PRIVESC_ACTOR_NOT_FOUND`: Specified actor context not found.
   - `ERR_PRIVESC_INVALID_TIER`: Target tier unrecognized or malformed.
   - `ERR_PRIVESC_KERNEL_TIER_IMMUTABLE`: Unauthorized attempt to elevate to SystemKernel.
   - `ERR_PRIVESC_GRANT_REQUIRED`: Elevation to higher tier without valid grant token.
   - `ERR_PRIVESC_INVALID_INPUT`: Bounds violation (payload, string length, control characters).
