# T-02276 Integration: Grant Lifecycle Observability

**Task:** Integrate the observability of Grant Lifecycle with the surrounding system.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Observability  

---

## 1. What Was Integrated

1. **MCP Server Integration (`aiosh-mcp`):**
   - Registered `aios.pep.grant.report` in the `tools/list` schema with optional `store_path` parameter.
   - Added dispatch handler for `"aios.pep.grant.report"`:
     - Opens the grant service via `Server::validate_and_open_grant_service()`.
     - Calls `generate_observability_report()`.
     - Validates report invariants via `report.validate()?`.
     - Returns serialized telemetry report in JSON-RPC standard result envelope.

2. **Core Library Re-Exports (`aiosh-core::lib.rs`):**
   - Re-exported `PepGrantObservabilityReport`, `PEP_GRANT_HEALTH_UTILIZATION_THRESHOLD`, `PEPOBS_GRANT_ERR_VALIDATION`, `sanitize_grant_telemetry_text`.

3. **Multi-Substrate Synchronization:**
   - Both `PepGrantService` and `PepGrantStore` expose `.generate_observability_report()`.
   - Verified clean compilation across `aiosh-core` and `aiosh-mcp`.

---

## 2. Acceptance Verification
- ✅ Feature reachable through production MCP interface `aios.pep.grant.report`.
- ✅ Workspace compiles cleanly with zero warnings or errors.
