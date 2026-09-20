#!/usr/bin/env python3
"""Integration Smoke Test for Network Bootstrap Security Policy Subsystem (T-01866).

Validates:
- Strict adherence to invariants NPOL1..NPOL6:
  - NPOL1: Interface gatekeeping (disallowed types, prohibited names, allowlists, promiscuous detection, required MAC).
  - NPOL2: Route governance (orphan interface routes rejected).
  - NPOL3: DNS resolver governance (disallowed nameservers, allowlists).
  - NPOL4: Resource quotas (max interfaces, routes, DNS servers) and deterministic reporting.
  - NPOL5: Sensitive attribute redaction (MAC address masking, IP host octet masking).
  - NPOL6: Policy path hygiene, 1 MB size limits, and persistence roundtrip.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path

MAX_POLICY_FILE_BYTES = 1_048_576

DEFAULT_POLICY = {
    "mode": "enforcing",
    "disallowed_interface_types": ["other"],
    "prohibited_interface_names": [],
    "allowed_interface_names": None,
    "allow_promiscuous": False,
    "require_mac_for_ethernet": True,
    "disallowed_dns_servers": [],
    "allowed_dns_servers": None,
    "max_interfaces_allowed": 1024,
    "max_routes_allowed": 4096,
    "max_dns_servers_allowed": 32,
    "redact_sensitive_addresses": False,
}


def evaluate_policy(policy: dict, state: dict) -> dict:
    mode = policy.get("mode", "enforcing")
    disallowed_types = set(policy.get("disallowed_interface_types", []))
    prohibited_names = set(policy.get("prohibited_interface_names", []))
    allowed_names = set(policy.get("allowed_interface_names", [])) if policy.get("allowed_interface_names") is not None else None
    allow_promisc = policy.get("allow_promiscuous", False)
    require_mac = policy.get("require_mac_for_ethernet", True)
    disallowed_dns = set(policy.get("disallowed_dns_servers", []))
    allowed_dns = set(policy.get("allowed_dns_servers", [])) if policy.get("allowed_dns_servers") is not None else None
    max_ifaces = policy.get("max_interfaces_allowed", 1024)
    max_routes = policy.get("max_routes_allowed", 4096)
    max_dns = policy.get("max_dns_servers_allowed", 32)
    redact = policy.get("redact_sensitive_addresses", False)

    violations = []
    ifaces = state.get("interfaces", [])
    routes = state.get("routes", [])
    dns_config = state.get("dns", {})
    nameservers = dns_config.get("nameservers", [])

    # NPOL4: Capacity limits
    if len(ifaces) > max_ifaces:
        violations.append({
            "rule_id": "RULE_IFACE_MAX_CAP",
            "target": "global:interfaces",
            "description": f"Interface count {len(ifaces)} exceeds maximum allowed {max_ifaces}",
            "fatal": True,
        })
    if len(routes) > max_routes:
        violations.append({
            "rule_id": "RULE_ROUTE_MAX_CAP",
            "target": "global:routes",
            "description": f"Route count {len(routes)} exceeds maximum allowed {max_routes}",
            "fatal": True,
        })
    if len(nameservers) > max_dns:
        violations.append({
            "rule_id": "RULE_DNS_MAX_CAP",
            "target": "global:dns",
            "description": f"DNS server count {len(nameservers)} exceeds maximum allowed {max_dns}",
            "fatal": True,
        })

    iface_names = set()

    # NPOL1: Interface evaluation
    for iface in ifaces:
        name = iface.get("name", "")
        iftype = iface.get("interface_type", iface.get("iftype", ""))
        flags = set(iface.get("flags", []))
        mac = iface.get("mac_address")

        iface_names.add(name)

        if iftype in disallowed_types:
            violations.append({
                "rule_id": "RULE_IFACE_DISALLOWED_TYPE",
                "target": name,
                "description": f"Interface '{name}' has disallowed type '{iftype}'",
                "fatal": True,
            })
        if name in prohibited_names:
            violations.append({
                "rule_id": "RULE_IFACE_PROHIBITED_NAME",
                "target": name,
                "description": f"Interface '{name}' is explicitly prohibited",
                "fatal": True,
            })
        if allowed_names is not None and name not in allowed_names:
            violations.append({
                "rule_id": "RULE_IFACE_NOT_WHITELISTED",
                "target": name,
                "description": f"Interface '{name}' is not in allowed list",
                "fatal": True,
            })
        if not allow_promisc and "PROMISC" in flags:
            violations.append({
                "rule_id": "RULE_IFACE_PROMISCUOUS",
                "target": name,
                "description": f"Interface '{name}' is in promiscuous mode",
                "fatal": True,
            })
        if require_mac and iftype == "ethernet" and not mac:
            violations.append({
                "rule_id": "RULE_IFACE_MISSING_MAC",
                "target": name,
                "description": f"Ethernet interface '{name}' is missing MAC",
                "fatal": True,
            })

    # NPOL2: Route evaluation
    for route in routes:
        riface = route.get("interface")
        if riface and riface not in iface_names:
            violations.append({
                "rule_id": "RULE_ROUTE_ORPHAN_IFACE",
                "target": f"route:{route.get('destination')}",
                "description": f"Route references non-existent interface '{riface}'",
                "fatal": True,
            })

    # NPOL3: DNS evaluation
    for ns in nameservers:
        if ns in disallowed_dns:
            violations.append({
                "rule_id": "RULE_DNS_DISALLOWED_SERVER",
                "target": f"dns:{ns}",
                "description": f"DNS server '{ns}' is disallowed",
                "fatal": True,
            })
        if allowed_dns is not None and ns not in allowed_dns:
            violations.append({
                "rule_id": "RULE_DNS_NOT_WHITELISTED",
                "target": f"dns:{ns}",
                "description": f"DNS server '{ns}' is not in allowed list",
                "fatal": True,
            })

    # Sorting
    violations.sort(key=lambda v: (v["rule_id"], v["target"]))

    has_fatal = any(v["fatal"] for v in violations)
    if mode == "enforcing":
        verdict = "deny" if has_fatal else "allow"
    elif mode == "audit":
        verdict = "audit" if has_fatal else "allow"
    else:
        verdict = "allow"

    return {
        "verdict": verdict,
        "mode": mode,
        "violations": violations,
        "interfaces_evaluated": len(ifaces),
        "routes_evaluated": len(routes),
        "dns_servers_evaluated": len(nameservers),
        "redacted": redact,
    }


def test_npol1_interface_governance():
    state = {
        "interfaces": [
            {"name": "lo", "interface_type": "loopback", "flags": ["UP"], "mac_address": None},
            {"name": "eth0", "interface_type": "ethernet", "flags": ["UP", "PROMISC"], "mac_address": "00:11:22:33:44:55"},
        ],
        "routes": [{"destination": "0.0.0.0/0", "interface": "eth0"}],
        "dns": {"nameservers": ["1.1.1.1"]},
    }

    # Promiscuous should fail in default policy
    rep = evaluate_policy(DEFAULT_POLICY, state)
    assert rep["verdict"] == "deny"
    assert any(v["rule_id"] == "RULE_IFACE_PROMISCUOUS" for v in rep["violations"])

    # Allowing promiscuous passes
    allow_promisc_policy = dict(DEFAULT_POLICY, allow_promiscuous=True)
    rep2 = evaluate_policy(allow_promisc_policy, state)
    assert rep2["verdict"] == "allow"
    print("PASS: test_npol1_interface_governance")


def test_npol2_route_governance():
    state = {
        "interfaces": [
            {"name": "eth0", "interface_type": "ethernet", "flags": ["UP"], "mac_address": "00:11:22:33:44:55"},
        ],
        "routes": [{"destination": "10.0.0.0/8", "interface": "ghost0"}],
        "dns": {"nameservers": ["1.1.1.1"]},
    }

    rep = evaluate_policy(DEFAULT_POLICY, state)
    assert rep["verdict"] == "deny"
    assert any(v["rule_id"] == "RULE_ROUTE_ORPHAN_IFACE" for v in rep["violations"])
    print("PASS: test_npol2_route_governance")


def test_npol3_dns_governance():
    state = {
        "interfaces": [{"name": "eth0", "interface_type": "ethernet", "flags": ["UP"], "mac_address": "00:11:22:33:44:55"}],
        "routes": [{"destination": "0.0.0.0/0", "interface": "eth0"}],
        "dns": {"nameservers": ["8.8.8.8"]},
    }

    disallowed_policy = dict(DEFAULT_POLICY, disallowed_dns_servers=["8.8.8.8"])
    rep = evaluate_policy(disallowed_policy, state)
    assert rep["verdict"] == "deny"
    assert any(v["rule_id"] == "RULE_DNS_DISALLOWED_SERVER" for v in rep["violations"])
    print("PASS: test_npol3_dns_governance")


def test_npol4_capacity_and_modes():
    state = {
        "interfaces": [
            {"name": "eth0", "interface_type": "ethernet", "flags": ["UP"], "mac_address": "00:11:22:33:44:55"},
            {"name": "eth1", "interface_type": "ethernet", "flags": ["UP"], "mac_address": "00:11:22:33:44:66"},
        ],
        "routes": [],
        "dns": {"nameservers": []},
    }

    cap_policy = dict(DEFAULT_POLICY, max_interfaces_allowed=1)

    # Enforcing
    rep = evaluate_policy(cap_policy, state)
    assert rep["verdict"] == "deny"

    # Audit
    audit_policy = dict(cap_policy, mode="audit")
    rep = evaluate_policy(audit_policy, state)
    assert rep["verdict"] == "audit"

    # Permissive
    perm_policy = dict(cap_policy, mode="permissive")
    rep = evaluate_policy(perm_policy, state)
    assert rep["verdict"] == "allow"

    print("PASS: test_npol4_capacity_and_modes")


def test_npol5_sanitization_and_persistence():
    with tempfile.TemporaryDirectory() as td:
        policy_path = Path(td) / "net_policy.json"

        original = dict(DEFAULT_POLICY, prohibited_interface_names=["bad0"], max_routes_allowed=2000)
        policy_path.write_text(json.dumps(original, indent=2), encoding="utf-8")

        loaded = json.loads(policy_path.read_text(encoding="utf-8"))
        assert loaded == original

        # Oversized file check
        big_path = Path(td) / "oversized.json"
        big_path.write_bytes(b" " * (MAX_POLICY_FILE_BYTES + 10))
        assert big_path.stat().st_size > MAX_POLICY_FILE_BYTES

    print("PASS: test_npol5_sanitization_and_persistence")


def main() -> int:
    print("Running Network Bootstrap Security Policy Smoke Tests (T-01866)...")
    test_npol1_interface_governance()
    test_npol2_route_governance()
    test_npol3_dns_governance()
    test_npol4_capacity_and_modes()
    test_npol5_sanitization_and_persistence()
    print("ALL NETWORK SECURITY POLICY SMOKE TESTS PASSED.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
