# Task Evidence: T-02161 (PEP Decision Engine Security Policy: Research)

## Overview
- **Task ID**: `T-02161`
- **Task Name**: security policy: Research
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 7: Security Policy Subsystem
- **Timestamp**: 2026-09-21T01:19:40+05:00
- **Status**: COMPLETED

## Research: PEP Decision Engine Security Policy Subsystem

### 1. Authoritative Sources & Prior Art
- **NIST SP 800-162**: *Guide to Attribute Based Access Control (ABAC) Definition and Considerations*. Defines governance rules for Policy Administration Points (PAP) and Policy Decision Points (PDP).
- **OASIS XACML v3.0**: Core specification defining policy governance, obligation criticality, and administrative policy boundaries.
- **SELinux & Kubernetes Admission Governance**: Two-tier enforcement modes (`Enforcing`, `Permissive`/`AuditOnly`, `Disabled`) allowing zero-downtime policy simulation, dry-run evaluation, and safe rule deployment.
- **ADR-0035**: AIOS Security Architecture Decision Record mandating fail-closed defaults, immutable SQLite audit ring recording, and explicit error propagation.

### 2. Facts vs. Assumptions

#### Established Facts
1. `aiosh_core::pep_decision` implements request evaluation against rules using combining algorithms (`DenyOverrides`, `PermitOverrides`, `FirstApplicable`).
2. `aiosh_core::pep_decision_service` manages in-memory rule sets, indexes, atomic JSON persistence, and corrupt file quarantine.
3. `aiosh_core::pep_config` manages configuration options (paths, max rules, default algorithm).
4. No existing component governs:
   - **Enforcement Modes**: Switching between `Enforcing` (enforcing denies), `Permissive` (evaluating and logging denies without blocking execution), and `Disabled`.
   - **Administrative Authoring Constraints**: Preventing unprivileged agents from injecting `Permit` rules that override system-critical boundaries (`sys:*`, `fs:/etc/*`).
   - **Obligation Criticality**: Dictating whether an obligation failure (e.g. rate-limit exhaustion or audit log failure) causes an otherwise permitted decision to fail-closed.
   - **Policy Integrity & Expiration**: Validating policy signatures, versioning, and time-based validity windows (`valid_from`, `valid_until`).

#### Assumptions
1. The security policy subsystem should be implemented in `code/aiosh-rust/aiosh-core/src/pep_security_policy.rs` with clean exports in `lib.rs`.
2. The security policy model should define `PepEnforcementMode` (`Enforcing`, `Permissive`, `Disabled`) and `PepSecurityPolicy`.
3. In `Permissive` mode, the evaluation records `allowed = true`, but sets an audit indicator and maintains the underlying evaluated effect as `Deny` for transparent telemetry.
4. Consequential changes to policy mode or rules must record an immutable audit row in the SQLite audit ring.

### 3. Decisions & Invariants for Sub-Epic 7 (`PEPPOL1..PEPPOL6`)
- `PEPPOL1` (Enforcement Modes): Support `Enforcing` (default, fail-closed), `Permissive` (dry-run with audit telemetry), and `Disabled`.
- `PEPPOL2` (Administrative Boundary Governance): Disallow unprivileged rules permitting sensitive resources (`sys:*`, `sec:*`, `kernel:*`).
- `PEPPOL3` (Obligation Criticality): Support marking obligations as `Strict` (failure invalidates permit) vs `BestEffort`.
- `PEPPOL4` (Temporal Validity): Support optional `valid_from` and `valid_until` UTC timestamps on policy sets.
- `PEPPOL5` (Atomic Persistence & Path Hygiene): Enforce atomic file persistence (`.tmp.<pid>`) and reject symlinks and path traversal.
- `PEPPOL6` (Audit Integration): Ensure policy state changes emit audit events via the SQLite audit ring.
