# Task Evidence: T-02231 (Grant Lifecycle / MCP/API surface: Research)

## 1. Metadata
- **Task ID:** `T-02231`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle MCP/API Surface Research (`code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic 4: Grant Lifecycle MCP Surface (4/10) — Research

---

## 2. Executive Summary & Existing State Research

The AIOS Model Context Protocol (MCP) server (`code/aiosh-rust/aiosh-mcp/src/main.rs`) exposes tool calls over JSON-RPC 2.0 stdio transport adhering to ADR-0035 §D-2.

### 2.1 Inventory of Existing MCP Tools
Inspection of `code/aiosh-rust/aiosh-mcp/src/main.rs` reveals existing partial grant lifecycle tools:
- `aios.pep.grant.list`: List grants with optional subject filter.
- `aios.pep.grant.inspect`: Inspect full details of a grant by ID.
- `aios.pep.grant.validate`: Validate grant authorization and temporal validity.
- `aios.pep.grant.revoke`: Revoke grant with audit reason and optional cascade.
- `aios.pep.grant.attenuate`: Derive child grant with attenuated rights and depth decrement.
- `aios.pep.grant.sweep`: Sweep expired and quota-exhausted grants.

### 2.2 Gap Analysis: Parity with CLI Surface
While query, derivation, and revocation tools exist, root grant creation is currently absent from MCP:
- The CLI surface provides `aiosh pep grant issue` for administrative issuance of root parent grants.
- The MCP surface currently lacks an equivalent tool (`aios.pep.grant.issue`), forcing smoke tests (`code/aiosh-mcp/tests/test_pep_decision_smoke.py`) to manually seed the test store using ad-hoc JSON file writes.
- Providing `aios.pep.grant.issue` completes full functional parity across CLI and MCP surfaces, allowing autonomous agents to administer and manage authorization grants strictly through MCP tool calling.

---

## 3. Authoritative Sources & Prior Art

1. **Model Context Protocol Specification (2025-06-18)**:
   - Tool calling JSON-RPC 2.0 protocol format: `{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "...", "arguments": {...}}}`.
   - Standardized error responses: `{"jsonrpc": "2.0", "id": 1, "error": {"code": -32602, "message": "..."}}`.
   - Tool registration manifest schema: `tools/list` returns name, description, and JSON Schema `inputSchema`.
2. **ADR-0035 (Audit Ring & PEP Invariants)**:
   - §D-2: MCP is the sole external model tool-call interface.
   - §F-2: All mutating and security-critical calls must commit an immutable audit record via `dispatch::recorded_call`.
3. **AIOS PEP Decision Engine Specification (`docs/pep_decision_engine.md`)**:
   - Section 16: Core service grant management API (`PepGrantService`, `PepGrantStore`).
   - Section 17: CLI surface command specifications.

---

## 4. Facts vs Assumptions

| Item | Fact | Assumption |
|:---|:---|:---|
| **Transport** | JSON-RPC 2.0 over standard I/O streams (`stdio`) | None |
| **Audit Requirement** | Every tool invocation must emit a row into the SQLite audit ring via `dispatch::recorded_call` | None |
| **Storage Engine** | Grants persist in atomic JSON stores validated by `aiosh_core::pep_grant` | None |
| **Root Issuance Policy** | Root grants must define mandatory ID, issuer, subject, scope type, and rights subset | Administrative authorization gating will be handled via PEP decision checks |
| **Error Format** | Return structured envelope `{"ok": false, "error": "..."}` or JSON-RPC error | Envelope matches existing `aios.pep.grant.*` tools |

---

## 5. Decisions Needed Before Implementation

1. **Tool Naming**: Should root issuance be named `aios.pep.grant.issue` or `aios.pep.grant.create`?
   - *Decision*: Name it `aios.pep.grant.issue` to match the CLI surface (`aiosh pep grant issue`) and data model terminology.
2. **Argument Schema**: What parameters should `aios.pep.grant.issue` accept?
   - *Decision*:
     - Required: `id`, `subject`, `scope_type`, `rights`
     - Optional: `issuer` (defaults to `"mcp-agent"`), `scope_path`, `delegation_depth` (default 0), `expires_at`, `not_before`, `max_invocations`, `max_bytes`, `store_path`
3. **Audit Classification**:
   - Action name: `"aios.pep.grant.issue"`
   - Tool description: `"Issue a new root PEP authorization grant"`
   - Emits exactly one audit row with SHA-256 integrity linking.

---

## 6. Acceptance Confirmation
- [x] Authoritative code, documentation, and RFCs reviewed.
- [x] Clear separation of facts vs assumptions documented.
- [x] Missing `aios.pep.grant.issue` identified and architectural decisions established.
- [x] Zero code changes performed in research phase.
