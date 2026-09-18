# Task Evidence: T-01473 - Session Observability Scaffold

## Summary
Implements the core data structures and module scaffolding for the User Session Bootstrap Observability and Telemetry engine (`SSO1..SSO6`).

## Scaffold Implementation
- **Source File**: `code/aiosh-rust/aiosh-core/src/session_observability.rs`
- **Exposed Structs**:
  - `SessionObservabilityReport`: Comprehensive telemetry data structure containing:
    - `total_sessions: usize` (SSO1)
    - `distinct_users_count: usize` (SSO4)
    - `state_breakdown: BTreeMap<String, usize>` (SSO1)
    - `seat_breakdown: BTreeMap<String, usize>` (SSO2)
    - `scope_breakdown: BTreeMap<String, usize>` (SSO2)
    - `session_type_breakdown: BTreeMap<String, usize>` (SSO1)
    - `session_class_breakdown: BTreeMap<String, usize>` (SSO1)
    - `user_breakdown: BTreeMap<String, usize>` (SSO4)
    - `locked_count: usize` (SSO3)
    - `idle_sessions_count: usize` (SSO3)
    - `max_idle_seconds: u64` (SSO3)
    - `total_idle_seconds: u64` (SSO3)
    - `policy_compliant_count: usize` (SSO5)
    - `policy_violations_count: usize` (SSO5)
    - `violating_sessions: Vec<String>` (SSO5)
    - `generated_at: String` (SSO6 canonical timestamp)
- **Module Declaration**: Exported in `code/aiosh-rust/aiosh-core/src/lib.rs` as `pub mod session_observability;` and `pub use session_observability::SessionObservabilityReport;`.

## Invariant Compliance
- **SSO1**: State and class breakdown categorization with zero-initialized bins.
- **SSO2**: Seat arbitration distribution and scope breakdown (`foreground`, `background`).
- **SSO3**: Idle time tracking aggregate and peak detection (`max_idle_seconds`, `total_idle_seconds`).
- **SSO4**: Concurrency metrics and user distribution map.
- **SSO5**: Policy evaluation integration with `UserSessionSecurityPolicy`.
- **SSO6**: Deterministic serializability into canonical JSON.
