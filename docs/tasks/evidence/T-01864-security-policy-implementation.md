# Task Evidence: T-01864 - Network Bootstrap / security policy: Implementation

## 1. Overview
- **Task ID**: `T-01864`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Implement `NetworkSecurityPolicy` in `code/aiosh-rust/aiosh-core/src/network_policy.rs` and verify compliance with invariants `NPOL1..NPOL6`.

---

## 2. Implementation Details
1. **Enforcement Modes**:
   - `NetworkPolicyMode`: `Enforcing`, `Audit`, `Permissive`.
2. **Policy Invariants Implemented**:
   - `NPOL1` (Interface Governance): Disallowed interface types (`InterfaceType::Other`, etc.), prohibited interface names, optional interface allowlist, promiscuous flag detection (`PROMISC`), and required MAC address validation on Ethernet.
   - `NPOL2` (Route Governance): Verifies that all routes reference valid, discovered interface names.
   - `NPOL3` (DNS Governance): Verifies that DNS nameservers do not match `disallowed_dns_servers` and are included in `allowed_dns_servers` if specified.
   - `NPOL4` (Capacity Bounds): Validates maximum interface count, route count, and DNS nameserver count. Sorts violations deterministically by `rule_id` then `target`.
   - `NPOL5` (Sanitization & Redaction): Implemented `apply_and_sanitize()` which redacts MAC address prefixes and IPv4 host octets when `redact_sensitive_addresses` is enabled.
   - `NPOL6` (Path Validation & Persistence): Implemented `validate_policy_path()`, `load_from_path()` with 1 MB size cap (`MAX_POLICY_FILE_BYTES`), atomic sibling write and rename in `save_to_path()`, and `from_env()`.

---

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core` passed with zero errors and zero warnings.
