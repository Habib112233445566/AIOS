# T-02256 Evidence: Automated Tests — Integration

**Task:** Integrate the automated tests of Grant Lifecycle with the surrounding system.  
**Status:** COMPLETE  
**Date:** 2026-09-22  

## What Was Done

1. **Fixed Cross-Surface Integration Smoke Test** (`code/aiosh-mcp/tests/test_pep_grant_automated_smoke.py`):
   - Rewrote broken smoke test to use correct MCP JSON-RPC tool argument names:
     - `id` (not `grant_id`), `parent_id`/`child_id` (not `parent_grant_id`/`child_grant_id`)
     - `scope_type`/`scope_path` (not nested `scope` dict)
     - `grant_id_param` for inspect/validate/revoke
   - Tests cover all 7 MCP grant tools: issue, attenuate, list, inspect, validate, revoke (cascade), sweep

2. **End-to-End Verification:**
   - Root grant issuance via `aios.pep.grant.issue` → PASS
   - Multi-tier attenuation (5→4→3 depth decrement) → PASS
   - Right expansion rejection (monotonicity enforcement) → PASS
   - Listing with subject filter → PASS
   - Inspection by grant ID → PASS
   - Positive + negative validation (subject mismatch, right mismatch) → PASS
   - Cascade revocation (revokes 2 grants in hierarchy) → PASS
   - Expiration sweep → PASS
   - Boundary protection (duplicate ID, non-existent grant) → PASS

3. **Cross-Substrate Parity:** The test validates that the Rust MCP binary (`aiosh-mcp.exe`) correctly dispatches all 7 grant lifecycle tools with JSON-RPC 2.0 protocol, confirming parity between the CLI surface (tested in `test_pep_grant_automated.rs`) and the MCP surface.

## Acceptance Criteria Met
- ✅ Feature reachable through its production surface (MCP JSON-RPC)
- ✅ Integration smoke passes end-to-end (all 7 test vectors green)

## Test Output
```
[1] Testing Root Grant Issuance (aios.pep.grant.issue)...
[2] Testing Multi-tier Attenuation (aios.pep.grant.attenuate)...
[3] Testing Grant Listing and Inspection...
[4] Testing Grant Validation (positive + negative)...
[5] Testing Cascade Revocation (aios.pep.grant.revoke)...
[6] Testing Expiration Sweep (aios.pep.grant.sweep)...
[7] Testing Negative Security & Boundary Protection...
=== ALL AUTOMATED PEP GRANT INTEGRATION TESTS PASSED ===
```
