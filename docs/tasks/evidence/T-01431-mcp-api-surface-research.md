# T-01431: User Session Bootstrap - MCP/API Surface: Research

## Metadata
- **Task ID:** `T-01431`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Research (`code/aiosh-rust/aiosh-mcp`, `code/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (1/10) — MCP/API Surface Research

---

## 1. Objectives & Scope

Research the autonomous AI agent tool-call interface (`aios.session.*`) over the Model Context Protocol (MCP) for validating, discovering, inspecting, manipulating lifecycle state, and bootstrapping user and agent sessions on AIOS.

The research establishes:
1. **MCP Standard Alignment**: Adherence to the Model Context Protocol specification (JSON-RPC 2.0 over stdio) per ADR-0035 §D-2.
2. **Existing Tool Surface Audit**: Inventory and evaluation of existing tools in `code/aiosh-rust/aiosh-mcp/src/main.rs` and Python bridge.
3. **Parity with CLI & Capabilities**: Identification of feature gaps between the operator CLI (`aiosh session`) and agent MCP tool surface (notably `create` and `reason`).
4. **Defensive Hardening & Input Bounds**: Enforcement of memory ceilings (1 MiB payload), path bounds ($\le 1,024$ bytes, control character rejection), query pagination bounds ($1 \le \text{limit} \le 10,000$), and structured error reporting.
5. **Authorization & PEP Gate**: Alignment with AI_CONSTITUTION §1.4 C-1..C-3 and ADR-0035 §D-4 regarding capability grants and audit trail generation.

---

## 2. Authoritative Sources & Prior Art

- **Model Context Protocol (MCP) Specification (Anthropic, 2024-11-05)**:
  - Standard JSON-RPC 2.0 transport over bidirectional stdio streams.
  - `tools/list`: Returns array of tool definitions with standard JSON Schema `inputSchema` specifications.
  - `tools/call`: Executes tool with validated arguments and returns structured content with non-fatal or fatal error indicators.
- **`systemd-logind` D-Bus Control Surface (`org.freedesktop.login1.Manager`, `Session`, `Seat`)**:
  - Authoritative Linux session manager API providing:
    - `CreateSession(uid, pid, service, type, class, c seat, vtnr, ...)`
    - `ListSessions()`
    - `GetSession(session_id)`
    - `ActivateSession(session_id)`
    - `LockSession(session_id)` / `UnlockSession(session_id)`
    - `TerminateSession(session_id)`
- **RFC 8259 (JSON Data Interchange Format)** & **RFC 7049 (CBOR/Binary Canonicalization)**:
  - Format standards for JSON payload interchange and deterministic key sorting.
- **AIOS Architecture Decisions & Constitution**:
  - **ADR-0035 §D-2**: MCP is the exclusive tool-call protocol for AI agents.
  - **ADR-0035 §D-4**: PEP capability authorization gating on all tool dispatch calls.
  - **AI_CONSTITUTION.md §1.4 C-1..C-3**: Irreversible actions require explicit PEP grants; non-repudiation audit row generation into SQLite WAL ring buffer.

---

## 3. Fact vs. Assumption Separation

### Established Facts
- **Fact 1 (Underlying Subsystem Readiness)**:
  - `code/aiosh-rust/aiosh-core::session` and `session_service` implement all required session logic (`UserSessionSpec`, `UserSessionService`, `apply_action`, `create_session`, `query_sessions`, `save_to_path`, `load_from_path`) and enforce invariants `SB1..SB5` and `CS1..CS5`.
- **Fact 2 (Existing MCP Tools in Rust Server)**:
  - `code/aiosh-rust/aiosh-mcp/src/main.rs` registers:
    - `aios.session.validate`: Validates `session_id`, `username`, or `spec`.
    - `aios.session.list`: Filters sessions by `username`, `state`, `session_type`, `seat`, `limit`, `store_path`.
    - `aios.session.get`: Retrieves status and specification for a specific session ID.
    - `aios.session.action`: Applies lifecycle action (`authenticate`, `activate`, `lock`, `unlock`, `terminate`) with seat arbitration.
- **Fact 3 (Existing Dispatch & Audit Gate)**:
  - Every MCP tool call routes through `dispatch::recorded_call(&mut ring, &pep, ...)` emitting a hash-chained audit record with non-repudiation SHA-256 integrity into `$AIOSH_HOME/audit.db`.
- **Fact 4 (CLI Parity Gap)**:
  - While `aiosh session create <spec>` was implemented in Sub-Epic 3 (CLI surface), there is currently no corresponding `aios.session.create` tool exposed in the MCP tool manifest.
- **Fact 5 (Existing Hardening Gaps in MCP Handlers)**:
  - In `aios.session.list`, the `limit` argument is parsed via `.as_u64()` without enforcing upper bound ($10,000$) or non-zero checks.
  - In `aios.session.get` and `aios.session.action`, `session_id` is not validated for length ($\le 64$) or control character rejection prior to service lookup.
  - Custom `store_path` arguments across session tools lack length bounds ($\le 1,024$) and control character validation.

### Engineering Assumptions
- **Assumption 1 (Autonomous Session Creation)**:
  - Autonomous S-rank AI agents require the ability to provision and bootstrap dedicated agent sessions (`session_type: "ai_agent"`, `session_class: "agent"`) via `aios.session.create`.
- **Assumption 2 (Strict Input Schema & JSON Validation)**:
  - Declaring a structured JSON Schema for `spec` in `aios.session.create` and accepting both parsed JSON objects and inline strings (capped at 1 MiB) provides maximum LLM ergonomics.
- **Assumption 3 (Action Reason Tracking)**:
  - Extending `aios.session.action` to accept an optional `reason` parameter allows agents to document rationale (e.g., "Terminated due to inactivity timeout") directly into the SQLite WAL audit trail.
- **Assumption 4 (Cross-Language Smoke Testing)**:
  - Adding a Python smoke test `code/aiosh-mcp/tests/test_session_mcp_smoke.py` (analogous to `test_service_mcp_smoke.py`) will guarantee cross-substrate reliability between Python MCP clients and the Rust MCP server.

---

## 4. Proposed MCP Tool Surface Inventory

| Tool Name | Operation | Input Parameters | Return Object | PEP Gated |
|---|---|---|---|---|
| `aios.session.validate` | Validate identifier or spec | `session_id?`, `username?`, `spec?`, `grant_id?` | `{ ok, valid, session_id?, username?, spec? }` | Yes (Read-only) |
| `aios.session.list` | Query active/tracked sessions | `username?`, `state?`, `session_type?`, `seat?`, `limit?`, `store_path?`, `grant_id?` | `{ ok, count, sessions: [...] }` | Yes (Read-only) |
| `aios.session.get` | Inspect session & spec | `session_id`, `store_path?`, `grant_id?` | `{ ok, session_id, status, spec }` | Yes (Read-only) |
| `aios.session.action` | Transition lifecycle state | `session_id`, `action`, `reason?`, `store_path?`, `grant_id?` | `{ ok, report: { session_id, action, previous_state, new_state, success, ... } }` | Yes (State mutation) |
| `aios.session.create` | Bootstrap new session | `spec`, `store_path?`, `grant_id?` | `{ ok, session_id, spec, status }` | Yes (Resource provisioning) |

---

## 5. Defensive Sizing & Security Constraints

1. **Payload Bounds**:
   - `spec` object or inline string in `aios.session.validate` and `aios.session.create` must not exceed `1,048,576` bytes (1 MiB).
2. **String Limits**:
   - `session_id`: $[1 \dots 64]$ chars, no ASCII control characters.
   - `username`: $[1 \dots 32]$ chars, no ASCII control characters.
   - `store_path`: $\le 1,024$ chars, no ASCII control characters.
   - `reason`: $\le 512$ chars.
3. **Query Bounds**:
   - `limit`: constrained to $[1 \dots 10,000]$.
4. **Audit Integrity**:
   - Every tool call (successful or rejected) writes a SHA-256 hash-chained event to `audit.db`.

---

## 6. Decisions & Unknowns for Specification Phase (T-01432)

1. **Decision 1 (`aios.session.create` Tool Specification)**:
   - Formally specify `aios.session.create` in the tool manifest with full `inputSchema`, argument validation, and invocation contracts.
2. **Decision 2 (`aios.session.action` Reason Parameter)**:
   - Add optional `reason` parameter to `aios.session.action` schema and forward into `dispatch::recorded_call` target/summary.
3. **Decision 3 (Handler Hardening)**:
   - Harden all session tool handlers in `code/aiosh-rust/aiosh-mcp/src/main.rs` with explicit parameter bounds checks matching the CLI surface.
4. **Decision 4 (Test Suite Integration)**:
   - Create `code/aiosh-mcp/tests/test_session_mcp_smoke.py` and update `tools/test_session_suites.py` criterion `SB3` to execute both in-tree Rust and cross-substrate MCP smoke tests.

---

## 7. Acceptance Verification
- [x] Authoritative sources consulted (`loginctl`, MCP specification, RFC 8259, AI_CONSTITUTION, ADR-0035).
- [x] Facts strictly separated from assumptions.
- [x] No source code changed during research phase.
- [x] Explicit decisions and unknowns cataloged for specification phase.
