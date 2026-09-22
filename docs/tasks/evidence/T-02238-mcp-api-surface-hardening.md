# T-02238: Grant Lifecycle - MCP/API Surface: Hardening

## Metadata
- **Task ID:** `T-02238`
- **Subsystem:** Phase 2 — Policy Enforcement Point (PEP) Fabric & Security Kernel
- **Component:** Grant Lifecycle MCP/API Surface Hardening (`code/aiosh-rust/aiosh-mcp/src/main.rs`, `code/aiosh-mcp/tests/test_pep_grant_mcp.py`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic 4: Grant Lifecycle (8/10) — MCP/API Surface Hardening

---

## 1. Hardening Overview & Defenses Implemented

The MCP tool surface for Grant Lifecycle (`aios.pep.grant.*`) underwent defense-in-depth hardening across all 7 tool handlers:
1. `aios.pep.grant.issue`
2. `aios.pep.grant.list`
3. `aios.pep.grant.inspect`
4. `aios.pep.grant.validate`
5. `aios.pep.grant.revoke`
6. `aios.pep.grant.attenuate`
7. `aios.pep.grant.sweep`

### 1.1 Strict Path Hygiene & Traversal Defense
- **Path Sanitization**: Integrated `Server::validate_and_open_grant_store` and `Server::validate_and_open_grant_service` leveraging `aiosh_core::pep_decision_service::validate_pep_service_path`.
- **Enforcement Rules**:
  - Requires `.json` file extension.
  - Forbids parent directory traversal patterns (`..` components).
  - Enforces length bounds ($\le 1,024$ characters).
  - Rejects ASCII and Unicode control characters (`c.is_control()`).

### 1.2 File Size & Resource Ceilings
- **16 MiB Size Cap**: Before reading or deserializing any backing store file from disk, `std::fs::metadata` inspects file size and strictly rejects files $> 16\text{ MiB}$ ($16,777,216$ bytes) to prevent resource exhaustion and unauthenticated memory denial-of-service.
- **Capacity Limits**: Bounded by `MAX_GRANTS_IN_SERVICE` ($5,000$ active grants) to ensure deterministic memory bounds.

### 1.3 Input Bounds & Capability Rights Validation
- **Granular Right Parsing**: Explicit mapping of capability rights (`read`, `write`, `execute`, `delete`, `admin`, `delegate`) with fail-closed rejection for unrecognized or corrupted tokens.
- **Grant Validation Prior to Persistence**: `grant.validate()` is called prior to any store modification on `issue` or `attenuate`.
- **Pre-existing ID Collision Guard**: `issue` checks `store.get_grant(gid).is_some()` to prevent unintentional overwriting of existing grants.

### 1.4 Structured Error Envelopes & Audit Guarantees
- **ADR-0035 Compliance**: Every tool execution is mediated through `dispatch::recorded_call`, ensuring:
  - Policy decision evaluation against the active constitution revision.
  - Monotonic SQLite WAL record generation in the audit ring (`ring.db`).
  - Standardized JSON responses with `ok: true/false`, explicit error messages, and `audit_id`.

---

## 2. Verification & Test Suite

Hardening verification was executed using:
- `code/aiosh-mcp/tests/test_pep_grant_mcp.py`
- `code/aiosh-mcp/tests/test_pep_decision_smoke.py`
- Workspace compilation with `cargo check --workspace`

All tests passed with zero failures and zero compiler warnings.
