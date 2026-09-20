# Research: T-02101 (PEP Decision Engine / data model: Research)

## 1. Executive Summary & Objective
This research establishes the foundational data model, decision lattice, and operational invariants for the **PEP Decision Engine** in Phase 2 (Security Kernel & PEP Fabric). The PEP Decision Engine serves as the authoritative Policy Decision Point (PDP) and Policy Enforcement Point (PEP) evaluation fabric for all AIOS tool invocations, shell executions, and inter-process communications.

## 2. Industry Standards & Architectural Precedents
1. **NIST SP 800-162 & XACML 3.0 (Attribute-Based Access Control)**:
   - Evaluates attributes of the **Subject** (role, security clearance, user/agent ID), **Resource** (URI, sensitivity classification, owner), **Action** (read, write, execute, delete, admin), and **Environment** (timestamp, network location, execution mode).
   - Combining algorithms: `DenyOverrides` (default secure), `PermitOverrides`, `FirstApplicable`.
2. **AWS Cedar & Open Policy Agent (OPA)**:
   - High-speed in-memory evaluation with deterministic, non-ambiguous policy outcomes.
   - Strict separation of authorization decision logic from enforcement points.
3. **Google Zanzibar & Relation Tuples**:
   - Explicit namespace and object relation matching with fast graph reachability checks.

## 3. Invariants for PEP Decision Engine (`PEPDEC1..PEPDEC6`)

| Invariant | Name | Formal Rule |
|---|---|---|
| **`PEPDEC1`** | **Complete Mediation & Fail-Closed** | Default deny: any authorization request evaluated by the decision engine that lacks an explicit permit rule or encounters an error must evaluate to `Deny`. No ambient or implicit permissions. |
| **`PEPDEC2`** | **Canonical Request Context** | Every authorization request must be fully contextualized with validated `subject`, `resource`, `action`, and `environment` (timestamp, session token, client address, grant). |
| **`PEPDEC3`** | **Deterministic Combining Logic** | Rule evaluation follows explicit combining algorithms (`DenyOverrides`, `PermitOverrides`, `FirstApplicable`). Results must be deterministic across all execution targets. |
| **`PEPDEC4`** | **Atomic Decision Response** | Evaluation generates a structured `PepDecision` containing `decision` (`Permit`, `Deny`, `Indeterminate`, `NotApplicable`), `reason`, `matched_rule_id`, `obligations`, and evaluation latency. |
| **`PEPDEC5`** | **Pure & Idempotent Evaluation** | Evaluating an authorization request is strictly side-effect-free and does not mutate engine state or environment variables. |
| **`PEPDEC6`** | **Audit Trail Integrity** | Every decision produces a structured audit context ready for direct intake by the AIOS Audit Ring (`aios.audit.*`). |

## 4. Proposed Data Model
- `PepRequest`: Subject, Resource, Action, Environment context.
- `PepDecision`: Decision verdict enum (`Permit`, `Deny`, `Indeterminate`, `NotApplicable`), matched rule, reason, obligations.
- `PepObligation`: Post-decision actions (e.g., `AuditLog`, `RateLimitRecord`, `RedactOutput`).
- `PepCombiningAlgorithm`: `DenyOverrides`, `PermitOverrides`, `FirstApplicable`.
