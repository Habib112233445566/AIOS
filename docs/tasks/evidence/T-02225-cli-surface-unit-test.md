# Task Evidence: T-02225 (Grant Lifecycle / CLI surface: Unit Test)

## 1. Scope & Execution
Authored and executed a comprehensive automated unit and integration test suite for the `aiosh pep grant` CLI surface in `code/aiosh-cli/tests/test_pep_grant_cli.py`:
1. **Grant Issuance & Parameter Validation**:
   - Tested happy path creation of root authorization grants with filesystem scope, rights (`read,write,delegate`), delegation depth, and expiration constraints.
   - Asserted refusal and exit code 2 on missing required arguments (`--subject`, `--rights`).
   - Asserted refusal on unrecognized capability scope types and unrecognized rights.
2. **Delegation & Attenuation**:
   - Verified valid child derivation under parent with rights subset containment and delegation depth decrement.
   - Tested right expansion attempt (requesting unauthorized rights), verifying rejection with exit code 1.
3. **Multi-Index Listing & Inspection**:
   - Verified unfiltered grant listing, subject filtering, and state filtering.
   - Tested inspection of existing grant vs non-existent grant (code 1) vs missing ID (code 2).
4. **Authorization Validation & Temporal Bounds**:
   - Tested validation of active grants for authorized actions (code 0).
   - Tested rejection on subject mismatch, right mismatch, and post-expiration `--now` timestamps (code 1).
5. **Cascading Revocation**:
   - Verified recursive cascade revocation across multi-level parent-child hierarchy (code 0, affects both parent and child).
   - Asserted subsequent evaluation failures on revoked descendant grants (code 1).
6. **Temporal Expiration Sweep**:
   - Tested batch sweep of expired grants with reference timestamp (code 0, transitions to `Expired`).

---

## 2. Test Execution Output
```
> python code/aiosh-cli/tests/test_pep_grant_cli.py
PASS: test_pep_grant_issue_and_validation
PASS: test_pep_grant_attenuation
PASS: test_pep_grant_list_and_inspect
PASS: test_pep_grant_validate_and_revoke
PASS: test_pep_grant_sweep
=== All PEP Grant CLI unit tests passed successfully ===

> python -m pytest code/aiosh-cli/tests/test_pep_grant_cli.py code/aiosh-cli/tests/test_pep_cli_smoke.py code/aiosh-mcp/tests/test_pep_decision_smoke.py
============================= test session starts =============================
platform win32 -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
collected 21 items

code\aiosh-mcp ..............                                            [ 66%]
code\aiosh-mcp\tests\test_pep_decision_smoke.py .......                  [100%]

============================= 21 passed in 9.57s ==============================
```

---

## 3. Acceptance Confirmation
- [x] Dedicated unit test suite authored in `test_pep_grant_cli.py`.
- [x] 100% pass rate achieved across all happy path, boundary, negative, and edge cases.
- [x] Zero regressions across CLI and MCP smoke tests.
