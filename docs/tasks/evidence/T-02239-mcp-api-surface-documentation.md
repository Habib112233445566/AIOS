# T-02239: Grant Lifecycle - MCP/API Surface: Documentation

## Metadata
- **Task ID:** `T-02239`
- **Subsystem:** Phase 2 — Policy Enforcement Point (PEP) Fabric & Security Kernel
- **Component:** Grant Lifecycle MCP/API Surface Documentation (`docs/pep_decision_engine.md`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic 4: Grant Lifecycle (9/10) — MCP/API Surface Documentation

---

## 1. Documentation Overview

Comprehensive architectural and API documentation for the Grant Lifecycle MCP surface (`aios.pep.grant.*`) was added to `docs/pep_decision_engine.md` under **Section 18: Grant Lifecycle MCP Surface Reference**.

### 1.1 Documented Tools
The documentation covers the complete tool surface across 7 MCP endpoints:
1. `aios.pep.grant.issue`: Root grant issuance, input schemas, required parameters, and response envelopes.
2. `aios.pep.grant.list`: Grant catalog querying with optional subject filtering.
3. `aios.pep.grant.inspect`: Targeted grant lookup and full attribute inspection.
4. `aios.pep.grant.validate`: Evaluation of grant status, temporal validity window, and specific capability rights.
5. `aios.pep.grant.revoke`: Revocation semantics, audit actor attribution, reason logging, and cascade mechanics.
6. `aios.pep.grant.attenuate`: Formal derivation rules, parent precondition verification, monotonic right subset enforcement, and depth decrements.
7. `aios.pep.grant.sweep`: Expired grant detection, state transitions, and persistence updates.

### 1.2 Documented Security Guarantees & Constraints
- **Zero Ambient Authority & Complete Mediation**: All tool invocations require authentication, parameter validation, and classification gate passing.
- **Path Traversal & Resource Caps**: Enforcement of `validate_pep_service_path` (prohibiting `..`, length $> 1024$, non-`.json`) and 16 MiB size cap.
- **Audit Ring Integration (ADR-0035 §D-2)**: Immutable audit record generation for every MCP call via `dispatch::recorded_call`.
- **JSON Envelope Standards**: Strict `ok`, `tool`, payload, and `audit_id` fields.

---

## 2. Verification

Verified that `docs/pep_decision_engine.md` contains Section 18 with accurate JSON schemas, links to evidence files `T-02231` through `T-02240`, and clear usage guidelines.
