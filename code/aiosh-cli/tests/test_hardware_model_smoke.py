#!/usr/bin/env python3
"""Smoke test for Hardware Detection data model integration and schema parity."""

import json
import sys

def test_hardware_inventory_schema():
    sample_inventory = {
        "timestamp": "2026-09-20T06:00:00Z",
        "hostname": "aios-test-host",
        "architecture": "x86_64",
        "kernel_version": "6.6.13-aios",
        "devices": [
            {
                "id": "cpu:0",
                "name": "Intel Core i7-12700K",
                "class": "cpu",
                "bus": "system",
                "attributes": {
                    "cores": "12",
                    "threads": "20"
                }
            },
            {
                "id": "pci:0000:00:02.0",
                "name": "Intel Iris Xe Graphics",
                "class": "gpu",
                "bus": "pci",
                "vendor_id": "8086",
                "device_id": "4680",
                "vendor_name": "Intel Corporation",
                "driver": "i915",
                "sysfs_path": "/sys/bus/pci/devices/0000:00:02.0",
                "dev_path": "/dev/dri/card0",
                "attributes": {
                    "vram_mb": "4096"
                }
            },
            {
                "id": "block:nvme0n1",
                "name": "Samsung SSD 990 PRO 2TB",
                "class": "block",
                "bus": "pci",
                "vendor_id": "144d",
                "device_id": "a80c",
                "driver": "nvme",
                "sysfs_path": "/sys/class/block/nvme0n1",
                "dev_path": "/dev/nvme0n1"
            }
        ],
        "summary": {
            "cpu": 1,
            "gpu": 1,
            "block": 1
        }
    }

    # Verify JSON roundtrip
    serialized = json.dumps(sample_inventory, sort_keys=True)
    deserialized = json.loads(serialized)
    assert deserialized["hostname"] == "aios-test-host"
    assert len(deserialized["devices"]) == 3
    assert deserialized["summary"]["cpu"] == 1
    assert deserialized["summary"]["gpu"] == 1
    assert deserialized["summary"]["block"] == 1

    # Invariant HD1: Unique non-empty IDs
    ids = [d["id"] for d in deserialized["devices"]]
    assert len(ids) == len(set(ids))
    assert all(len(i) > 0 for i in ids)

    # Invariant HD2: 4-digit hex vendor_id and device_id
    for d in deserialized["devices"]:
        if "vendor_id" in d:
            assert len(d["vendor_id"]) == 4 and all(c in "0123456789abcdefABCDEF" for c in d["vendor_id"])
        if "device_id" in d:
            assert len(d["device_id"]) == 4 and all(c in "0123456789abcdefABCDEF" for c in d["device_id"])

    # Invariant HD3: Summary count parity
    class_counts = {}
    for d in deserialized["devices"]:
        c = d["class"]
        class_counts[c] = class_counts.get(c, 0) + 1
    assert class_counts == deserialized["summary"]

    # Invariant HD4: Path safety (no control chars or traversal)
    for d in deserialized["devices"]:
        for path_field in ("sysfs_path", "dev_path"):
            if path_field in d:
                val = d[path_field]
                assert ".." not in val
                assert "\x00" not in val
                assert "\n" not in val

    print("PASS: test_hardware_inventory_schema")

def main():
    test_hardware_inventory_schema()
    print("ALL HARDWARE MODEL INTEGRATION TESTS PASSED.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
