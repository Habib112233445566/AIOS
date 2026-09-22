# Task Evidence: T-02235 (Grant Lifecycle / MCP/API surface: Unit Test)

## 1. Metadata
- **Task ID:** `T-02235`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle MCP/API Surface Unit Test (`code/aiosh-mcp/tests/test_pep_grant_mcp.py`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic 4: Grant Lifecycle MCP Surface (4/10) — Unit Test

---

## 2. Test Scope & Invariants Covered

Created dedicated, standalone test suite in `code/aiosh-mcp/tests/test_pep_grant_mcp.py` exercising all 7 Grant Lifecycle tools over MCP JSON-RPC 2.0 stdio:
1. **Tool Registration Invariants (`test_mcp_grant_tool_registration`)**:
   - Discovers `aios.pep.grant.issue`, `aios.pep.grant.attenuate`, `aios.pep.grant.list`, `aios.pep.grant.inspect`, `aios.pep.grant.validate`, `aios.pep.grant.revoke`, and `aios.pep.grant.sweep` in `tools/list` manifest.
2. **Grant Issuance & Validation (`test_mcp_grant_issue_and_validate`)**:
   - Happy path creation of root authorization grants with scope, rights, and delegation constraints.
   - Validation of active grant for permitted actions.
   - Negative rejection on unauthorized right (`ok: false`).
   - Negative rejection on subject mismatch (`ok: false`).
   - Negative issuance on missing required arguments.
   - Negative issuance on unrecognized scope type.
   - Negative issuance on duplicate grant ID (`grant ID already exists`).
3. **Attenuation & Cascade Revocation (`test_mcp_grant_attenuation_and_cascade_revocation`)**:
   - Derivation of attenuated child grant with rights subset containment and depth decrement.
   - Strict rejection on right expansion attempt (`ATTENUATION_FAILED`).
   - Filtered listing by subject and grant inspection.
   - Recursive cascade revocation affecting full parent/child hierarchy.
   - Subsequent validation refusal on revoked child grant.
4. **Expiration Sweeping (`test_mcp_grant_sweep`)**:
   - Transition of past-expired grants to `Expired` state via `aios.pep.grant.sweep`.

---

## 3. Test Output

### 3.1 Standalone Test Runner
```text
> python code/aiosh-mcp/tests/test_pep_grant_mcp.py
=== All MCP Grant Unit Tests Passed Successfully ===
```

### 3.2 Pytest Integration Runner
```text
> python -m pytest code/aiosh-mcp/tests/test_pep_grant_mcp.py code/aiosh-mcp/tests/test_pep_decision_smoke.py
============================= test session starts =============================
platform win32 -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
rootdir: C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-mcp
configfile: pyproject.toml
plugins: anyio-4.14.2
collected 11 items

code\aiosh-mcp\tests\test_pep_grant_mcp.py ....                          [ 36%]
code\aiosh-mcp\tests\test_pep_decision_smoke.py .......                  [100%]

============================= 11 passed in 15.87s =============================
```

---

## 4. Acceptance Confirmation
- [x] Dedicated unit test file `code/aiosh-mcp/tests/test_pep_grant_mcp.py` authored and runs standalone.
- [x] Happy paths, boundaries, and negative cases fully asserted.
- [x] 100% pass rate achieved across MCP test suites.
