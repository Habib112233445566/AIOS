# T-01463: User Session Bootstrap — Security Policy: Scaffold

## Metadata
- **Task ID:** `T-01463`
- **Subsystem:** `code/aiosh-rust/aiosh-core::session_policy`
- **Component:** User Session Bootstrap Security Policy Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Scaffold Deliverables

1. Created `code/aiosh-rust/aiosh-core/src/session_policy.rs`:
   - Defined `SessionPolicyMode` (`Enforcing`, `Audit`, `Permissive`).
   - Defined `SessionPolicyViolation` and `SessionPolicyVerdict`.
   - Defined `UserSessionSecurityPolicy` with criteria fields:
     - `disallow_root`, `allowed_root_users`, `allowed_greeter_users`.
     - `allowed_session_types`, `allow_remote_seat0`.
     - `disallowed_env_vars`, `max_env_vars`.
     - `max_sessions_per_user`, `max_total_sessions`.
     - `require_agent_sandboxed`.
   - Defined method signatures:
     - `validate(&self) -> Result<(), String>`
     - `evaluate_spec(&self, spec: &UserSessionSpec) -> SessionPolicyVerdict`
     - `evaluate_store(&self, store: &UserSessionStore) -> Vec<SessionPolicyVerdict>`
     - `from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>`

2. Exported in `code/aiosh-rust/aiosh-core/src/lib.rs`:
   - `pub mod session_policy;`
   - `pub use session_policy::{SessionPolicyMode, SessionPolicyVerdict, SessionPolicyViolation, UserSessionSecurityPolicy};`

## 2. Verification
- `cargo check -p aiosh-core -p aiosh-cli -p aiosh-mcp` completed with exit code 0 and zero warnings.
