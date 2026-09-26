# T-02279 Documentation: Grant Lifecycle Observability

**Task:** Document the observability of Grant Lifecycle for operators and agents.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Observability  

---

## 1. Overview

The Grant Lifecycle Observability Subsystem (`PepGrantObservabilityReport`) provides point-in-time state aggregation, delegation hierarchy metrics, capacity monitoring, and health status for capability grants across AIOS.

---

## 2. MCP JSON-RPC Invocation

### Tool: `aios.pep.grant.report`

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.grant.report",
    "arguments": {
      "store_path": ".aios/pep_grants.json"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "ok": true,
    "tool": "aios.pep.grant.report",
    "report": {
      "total_grants": 12,
      "requested_grants": 0,
      "active_grants": 10,
      "suspended_grants": 1,
      "revoked_grants": 1,
      "expired_grants": 0,
      "root_grants_count": 3,
      "derived_grants_count": 9,
      "unique_subjects_count": 5,
      "unique_issuers_count": 2,
      "grants_by_state": {
        "active": 10,
        "suspended": 1,
        "revoked": 1
      },
      "grants_by_right": {
        "read": 12,
        "write": 6,
        "delegate": 3
      },
      "grants_by_scope_type": {
        "filesystem": 8,
        "system": 4
      },
      "capacity_limit": 5000,
      "capacity_utilization_percent": 0,
      "is_healthy": true,
      "generated_at": "2026-09-22T14:30:00Z"
    }
  }
}
```

---

## 3. Rust API Example

```rust
use aiosh_core::pep_grant_service::PepGrantService;

let service = PepGrantService::new();
// ... populate grants ...

let report = service.generate_observability_report();
report.validate().expect("Report invariant check failed");

println!("Total Grants: {}", report.total_grants);
println!("Health Status: {}", if report.is_healthy { "HEALTHY" } else { "DEGRADED" });
println!("Capacity: {}%", report.capacity_utilization_percent);
```

---

## 4. Operational Invariants & Thresholds

1. **Composite Health Rule:** `is_healthy` remains `true` if `capacity_utilization_percent < 90%`. At 90% or above, the health flips to `false` (DEGRADED).
2. **State Sum Identity:** `total_grants == requested + active + suspended + revoked + expired`.
3. **Hierarchy Identity:** `total_grants == root_grants_count + derived_grants_count`.
