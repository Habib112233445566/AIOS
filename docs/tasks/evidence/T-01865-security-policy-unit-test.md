# Task Evidence: T-01865 - Network Bootstrap / security policy: Unit Test

## 1. Overview
- **Task ID**: `T-01865`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Implement and execute comprehensive unit tests for `NetworkSecurityPolicy` in `code/aiosh-rust/aiosh-core/tests/test_network_policy.rs`.

---

## 2. Test Coverage & Cases
1. `test_policy_default_valid`: Asserts default policy passes validation and has safe defaults (`Enforcing`, promiscuous disabled, MAC required on Ethernet).
2. `test_npol1_disallowed_interface_type`: Rejects disallowed types (e.g. `TunTap`).
3. `test_npol1_prohibited_interface_name`: Rejects explicitly prohibited names (e.g. `wlan0`).
4. `test_npol1_whitelist_interface_name`: Enforces interface allowlist.
5. `test_npol1_promiscuous_mode_violation`: Detects `PROMISC` flag and denies.
6. `test_npol1_missing_mac_on_ethernet`: Detects Ethernet missing MAC address.
7. `test_npol2_orphan_route_rejected`: Detects route referencing non-existent interface.
8. `test_npol3_disallowed_dns_server`: Detects server in `disallowed_dns_servers`.
9. `test_npol3_whitelist_dns_server`: Detects server not in `allowed_dns_servers`.
10. `test_npol4_capacity_limits`: Enforces capacity limits on interfaces, routes, DNS.
11. `test_modes_enforcing_audit_permissive`: Verifies mode parity (Enforcing $\to$ `deny`, Audit $\to$ `audit`, Permissive $\to$ `allow`).
12. `test_npol5_apply_and_sanitize_redaction`: Verifies MAC and IP host redaction.
13. `test_npol6_policy_path_hygiene_and_persistence`: Tests path validation, atomic saving, and loading.
14. `test_npol6_oversized_policy_rejected`: Rejects policy files exceeding `MAX_POLICY_FILE_BYTES` (1 MB).

---

## 3. Test Execution Verification
```text
running 14 tests
test test_npol1_missing_mac_on_ethernet ... ok
test test_npol1_disallowed_interface_type ... ok
test test_npol1_prohibited_interface_name ... ok
test test_modes_enforcing_audit_permissive ... ok
test test_npol1_promiscuous_mode_violation ... ok
test test_npol1_whitelist_interface_name ... ok
test test_npol2_orphan_route_rejected ... ok
test test_npol3_disallowed_dns_server ... ok
test test_npol3_whitelist_dns_server ... ok
test test_npol4_capacity_limits ... ok
test test_npol5_apply_and_sanitize_redaction ... ok
test test_npol6_oversized_policy_rejected ... ok
test test_npol6_policy_path_hygiene_and_persistence ... ok
test test_policy_default_valid ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```
Status: Verified and Passed.

