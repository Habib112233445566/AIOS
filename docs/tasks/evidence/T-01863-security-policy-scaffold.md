# Task Evidence: T-01863 - Network Bootstrap / security policy: Scaffold

## 1. Overview
- **Task ID**: `T-01863`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Scaffold `network_policy.rs` in `aiosh-core` and register/re-export types in `lib.rs`.

---

## 2. Scaffold Details
1. **Module**: `code/aiosh-rust/aiosh-core/src/network_policy.rs`
2. **Re-export**: Re-exported `pub mod network_policy;` and `pub use network_policy::{NetworkPolicyMode, NetworkSecurityPolicy, NetworkPolicyViolation, NetworkPolicyReport, validate_policy_path};` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
3. **Core Types**:
   - `NetworkPolicyMode`: `Enforcing`, `Audit`, `Permissive`.
   - `NetworkSecurityPolicy`: struct with mode, disallowed interface types, prohibited names, whitelist, promiscuous toggle, MAC requirements, DNS denylist/allowlist, and resource caps.
   - `NetworkPolicyViolation`: violation descriptor with `rule_id`, `target`, `description`, `fatal`.
   - `NetworkPolicyReport`: summary report with `verdict`, `mode`, `violations`, counts evaluated.
   - Functions: `validate()`, `evaluate()`, `load_from_path()`, `save_to_path()`, `from_env()`, `validate_policy_path()`.

---

## 3. Verification
- Project compilation verified via `cargo check`.
