# Task Evidence: T-02209 (Grant Lifecycle / data model: Documentation)

## 1. Scope & Execution
Documented the Grant Lifecycle Data Model in authoritative system reference `docs/pep_decision_engine.md` (Section 15: Grant Lifecycle Subsystem Reference — Data Model):
- Detailed the 6 core architectural invariants (`PEPGRANT1..PEPGRANT6`):
  - `PEPGRANT1`: Finite State Machine (`Requested`, `Active`, `Suspended`, `Revoked`, `Expired`).
  - `PEPGRANT2`: Strict Identification & Hygiene Validation (length caps, format checks, control-char exclusion).
  - `PEPGRANT3`: Delegation & Attenuation Calculus (subsetting of rights, `Delegate` right prerequisite, depth counter decrement).
  - `PEPGRANT4`: Temporal & Volumetric Quota Governance (`not_before`, `expires_at`, invocation/byte counters).
  - `PEPGRANT5`: Authoritative Revocation & Recursive Cascade (idempotent, audit trails, transitive closure cascade).
  - `PEPGRANT6`: Atomic Persistence & Dual-Substrate Interfaces (safe file atomic replace, CLI `aiosh pep grant ...`, MCP `aios.pep.grant.*`, SQLite audit row emission).
- Added copy-pasteable operator examples for CLI (`aiosh pep grant list`, `inspect`, `validate`, `revoke --cascade`) and agent MCP tool calls (`aios.pep.grant.validate`, `aios.pep.grant.revoke`).
- Honestly documented operational constraints and limitations (`MAX_GRANTS_IN_STORE = 5000`, `MAX_GRANT_STORE_SIZE = 10 MiB`, `MAX_DELEGATION_DEPTH_LIMIT = 8`, metadata bounds, terminal state sinks).
- Linked all sub-epic task evidence files (`T-02201` through `T-02210`).

---

## 2. Acceptance Confirmation
- [x] Reference documentation updated in `docs/pep_decision_engine.md` Section 15.
- [x] Copy-pasteable CLI and MCP examples provided.
- [x] System limits and architectural constraints explicitly stated.
- [x] Evidence links established across all Sub-Epic 1 tasks.
