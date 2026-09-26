# T-02271 Research: Grant Lifecycle Observability

**Task:** Establish facts, constraints, and prior art for the observability of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Observability  

---

## 1. Executive Summary

This research establishes the requirements, architectural constraints, and prior art for the **Grant Lifecycle Observability Subsystem** (`PepGrantObservabilityReport`). 

Grant observability provides runtime visibility into capability distribution, delegation trees, credential expiration velocity, storage utilization, and security health across AIOS kernel and userland agents.

---

## 2. Established Facts vs Assumptions

### 2.1 Established Facts (Source Code & Invariants)
1. **State Machine Facts (OBS-FACT-1):** Grants exist in 4 mutually exclusive states (`active`, `suspended`, `revoked`, `expired`). The grant service maintains index mappings (`by_state`) over these states.
2. **Hierarchy Facts (OBS-FACT-2):** Grants form a forest of trees via `parent_grant_id` with root grants having `parent_grant_id: None` and leaves having `max_delegation_depth: 0` or no children.
3. **Capacity Limit (OBS-FACT-3):** `PepGrantService` enforces `MAX_GRANTS_IN_SERVICE` (5,000 grants). Exceeding this limit triggers `GSVC_ERR_CAPACITY`.
4. **Telemetry Sanitization (OBS-FACT-4):** Observability telemetry must not leak sensitive credential secrets or unescaped control characters.

### 2.2 Assumptions Requiring Design Decisions
1. **Health Threshold (OBS-ASSUME-1):** Analogous to PEP Decision Engine observability (`PEP_HEALTH_UTILIZATION_THRESHOLD = 90%`), grant store health should be defined as utilization < 90% and non-corrupted state.
2. **Sweep Backlog Metric (OBS-ASSUME-2):** The report should track expired grants that have not yet been swept, alerting operators to sweep backlog.
3. **Telemetry Frequency (OBS-ASSUME-3):** Reports should be dynamically generated on-demand without blocking concurrent read-only validation operations.

---

## 3. Prior Art & Authoritative Sources

1. **Google Site Reliability Engineering (SRE) Golden Signals:**
   - Saturation: Capacity utilization percentage (`total_grants / max_capacity * 100`).
   - Errors: Count of revoked or corrupted grants.
   - Traffic: Volume of active delegation chains.
2. **OpenTelemetry Security Attributes Spec:**
   - Categorization of security credentials by state, lifetime, and privilege levels.
   - Elimination of sensitive credentials from telemetry spans.
3. **Linux Kernel Capability & Keyring Audit Subsystems:**
   - Audit reporting on key ring cardinality, expiration, and delegation tracking.

---

## 4. Key Design Decisions for Specification

1. **Struct Design:** Implement `PepGrantObservabilityReport` in `aiosh-core::pep_grant_observability`.
2. **Aggregation Method:** Provide `PepGrantService::generate_observability_report(&self) -> PepGrantObservabilityReport`.
3. **Metrics Reported:**
   - Grant states breakdown: `total_grants`, `active_grants`, `suspended_grants`, `revoked_grants`, `expired_grants`.
   - Tree structure: `root_grants_count`, `derived_grants_count`, `max_depth_observed`.
   - Privilege distribution: `grants_by_right`, `unique_subjects_count`, `unique_issuers_count`.
   - Saturation & Health: `capacity_limit`, `capacity_utilization_percent`, `is_healthy`.

---

## 5. Acceptance Verification
- ✅ Authoritative sources cited (SRE Golden Signals, OpenTelemetry, Linux Keyring).
- ✅ Facts and assumptions separated.
- ✅ Decisions outlined for T-02272 Specification.
- ✅ Zero code modifications in this research task.
