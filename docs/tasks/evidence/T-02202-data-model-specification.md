# Task Evidence: T-02202 (Grant Lifecycle / data model: Specification)

## 1. Scope & Execution
Authored the formal specification for Grant Lifecycle data models in `docs/tasks/evidence/T-02202-spec.md`:
- Defined invariants `PEPGRANT1..PEPGRANT6`.
- Specified Rust data structures: `PepGrantState`, `PepGrantRevocation`, `PepGrantConstraints`, `PepGrant`, and error constants.
- Specified formal finite state machine with terminal-state enforcement.
- Integrated existing `CapabilityScope` and `CapabilityRight` from `aiosh_core::capability`.
- Defined error codes and standard JSON envelope format.

## 2. Invariants Summary
- `PEPGRANT1`: Deterministic finite state machine with monotonic terminal states (`Revoked`, `Expired`).
- `PEPGRANT2`: Input hygiene and character validation on identifiers and scopes.
- `PEPGRANT3`: Monotonic attenuation of rights and delegation depth.
- `PEPGRANT4`: Temporal bounds and quota verification.
- `PEPGRANT5`: Non-destructive revocation with audit metadata.
- `PEPGRANT6`: Canonical JSON serialization and machine-readable error codes.

## 3. Acceptance Confirmation
- [x] Inputs, outputs, state transitions, and error codes defined.
- [x] Reused interfaces (`CapabilityScope`, `CapabilityRight`) clearly distinguished from new types.
- [x] Spec reviewable and complete without reading implementation code.
