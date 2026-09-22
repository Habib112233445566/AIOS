# Security Audit: Batch T-02206 through T-02215
**Subsystem**: Phase 2 — Security Kernel & PEP Fabric / Grant Lifecycle  
**Scope**: Sub-Epic 1 (Data Model Closure: T-02206..T-02210) & Sub-Epic 2 (Core Service Launch & Implementation: T-02211..T-02215)  
**Date**: 2026-09-22  
**Auditor**: Antigravity Autonomous Security Agent

---

## 1. Executive Summary
A comprehensive security review and architectural audit was conducted over the 10 completed tasks spanning the Grant Lifecycle subsystem:
1. **Sub-Epic 1 Closure (T-02206..T-02210)**:
   - `T-02206`: CLI and MCP integration of Grant Lifecycle data model.
   - `T-02207`: Security review and abuse threat modeling.
   - `T-02208`: Hardening against resource exhaustion, memory inflation, and path vulnerabilities.
   - `T-02209`: Documentation update in `docs/pep_decision_engine.md` (Section 15).
   - `T-02210`: Sub-Epic 1 milestone closure verification and evidence sign-off.
2. **Sub-Epic 2 Core Service (T-02211..T-02215)**:
   - `T-02211`: Core service research into capability delegation, token lifecycle, and prior art.
   - `T-02212`: Formal specification of `PepGrantService` and invariants `GSVC1..GSVC6`.
   - `T-02213`: Scaffolding of `code/aiosh-rust/aiosh-core/src/pep_grant_service.rs` and registration in `lib.rs`.
   - `T-02214`: Complete implementation of `PepGrantService` with multi-indexing, FSM gating, quota metering, cascade revocation, and persistence.
   - `T-02215`: Comprehensive automated unit test suite (11/11 tests passing, including rigorous negative boundaries).

---

## 2. Threat Vector Evaluation & Defensive Controls

### TV-01: Unauthorized Privilege Escalation via Attenuation
- **Threat Vector**: A malicious caller or delegated sub-agent holding a restricted grant attempts to generate a child grant with elevated rights (e.g. escalating from `Read` to `Admin`).
- **Defensive Control**:
  - `attenuate_grant()` verifies that the parent grant possesses `CapabilityRight::Delegate`.
  - Asserts that every right in `delegated_rights` is present in the parent grant.
  - Enforces `max_delegation_depth` bounding; decrements depth at each derivation level and blocks attenuation if depth is 0.
- **Audit Verdict**: **DEFENDED & VERIFIED**.

### TV-02: Zombie Token Resurrection (Terminal State Evasion)
- **Threat Vector**: An attacker attempts to reactivate a revoked or expired grant via FSM state transition manipulation.
- **Defensive Control**:
  - Invariant `PEPGRANT1` and `GSVC2` define `Revoked` and `Expired` as terminal sink states.
  - Calling `transition_grant()` or `transition_to()` on a grant in `Revoked` or `Expired` state immediately rejects the operation with `GSVC_ERR_INVALID_TRANSITION`.
- **Audit Verdict**: **DEFENDED & VERIFIED**.

### TV-03: Orphan Child Grant Execution after Parent Revocation
- **Threat Vector**: When a parent grant is revoked, delegated child tokens remain active, allowing sub-agents to retain unauthorized access.
- **Defensive Control**:
  - Invariant `PEPGRANT5` and `GSVC5` enforce recursive cascade revocation via BFS traversal of the `by_parent` DAG.
  - Revoking the root grant transitions all descendant child and grandchild grants to `Revoked` atomically.
  - Each revoked token records the operator ID, timestamp, and audit reason.
- **Audit Verdict**: **DEFENDED & VERIFIED**.

### TV-04: Volumetric & Temporal Quota Bypasses
- **Threat Vector**: An attacker executes actions before a grant becomes valid (`not_before`), after it expires (`expires_at`), or continues calling after invocation/byte quotas are exhausted.
- **Defensive Control**:
  - `evaluate_grant()` evaluates `is_usable_at()` enforcing `not_before <= now < expires_at`.
  - `record_grant_usage()` tracks invocation counts and byte consumption; automatically transitions the grant to `Expired` once quotas are reached.
  - `sweep_expired()` provides active background/cron sweeping of past-deadline grants.
- **Audit Verdict**: **DEFENDED & VERIFIED**.

### TV-05: Storage Tampering & Path Injection
- **Threat Vector**: An adversary passes arbitrary file paths (containing `..` or control characters) to corrupt host files or loads directory objects causing server panics.
- **Defensive Control**:
  - `validate_grant_service_path()` enforces path length caps ($\le 1024$), checks for control characters, rejects `..` traversal, and requires `.json` extension.
  - File loading validates file existence, rejects directory paths (`meta.is_dir()`), and caps size at `MAX_GRANT_SERVICE_STORE_SIZE = 10 MiB`.
  - File writing uses collision-resistant temporary file naming (`.tmp.<pid>.<nonce>`) and atomic replace rename.
- **Audit Verdict**: **DEFENDED & VERIFIED**.

### TV-06: Non-Repudiation & Audit Emission
- **Threat Vector**: Grant lifecycle mutations execute silently without generating audit trails.
- **Defensive Control**:
  - All state-altering MCP tool calls (`aios.pep.grant.*`) execute through `dispatch::recorded_call`, writing immutable audit rows to the SQLite ring (`audit.db`).
  - All CLI subcommands return structured JSON envelopes and emit audit rows.
- **Audit Verdict**: **DEFENDED & VERIFIED**.

---

## 3. Test & Verification Matrix
| Test Suite | File | Tests | Result |
|---|---|---|---|
| Data Model Tests | `code/aiosh-rust/aiosh-core/tests/test_pep_grant.rs` | 10 | **PASS (10/10)** |
| Core Service Tests | `code/aiosh-rust/aiosh-core/tests/test_pep_grant_service.rs` | 11 | **PASS (11/11)** |
| PEP CLI Smoke Suite | `code/aiosh-cli/tests/test_pep_cli_smoke.py` | 9 phases | **PASS (100%)** |
| PEP MCP Smoke Suite | `code/aiosh-mcp/tests/test_pep_decision_smoke.py` | 7 flows | **PASS (100%)** |

Compiler Quality: Zero warnings (`#[warn(unused)]` clean) across all production and test targets.

---

## 4. Conclusion & Certification
All 10 tasks (T-02206..T-02215) have been thoroughly verified, audited, and tested. Zero known security vulnerabilities or policy bypasses remain open in the audited batch.
