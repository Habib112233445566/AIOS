# Task Evidence: T-02219 (Grant Lifecycle / core service: Documentation)

## 1. Scope & Execution
Documented the Grant Lifecycle Core Service subsystem (`pep_grant_service.rs`) in authoritative system documentation:
1. **System Documentation Reference**:
   - Added Section 16 to `docs/pep_decision_engine.md`: *Grant Lifecycle Subsystem Reference — Core Service (Sub-Epic 2)*.
   - Formalized Core Service Invariants:
     - `GSVC1`: Multi-Indexed State Coordination (`grants`, `by_subject`, `by_parent`, `by_state` synchrony).
     - `GSVC2`: Dynamic Issuance & Lifecycle FSM Management.
     - `GSVC3`: Delegation Attenuation & Containment Calculus.
     - `GSVC4`: Active Grant Authorization Evaluation & Usage Metering.
     - `GSVC5`: Transitive Cascade Revocation Traversal.
     - `GSVC6`: Periodic Temporal/Quota Sweep & Crash-Resilient Storage.
2. **Invocation & Interface Documentation**:
   - Operator CLI invocation (`aiosh pep grant sweep`).
   - Agent MCP tool schemas and call examples (`aios.pep.grant.attenuate`, `aios.pep.grant.sweep`).
   - Rust Core Service API code examples (`load_from_path`, `evaluate_grant`, `record_grant_usage`, `attenuate_grant`, `sweep_expired`, `save_to_path`).
3. **Constraints, Hardening, and Evidence References**:
   - Documented memory limits (`MAX_GRANTS_IN_SERVICE = 5000`), file size bounds (`10 MiB`), path hygiene, child ID uniqueness, and fail-closed sweep expiration.
   - Cross-linked task evidence files for T-02211 through T-02220.

---

## 2. Acceptance Confirmation
- [x] Section 16 added to `docs/pep_decision_engine.md` detailing invariants `GSVC1..GSVC6`.
- [x] CLI, MCP, and Rust API usage examples clearly illustrated.
- [x] Security constraints and hardening limits formally documented.
