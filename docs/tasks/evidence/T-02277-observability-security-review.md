# T-02277 Security Review: Grant Lifecycle Observability

**Task:** Security-review the observability of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Observability  

---

## 1. Threat Modeling & Abuse Scenarios

| ID | Abuse Scenario | Attack Vector | Mitigation in Code | Status |
|---|---|---|---|---|
| **THREAT-1** | Telemetry Log Poisoning | Attacker injects ANSI escape sequences or control characters (`\x1b`, `\x00`) in subject or issuer strings | `sanitize_grant_telemetry_text()` strips all control characters (`!c.is_control()`) | ✅ Mitigated |
| **THREAT-2** | Memory Exhaustion via Long Identifiers | Oversized subject string (e.g. 100 KB) injected into telemetry | `sanitize_grant_telemetry_text()` enforces hard 256-character truncation | ✅ Mitigated |
| **THREAT-3** | Invariant / Telemetry Spoofing | Malformed or inconsistent grant counts reported to monitoring systems | `PepGrantObservabilityReport::validate()` guarantees mathematical sum consistency | ✅ Mitigated |
| **THREAT-4** | Path Traversal on Store Loading | Attacker specifies `store_path: "../../../evil.json"` in tool argument | `Server::validate_and_open_grant_service()` enforces path bounds and rejects `..` | ✅ Mitigated |
| **THREAT-5** | State Manipulation during Inspection | Concurrent report generation mutates active grant states | `generate_observability_report(&self)` is purely read-only (`&self`) | ✅ Mitigated |

---

## 2. Invariant Compliance
- **Read-Only Invariant:** Observability inspection produces zero mutating side-effects.
- **Audit Logging:** Invocations of `aios.pep.grant.report` are recorded via `dispatch::recorded_call` into the audit ring.
- **Zero Policy Bypasses:** All abuse scenarios verified mitigated.
