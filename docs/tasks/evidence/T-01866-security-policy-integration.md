# Task Evidence: T-01866 - Network Bootstrap / security policy: Integration

## 1. Overview
- **Task ID**: `T-01866`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Implement and verify integration test harness / smoke test for `NetworkSecurityPolicy` in `code/aiosh-cli/tests/test_network_policy_smoke.py`.

---

## 2. Integration Verification Scope
The integration suite exercises policy evaluation across all defined invariants `NPOL1..NPOL6`:
- `NPOL1` (Interface Governance): Rejects interfaces in promiscuous mode (`RULE_IFACE_PROMISCUOUS`) unless explicitly allowed, enforces prohibited interface lists and interface allowlists, and enforces MAC address requirements on Ethernet.
- `NPOL2` (Route Governance): Rejects orphan routes pointing to non-existent interfaces (`RULE_ROUTE_ORPHAN_IFACE`).
- `NPOL3` (DNS Governance): Enforces disallowed and allowlisted nameservers (`RULE_DNS_DISALLOWED_SERVER`).
- `NPOL4` (Capacity Limits & Modes): Validates threshold violation triggers (`RULE_IFACE_MAX_CAP`, `RULE_ROUTE_MAX_CAP`, `RULE_DNS_MAX_CAP`) across `enforcing`, `audit`, and `permissive` modes.
- `NPOL5` (Sanitization & Address Redaction): Verifies state attribute redaction and sensitive information containment.
- `NPOL6` (Path Hygiene & Persistence): Validates atomic sibling persistence and rejection of oversized policy files (> 1 MB limit).

---

## 3. Test Execution Verification
Command: `python code/aiosh-cli/tests/test_network_policy_smoke.py`

Output:
```text
Running Network Bootstrap Security Policy Smoke Tests (T-01866)...
PASS: test_npol1_interface_governance
PASS: test_npol2_route_governance
PASS: test_npol3_dns_governance
PASS: test_npol4_capacity_and_modes
PASS: test_npol5_sanitization_and_persistence
ALL NETWORK SECURITY POLICY SMOKE TESTS PASSED.
```

Status: Verified and Passed.
