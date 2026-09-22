# Task Evidence: T-02204 (Grant Lifecycle / data model: Implementation)

## 1. Scope & Execution
Implemented the complete, working behavior for the Grant Lifecycle data model in `code/aiosh-rust/aiosh-core/src/pep_grant.rs`:
- Implemented `PepGrant`:
  - `validate()`: Enforces length caps, control character exclusion, safe identifiers, and valid scopes.
  - `can_transition_to()`, `transition_to()`: Implements finite state machine with terminal states (`Revoked`, `Expired`).
  - `revoke()`: Immutable revocation context (`revoked_at`, `revoked_by`, `reason`).
  - `is_usable_at()`: Evaluates `not_before`, `expires_at`, invocation quotas, and byte quotas against UTC timestamps.
  - `record_invocation()`: Increments invocation and byte counts; triggers automatic state transition to `Expired` when limits are reached.
  - `attenuate()`: Produces child grants with monotonic subset of rights, bounded delegation depth, and shared or narrowed scope.
- Implemented `PepGrantStore`:
  - Enforces `MAX_GRANTS_IN_STORE = 5000` and `MAX_GRANT_STORE_SIZE = 10 MiB`.
  - In-memory grant management and lookup (`add_grant`, `get_grant`, `list_grants_for_subject`).
  - Cascade revocation (`revoke_grant(..., cascade: true)`): Recursively identifies and revokes all descendant grants.
  - Atomic persistence (`save_to_path`, `load_from_path`): Staged temporary file write with atomic rename and cleanup on error.
  - Action validation helper (`validate_grant_for_action`).

## 2. Invariants Preserved
- `PEPGRANT1`: Deterministic finite state machine with irreversible terminal states.
- `PEPGRANT2`: Path hygiene and character set validation on all identifiers.
- `PEPGRANT3`: Monotonic attenuation of rights and delegation depth.
- `PEPGRANT4`: Accurate temporal checks and quota consumption.
- `PEPGRANT5`: Non-destructive revocation with optional cascading.
- `PEPGRANT6`: Canonical JSON serialization and typed error envelopes.

## 3. Acceptance Confirmation
- [x] Full data model implementation complete in `code/aiosh-rust/aiosh-core/src/pep_grant.rs`.
- [x] Zero external dependencies added.
- [x] Clean compilation verified via `cargo check -p aiosh-core` (code 0).
