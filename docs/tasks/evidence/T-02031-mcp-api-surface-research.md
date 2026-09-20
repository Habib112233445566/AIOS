# T-02031: Capability Model / MCP/API Surface — Research

**Task ID**: `T-02031`  
**Phase**: Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic**: Sub-Epic 4: Capability Model / MCP/API Surface  
**Status**: COMPLETED  
**Date**: 2026-09-20  

---

## 1. Executive Summary

This research document analyzes the design, constraints, and integration architecture for exposing the AIOS Capability Model (`aiosh_core::capability` and `aiosh_core::capability_service`) through the Model Context Protocol (MCP) server (`aiosh-mcp`).

The Model Context Protocol (MCP) serves as the sole external tool invocation protocol for AIOS (ADR-0035 §D-2). Exposing capability management over MCP enables autonomous agents, administrators, and runtime supervisors to inspect, issue, attenuate, check, revoke, and prune capabilities within a PEP-gated and audit-ring-logged framework.

---

## 2. Authoritative Sources & Prior Art

1. **ADR-0035 §D-2 (MCP Protocol Binding)**:
   - MCP (`stdio` JSON-RPC) is the exclusive external tool protocol exposed to AI models.
   - Every tool call must route through the classifier → Policy Enforcement Point (PEP) → Audit Ring gate via `dispatch::recorded_call`.
2. **ADR-0035 §A F-2 (PEP Invariant & Audit Binding)**:
   - Consequential state changes must emit an honest audit event before committing or failing.
   - Every action requires an audit row with SHA-256 hash chaining.
3. **Capability Architecture Specification (`docs/capability_model.md`)**:
   - Capabilities are cryptographically bound, unforgeable authorization tokens (`CAP-<uuid>`).
   - Monotonic attenuation invariant: child capabilities can never possess rights or scope broader than their parent.
   - Cascade revocation invariant: revoking a parent immediately invalidates the entire delegation subtree.
4. **Existing MCP Server Architecture (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - `Server::tool_manifest(&self) -> Vec<Value>` defines tool schemas using JSON Schema Draft-07.
   - `Server::call_tool(&mut self, tool: &str, arguments: &Value) -> Value` routes execution through `dispatch::recorded_call`.
   - Persistence pattern: state stores (e.g. `handoff_store.json`, `triage_store.json`) accept an optional `store_path` parameter defaulting to a canonical location in `.aios/`.

---

## 3. Facts vs Assumptions

### Facts
1. **Tool Invocation Routing**: All MCP tool executions must use `dispatch::recorded_call` to guarantee audit logging and PEP evaluation.
2. **Core Service Maturity**: `CapabilityService` is fully implemented in `aiosh_core::capability_service` with tested root issuance (`issue_root_capability`), attenuation (`attenuate_capability`), cascade revocation (`revoke_capability`), access checking (`check_access`), pruning (`prune_expired`), and atomic persistence (`save_to_path` / `load_from_path`).
3. **Storage Format**: Capability registries are stored as formatted JSON files with maximum size limit of 10 MB (`MAX_CAPABILITY_STORE_SIZE`) and path safety validation (`validate_service_path`).
4. **Tool Granularity**: MCP tool names follow the `aios.<subsystem>.<action>` dot-delimited namespace convention.

### Assumptions
1. **Standard Store Path**: Default backing store path for MCP capability tools is `.aios/capability_store.json`.
2. **Default Issuer Policy**: In MCP tool invocations, root capability issuance (`aios.capability.issue`) requires PEP authorization or `kernel`/`admin:*` issuer context.
3. **Tool Set Coverage**: 7 tools provide comprehensive lifecycle coverage:
   - `aios.capability.list`: List capabilities matching optional subject or active filters.
   - `aios.capability.get`: Retrieve capability details by ID.
   - `aios.capability.issue`: Issue root capability.
   - `aios.capability.attenuate`: Derive child capability with narrowed rights/scope.
   - `aios.capability.revoke`: Revoke capability and cascade to descendants.
   - `aios.capability.check`: Fast permission check for subject, scope, and right.
   - `aios.capability.prune`: Prune expired leaf capabilities.

---

## 4. Architectural Decisions & Unknowns Resolved

### Decision 1: Stateless per-call storage loading vs In-Memory Singleton
- **Context**: In `aiosh-mcp`, server instances may run across CLI/daemon lifecycles.
- **Decision**: Follow the proven `load_or_recover` / `load_or_create` pattern used by `handoff_service` and `triage_service`. Tools load the store from `store_path` (defaulting to `.aios/capability_store.json`), perform operations, and commit atomically to disk via `save_to_path` for state-modifying actions.

### Decision 2: Input Hygiene & Defensive Validation
- **Context**: Tool arguments originate from external LLM tool calls.
- **Decision**: Enforce string length bounds (IDs $\le 128$, subjects/issuers $\le 256$, paths $\le 1024$), reject control characters, reject path traversal (`..`), and validate enum right/scope strings strictly before passing to `CapabilityService`.

### Decision 3: Audit Ring & Dispatch Categorization
- **Context**: Different capability tools have different security impacts.
- **Decision**:
  - Read-only tools (`list`, `get`, `check`) are marked non-consequential in dispatch.
  - Mutating tools (`issue`, `attenuate`, `revoke`, `prune`) are marked consequential, requiring PEP grant authorization where configured, and recording target IDs in the audit ring.

---

## 5. Next Steps

Proceed to `T-02032` (Specification) to define JSON schemas, input properties, required fields, and response envelopes for the 7 MCP capability tools.
