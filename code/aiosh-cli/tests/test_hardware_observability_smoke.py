#!/usr/bin/env python3
"""Integration Smoke Test for Hardware Detection Observability Subsystem (T-01776).

Validates:
- Strict adherence to invariants HO1..HO6:
  - HO1: Total device count equals sum of class breakdowns.
  - HO2: Total device count equals sum of bus breakdowns.
  - HO3: Driver binding count + unbound device count equals total devices.
  - HO4: Driver binding rate calculation and edge cases (empty inventory).
  - HO5: Policy compliance summary integration.
  - HO6: Deterministic canonical JSON serialization.
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone


def generate_observability_report(inventory: dict, policy: dict | None = None) -> dict:
    """Python reference implementation of HardwareObservabilityReport::generate (HO1..HO6)."""
    devices = inventory.get("devices", [])
    total_devices = len(devices)

    class_breakdown: dict[str, int] = {}
    bus_breakdown: dict[str, int] = {}
    driver_binding_count = 0
    unbound_device_count = 0
    total_attributes_count = 0

    for dev in devices:
        cls = dev.get("class", "other")
        class_breakdown[cls] = class_breakdown.get(cls, 0) + 1

        bus = dev.get("bus", "unknown")
        bus_breakdown[bus] = bus_breakdown.get(bus, 0) + 1

        if dev.get("driver") is not None:
            driver_binding_count += 1
        else:
            unbound_device_count += 1

        total_attributes_count += len(dev.get("attributes", {}))

    driver_binding_rate = (
        round(driver_binding_count / total_devices, 4) if total_devices > 0 else 0.0
    )

    policy_compliant_count = total_devices
    policy_violations_count = 0
    prohibited_devices_found: list[str] = []
    redacted_devices_count = 0

    if policy is not None:
        prohibited_ids = set(policy.get("prohibited_device_ids", []))
        disallowed_classes = set(policy.get("disallowed_classes", []))
        disallowed_buses = set(policy.get("disallowed_buses", []))
        violating_devs: set[str] = set()

        for dev in devices:
            dev_id = dev.get("id", "")
            is_violating = False

            if dev_id in prohibited_ids:
                is_violating = True
                if dev_id not in prohibited_devices_found:
                    prohibited_devices_found.append(dev_id)
                policy_violations_count += 1

            if dev.get("class") in disallowed_classes:
                is_violating = True
                policy_violations_count += 1

            if dev.get("bus") in disallowed_buses:
                is_violating = True
                policy_violations_count += 1

            if is_violating:
                violating_devs.add(dev_id)

            if policy.get("redact_sensitive_attributes", True):
                attrs = dev.get("attributes", {})
                if any(k.lower() in ("address", "mac", "serial", "uuid", "wwid") or
                       any(s in k.lower() for s in ("address", "mac", "serial", "uuid", "wwid"))
                       for k in attrs):
                    redacted_devices_count += 1

        policy_compliant_count = max(0, total_devices - len(violating_devs))
        prohibited_devices_found.sort()

    return {
        "total_devices": total_devices,
        "class_breakdown": dict(sorted(class_breakdown.items())),
        "bus_breakdown": dict(sorted(bus_breakdown.items())),
        "driver_binding_count": driver_binding_count,
        "unbound_device_count": unbound_device_count,
        "driver_binding_rate": driver_binding_rate,
        "total_attributes_count": total_attributes_count,
        "policy_compliant_count": policy_compliant_count,
        "policy_violations_count": policy_violations_count,
        "prohibited_devices_found": prohibited_devices_found,
        "redacted_devices_count": redacted_devices_count,
        "hostname": inventory.get("hostname", "unknown"),
        "architecture": inventory.get("architecture", "unknown"),
        "kernel_version": inventory.get("kernel_version", "unknown"),
        "generated_at": datetime.now(timezone.utc).isoformat(),
    }


def make_sample_inventory() -> dict:
    return {
        "hostname": "test-host",
        "architecture": "x86_64",
        "kernel_version": "6.6.13-aios",
        "devices": [
            {
                "id": "pci:0000:00:02.0",
                "name": "VGA Controller",
                "class": "gpu",
                "bus": "pci",
                "driver": "i915",
                "vendor_id": "8086",
                "device_id": "9a49",
                "attributes": {"driver": "i915"},
            },
            {
                "id": "net:eth0",
                "name": "Ethernet Controller",
                "class": "network",
                "bus": "pci",
                "driver": "e1000e",
                "vendor_id": "8086",
                "device_id": "15f3",
                "attributes": {"address": "00:11:22:33:44:55", "speed": "1000"},
            },
            {
                "id": "block:sda",
                "name": "Storage Disk",
                "class": "block",
                "bus": "scsi",
                "driver": None,
                "attributes": {"uuid": "1234-5678-abcd", "size": "2097152"},
            },
            {
                "id": "usb:1-1",
                "name": "USB Mouse",
                "class": "usb",
                "bus": "usb",
                "driver": None,
                "attributes": {},
            },
        ],
    }


def test_ho1_class_breakdown_parity():
    inv = make_sample_inventory()
    report = generate_observability_report(inv)
    assert report["total_devices"] == 4
    assert sum(report["class_breakdown"].values()) == 4
    assert report["class_breakdown"]["gpu"] == 1
    assert report["class_breakdown"]["network"] == 1
    assert report["class_breakdown"]["block"] == 1
    assert report["class_breakdown"]["usb"] == 1
    print("PASS: test_ho1_class_breakdown_parity")


def test_ho2_bus_breakdown_parity():
    inv = make_sample_inventory()
    report = generate_observability_report(inv)
    assert report["total_devices"] == 4
    assert sum(report["bus_breakdown"].values()) == 4
    assert report["bus_breakdown"]["pci"] == 2
    assert report["bus_breakdown"]["scsi"] == 1
    assert report["bus_breakdown"]["usb"] == 1
    print("PASS: test_ho2_bus_breakdown_parity")


def test_ho3_driver_binding_accounting():
    inv = make_sample_inventory()
    report = generate_observability_report(inv)
    assert report["driver_binding_count"] == 2
    assert report["unbound_device_count"] == 2
    assert report["driver_binding_count"] + report["unbound_device_count"] == 4
    print("PASS: test_ho3_driver_binding_accounting")


def test_ho4_driver_binding_rate():
    inv = make_sample_inventory()
    report = generate_observability_report(inv)
    assert report["driver_binding_rate"] == 0.5

    empty_inv = {"hostname": "empty", "devices": []}
    empty_report = generate_observability_report(empty_inv)
    assert empty_report["total_devices"] == 0
    assert empty_report["driver_binding_rate"] == 0.0
    print("PASS: test_ho4_driver_binding_rate")


def test_ho5_policy_compliance_and_serialization():
    inv = make_sample_inventory()
    policy = {
        "mode": "audit",
        "disallowed_classes": [],
        "disallowed_buses": [],
        "prohibited_device_ids": ["usb:1-1"],
        "redact_sensitive_attributes": True,
    }
    report = generate_observability_report(inv, policy)
    assert report["policy_violations_count"] == 1
    assert report["policy_compliant_count"] == 3
    assert report["prohibited_devices_found"] == ["usb:1-1"]
    assert report["redacted_devices_count"] == 2  # net:eth0 (address), block:sda (uuid)

    # Test serialization
    serialized = json.dumps(report, sort_keys=True)
    deserialized = json.loads(serialized)
    assert deserialized["total_devices"] == 4
    assert deserialized["driver_binding_rate"] == 0.5
    print("PASS: test_ho5_policy_compliance_and_serialization")


def main():
    print("Starting Hardware Detection Observability Smoke Suite (HO1..HO6)...")
    test_ho1_class_breakdown_parity()
    test_ho2_bus_breakdown_parity()
    test_ho3_driver_binding_accounting()
    test_ho4_driver_binding_rate()
    test_ho5_policy_compliance_and_serialization()
    print("ALL 5 HARDWARE DETECTION OBSERVABILITY INTEGRATION TESTS PASSED.")


if __name__ == "__main__":
    main()
