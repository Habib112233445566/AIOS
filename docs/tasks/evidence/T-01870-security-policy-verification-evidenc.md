# Task Evidence: T-01870 - Network Bootstrap / security policy: Verification & Evidence

## 1. Overview
- **Task ID**: `T-01870`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy) — **Sub-Epic Closure**
- **Goal**: Full test suite execution and formal closure of Sub-Epic 7.

---

## 2. Verification Summary

### Rust Unit & Hardened Tests (`aiosh-core`)
Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_policy`
```text
running 14 tests
test test_npol1_missing_mac_on_ethernet ... ok
test test_modes_enforcing_audit_permissive ... ok
test test_npol1_prohibited_interface_name ... ok
test test_npol1_disallowed_interface_type ... ok
test test_npol1_whitelist_interface_name ... ok
test test_npol1_promiscuous_mode_violation ... ok
test test_npol2_orphan_route_rejected ... ok
test test_npol3_disallowed_dns_server ... ok
test test_npol3_whitelist_dns_server ... ok
test test_npol5_apply_and_sanitize_redaction ... ok
test test_npol4_capacity_limits ... ok
test test_policy_default_valid ... ok
test test_npol6_oversized_policy_rejected ... ok
test test_npol6_policy_path_hygiene_and_persistence ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### Python Cross-Surface Integration Smoke Suite
Command: `python code/aiosh-cli/tests/test_network_policy_smoke.py`
```text
Running Network Bootstrap Security Policy Smoke Tests (T-01866)...
PASS: test_npol1_interface_governance
PASS: test_npol2_route_governance
PASS: test_npol3_dns_governance
PASS: test_npol4_capacity_and_modes
PASS: test_npol5_sanitization_and_persistence
ALL NETWORK SECURITY POLICY SMOKE TESTS PASSED.
```

### Full Regression Suite Verification
- `test_network_e2e_smoke.py`: 5/5 PASSED.
- `test_network_config_smoke.py`: 6/6 PASSED.

---

## 3. Sub-Epic 7 Formal Sign-Off
Sub-Epic 7 ("Network Bootstrap Security Policy", `T-01861` through `T-01870`) is formally verified, fully documented, and closed. All safety invariants `NPOL1..NPOL6` are satisfied with zero known vulnerabilities.
