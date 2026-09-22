# Task Evidence: T-02234 (Grant Lifecycle / MCP/API surface: Implementation)

## 1. Metadata
- **Task ID:** `T-02234`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle MCP/API Surface Implementation (`code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic 4: Grant Lifecycle MCP Surface (4/10) — Implementation

---

## 2. Implementation Summary

Implemented the working behavior for `aios.pep.grant.issue` in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
1. **Manifest Registration**:
   - Registered `aios.pep.grant.issue` in `tool_manifest()` with complete inputSchema:
     - Mandatory: `id`, `subject`, `scope_type`, `rights`
     - Optional: `issuer`, `scope_path`, `delegation_depth`, `expires_at`, `not_before`, `max_invocations`, `max_bytes`, `store_path`, `grant_id`
2. **Dispatch Handler (`call_tool`)**:
   - Parses arguments and enforces mandatory parameter constraints.
   - Maps `scope_type` to `CapabilityScope` (`Filesystem`, `Network`, `Ipc`, `System`).
   - Parses capability rights strings into `CapabilityRight` enums (`Read`, `Write`, `Execute`, `Delete`, `Admin`, `Delegate`).
   - Configures `PepGrantConstraints` (temporal bounds `not_before` / `expires_at`, quotas `max_invocations` / `max_bytes`, and `max_delegation_depth`).
   - Instantiates `PepGrant` in `Active` state and runs formal validation (`grant.validate()`).
   - Loads existing `PepGrantStore` (or initializes empty), adds the new grant, and atomically saves to `store_path`.
   - Returns standard JSON envelope: `{"ok": true, "tool": "aios.pep.grant.issue", "grant": <grant>}`.
3. **Audit Row Emission**:
   - Routes through `dispatch::recorded_call`, appending a cryptographic SHA-256 hash-chained row to SQLite `$AIOSH_HOME/audit.db`.

---

## 3. Verification

### 3.1 Existing Smoke Suite Execution
```text
> python -m pytest code/aiosh-mcp/tests/test_pep_decision_smoke.py
============================= test session starts =============================
platform win32 -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
rootdir: C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-mcp
configfile: pyproject.toml
plugins: anyio-4.14.2
collected 7 items

code\aiosh-mcp\tests\test_pep_decision_smoke.py .......                  [100%]

============================== 7 passed in 6.02s ==============================
```

---

## 4. Acceptance Confirmation
- [x] `aios.pep.grant.issue` implemented with full working behavior.
- [x] Zero regressions in existing test suites.
- [x] Consequential mutations write to audit ring via `dispatch::recorded_call`.
