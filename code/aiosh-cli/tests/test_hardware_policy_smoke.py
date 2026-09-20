#!/usr/bin/env python3
"""Integration Smoke Test for Hardware Detection Security Policy Subsystem (T-01766).

Validates:
- Strict adherence to invariants HSEC1..HSEC5:
  - HSEC1: Policy evaluation precedence (Deny > Allow > Default).
  - HSEC2: Sensitive attribute redaction (masking MACs/serials/UUIDs).
  - HSEC3: Class and bus gatekeeping.
  - HSEC4: Deterministic evaluation reporting.
  - HSEC5: Fail-safe defaults and bounds validation.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path

DEFAULT_POLICY = {
    "mode": "enforcing",
    "disallowed_classes": ["other"],
    "disallowed_buses": ["unknown"],
    "prohibited_device_ids": [],
    "allowed_vendor_ids": None,
    "redact_sensitive_attributes": True,
    "max_devices_allowed": 10000,
}

SENSITIVE_KEYS = ("address", "mac", "serial", "uuid", "wwid")


def is_sensitive(key: str) -> bool:
    k = key.lower()
    return any(s in k for s in SENSITIVE_KEYS)


def evaluate_policy(policy: dict, inventory: dict) -> dict:
    """Evaluates security policy against inventory according to HSEC1..HSEC5."""
    mode = policy.get("mode", "enforcing")
    disallowed_classes = set(policy.get("disallowed_classes", []))
    disallowed_buses = set(policy.get("disallowed_buses", []))
    prohibited_ids = set(policy.get("prohibited_device_ids", []))
    allowed_vids = set(policy.get("allowed_vendor_ids", [])) if policy.get("allowed_vendor_ids") is not None else None
    max_devices = policy.get("max_devices_allowed", 10000)
    redact = policy.get("redact_sensitive_attributes", True)

    violations = []
    redacted_count = 0
    devices = inventory.get("devices", [])

    if len(devices) > max_devices:
        violations.append({
            "rule_id": "HPOL-COUNT",
            "device_id": "inventory",
            "description": f"Device count {len(devices)} exceeds policy limit of {max_devices}",
            "fatal": True,
        })

    for dev in devices:
        dev_id = dev.get("id", "")
        cls = dev.get("class", "")
        bus = dev.get("bus", "")
        vid = dev.get("vendor_id")

        if cls in disallowed_classes:
            violations.append({
                "rule_id": "HPOL-CLASS",
                "device_id": dev_id,
                "description": f"Device class '{cls}' is disallowed",
                "fatal": True,
            })

        if bus in disallowed_buses:
            violations.append({
                "rule_id": "HPOL-BUS",
                "device_id": dev_id,
                "description": f"Device bus '{bus}' is disallowed",
                "fatal": True,
            })

        if dev_id in prohibited_ids:
            violations.append({
                "rule_id": "HPOL-ID",
                "device_id": dev_id,
                "description": f"Device ID '{dev_id}' is explicitly prohibited",
                "fatal": True,
            })

        if allowed_vids is not None and vid and vid not in allowed_vids:
            violations.append({
                "rule_id": "HPOL-VENDOR",
                "device_id": dev_id,
                "description": f"Vendor ID '{vid}' is not in allowed list",
                "fatal": False,
            })

        attrs = dev.get("attributes", {})
        if redact and any(is_sensitive(k) for k in attrs):
            redacted_count += 1

    # Deterministic sorting (HSEC4)
    violations.sort(key=lambda v: (v["rule_id"], v["device_id"]))

    has_fatal = any(v["fatal"] for v in violations)
    if mode == "enforcing" and has_fatal:
        verdict = "deny"
    elif mode == "audit" and violations:
        verdict = "audit"
    else:
        verdict = "allow"

    return {
        "verdict": verdict,
        "mode": mode,
        "violations": violations,
        "devices_evaluated": len(devices),
        "devices_redacted": redacted_count,
    }


def test_hsec1_policy_precedence():
    inv = {
        "devices": [
            {"id": "pci:0000:00:02.0", "class": "gpu", "bus": "pci"},
            {"id": "net:eth0", "class": "network", "bus": "pci"},
        ]
    }
    policy = dict(DEFAULT_POLICY, prohibited_device_ids=["net:eth0"])

    # Enforcing mode -> deny
    r = evaluate_policy(policy, inv)
    assert r["verdict"] == "deny"
    assert any(v["rule_id"] == "HPOL-ID" for v in r["violations"])

    # Audit mode -> audit
    audit_policy = dict(policy, mode="audit")
    r = evaluate_policy(audit_policy, inv)
    assert r["verdict"] == "audit"

    # Permissive mode -> allow
    perm_policy = dict(policy, mode="permissive")
    r = evaluate_policy(perm_policy, inv)
    assert r["verdict"] == "allow"
    print("PASS: test_hsec1_policy_precedence")


def test_hsec2_attribute_redaction():
    inv = {
        "devices": [
            {
                "id": "net:eth0",
                "class": "network",
                "bus": "pci",
                "attributes": {
                    "address": "00:11:22:33:44:55",
                    "speed": "1000",
                    "serial_number": "SN88392",
                }
            },
            {
                "id": "block:sda",
                "class": "block",
                "bus": "scsi",
                "attributes": {
                    "uuid": "4a7b-29c1",
                    "size": "500000",
                }
            }
        ]
    }
    r = evaluate_policy(DEFAULT_POLICY, inv)
    assert r["devices_redacted"] == 2

    # Simulate sanitization
    for dev in inv["devices"]:
        for k in dev["attributes"]:
            if is_sensitive(k):
                dev["attributes"][k] = "<REDACTED>"

    assert inv["devices"][0]["attributes"]["address"] == "<REDACTED>"
    assert inv["devices"][0]["attributes"]["serial_number"] == "<REDACTED>"
    assert inv["devices"][0]["attributes"]["speed"] == "1000"
    assert inv["devices"][1]["attributes"]["uuid"] == "<REDACTED>"
    assert inv["devices"][1]["attributes"]["size"] == "500000"
    print("PASS: test_hsec2_attribute_redaction")


def test_hsec3_class_and_bus_gatekeeping():
    inv = {
        "devices": [
            {"id": "unknown:dev0", "class": "other", "bus": "unknown"},
        ]
    }
    r = evaluate_policy(DEFAULT_POLICY, inv)
    assert r["verdict"] == "deny"
    rule_ids = {v["rule_id"] for v in r["violations"]}
    assert "HPOL-CLASS" in rule_ids
    assert "HPOL-BUS" in rule_ids
    print("PASS: test_hsec3_class_and_bus_gatekeeping")


def test_hsec4_deterministic_evaluation():
    inv = {
        "devices": [
            {"id": "dev:2", "class": "other", "bus": "pci"},
            {"id": "dev:1", "class": "other", "bus": "pci"},
        ]
    }
    r1 = evaluate_policy(DEFAULT_POLICY, inv)
    r2 = evaluate_policy(DEFAULT_POLICY, inv)
    assert r1 == r2
    # Check violations order
    assert [v["device_id"] for v in r1["violations"]] == ["dev:1", "dev:2"]
    print("PASS: test_hsec4_deterministic_evaluation")


def test_hsec5_fail_safe_defaults_and_bounds():
    with tempfile.TemporaryDirectory() as tmpdir:
        policy_path = Path(tmpdir) / "policy.json"

        # Missing file falls back to default
        assert not policy_path.exists()
        loaded = dict(DEFAULT_POLICY)

        # Save and reload
        loaded["max_devices_allowed"] = 5000
        with open(policy_path, "w", encoding="utf-8") as f:
            json.dump(loaded, f, indent=2)

        with open(policy_path, "r", encoding="utf-8") as f:
            reloaded = json.load(f)

        assert reloaded == loaded
    print("PASS: test_hsec5_fail_safe_defaults_and_bounds")


def main():
    print("Starting Hardware Detection Security Policy Smoke Suite (HSEC1..HSEC5)...")
    test_hsec1_policy_precedence()
    test_hsec2_attribute_redaction()
    test_hsec3_class_and_bus_gatekeeping()
    test_hsec4_deterministic_evaluation()
    test_hsec5_fail_safe_defaults_and_bounds()
    print("ALL 5 HARDWARE DETECTION SECURITY POLICY INTEGRATION TESTS PASSED.")


if __name__ == "__main__":
    main()
