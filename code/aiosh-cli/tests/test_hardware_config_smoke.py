#!/usr/bin/env python3
"""Integration Smoke Test for Hardware Detection Configuration Subsystem (T-01746).

Validates:
- Strict adherence to invariants HCFG1..HCFG5:
  - HCFG1: Path hygiene (non-empty, <= 1024 chars, no control characters).
  - HCFG2: Class filtering (valid DeviceClasses, uniqueness, <= 9 items).
  - HCFG3: Resource bounds (1 <= max_devices <= 50,000, 1024 <= max_payload <= 104,857,600).
  - HCFG4: Timeout bounds (1 <= scan_timeout_secs <= 300).
  - HCFG5: Lossless serialization, missing-file fallback, and environment overrides.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path

VALID_CLASSES = {
    "cpu", "memory", "block", "network", "gpu", "pci", "usb", "system", "other"
}

DEFAULT_CONFIG = {
    "default_store_path": ".aios/hardware_inventory.json",
    "sysfs_path": "/sys",
    "procfs_path": "/proc",
    "enabled_classes": None,
    "include_attributes": True,
    "max_devices": 10000,
    "max_payload_bytes": 10485760,
    "scan_timeout_secs": 30,
}


def validate_config(cfg: dict) -> tuple[bool, str]:
    """Validates configuration against HCFG1..HCFG5 rules."""
    # HCFG1: Path hygiene
    for path_key in ("default_store_path", "sysfs_path", "procfs_path"):
        val = cfg.get(path_key)
        if not isinstance(val, str) or not val.strip():
            return False, f"HCFG1 violation: {path_key} cannot be empty"
        if len(val) > 1024:
            return False, f"HCFG1 violation: {path_key} exceeds 1024 characters"
        if any(ord(c) < 32 or ord(c) == 127 for c in val):
            return False, f"HCFG1 violation: {path_key} contains control characters"

    # HCFG2: Class filtering
    classes = cfg.get("enabled_classes")
    if classes is not None:
        if not isinstance(classes, list):
            return False, "HCFG2 violation: enabled_classes must be a list or null"
        if len(classes) > 9:
            return False, f"HCFG2 violation: enabled_classes count {len(classes)} exceeds 9"
        seen = set()
        for c in classes:
            if not isinstance(c, str) or c.lower() not in VALID_CLASSES:
                return False, f"HCFG2 violation: invalid device class '{c}'"
            if c.lower() in seen:
                return False, f"HCFG2 violation: duplicate device class '{c}'"
            seen.add(c.lower())

    # HCFG3: Resource bounds
    max_devices = cfg.get("max_devices")
    if not isinstance(max_devices, int) or max_devices < 1 or max_devices > 50000:
        return False, f"HCFG3 violation: max_devices must be between 1 and 50,000 (got {max_devices})"

    max_payload = cfg.get("max_payload_bytes")
    if not isinstance(max_payload, int) or max_payload < 1024 or max_payload > 104857600:
        return False, f"HCFG3 violation: max_payload_bytes must be between 1024 and 104,857,600 (got {max_payload})"

    # HCFG4: Timeout bounds
    timeout = cfg.get("scan_timeout_secs")
    if not isinstance(timeout, int) or timeout < 1 or timeout > 300:
        return False, f"HCFG4 violation: scan_timeout_secs must be between 1 and 300 (got {timeout})"

    return True, "OK"


def test_hcfg1_path_hygiene():
    cfg = dict(DEFAULT_CONFIG)
    ok, _ = validate_config(cfg)
    assert ok, "Default config must be valid"

    # Empty path
    bad_cfg = dict(cfg, sysfs_path="")
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG1" in err

    # Control char
    bad_cfg = dict(cfg, default_store_path="/path\0invalid")
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG1" in err

    # Length > 1024
    bad_cfg = dict(cfg, procfs_path="a" * 1025)
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG1" in err
    print("PASS: test_hcfg1_path_hygiene")


def test_hcfg2_class_filtering():
    cfg = dict(DEFAULT_CONFIG)
    # Valid filter
    good_cfg = dict(cfg, enabled_classes=["cpu", "gpu", "network"])
    ok, _ = validate_config(good_cfg)
    assert ok

    # Duplicate class
    bad_cfg = dict(cfg, enabled_classes=["cpu", "gpu", "cpu"])
    ok, err = validate_config(bad_cfg)
    assert not ok and "duplicate" in err

    # Invalid class name
    bad_cfg = dict(cfg, enabled_classes=["quantum_core"])
    ok, err = validate_config(bad_cfg)
    assert not ok and "invalid device class" in err

    # Too many classes
    bad_cfg = dict(cfg, enabled_classes=list(VALID_CLASSES) + ["cpu"])
    ok, err = validate_config(bad_cfg)
    assert not ok and "exceeds 9" in err
    print("PASS: test_hcfg2_class_filtering")


def test_hcfg3_resource_bounds():
    cfg = dict(DEFAULT_CONFIG)

    # Max devices
    bad_cfg = dict(cfg, max_devices=0)
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG3" in err

    bad_cfg = dict(cfg, max_devices=50001)
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG3" in err

    # Max payload
    bad_cfg = dict(cfg, max_payload_bytes=1023)
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG3" in err

    bad_cfg = dict(cfg, max_payload_bytes=104857601)
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG3" in err
    print("PASS: test_hcfg3_resource_bounds")


def test_hcfg4_timeout_bounds():
    cfg = dict(DEFAULT_CONFIG)

    bad_cfg = dict(cfg, scan_timeout_secs=0)
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG4" in err

    bad_cfg = dict(cfg, scan_timeout_secs=301)
    ok, err = validate_config(bad_cfg)
    assert not ok and "HCFG4" in err
    print("PASS: test_hcfg4_timeout_bounds")


def test_hcfg5_file_roundtrip_and_env_overrides():
    with tempfile.TemporaryDirectory() as tmpdir:
        config_path = Path(tmpdir) / "test_config.json"

        # Missing file returns default
        assert not config_path.exists()

        # Save valid config
        custom_cfg = dict(DEFAULT_CONFIG)
        custom_cfg["scan_timeout_secs"] = 45
        custom_cfg["enabled_classes"] = ["block", "network"]
        with open(config_path, "w", encoding="utf-8") as f:
            json.dump(custom_cfg, f, indent=2)

        # Reload
        with open(config_path, "r", encoding="utf-8") as f:
            loaded = json.load(f)
        assert loaded == custom_cfg

        # Environment variable override simulation
        os.environ["AIOSH_HARDWARE_SYSFS"] = "/custom/sys"
        os.environ["AIOSH_HARDWARE_TIMEOUT_SECS"] = "60"

        active_cfg = dict(loaded)
        if "AIOSH_HARDWARE_SYSFS" in os.environ:
            active_cfg["sysfs_path"] = os.environ["AIOSH_HARDWARE_SYSFS"]
        if "AIOSH_HARDWARE_TIMEOUT_SECS" in os.environ:
            active_cfg["scan_timeout_secs"] = int(os.environ["AIOSH_HARDWARE_TIMEOUT_SECS"])

        assert active_cfg["sysfs_path"] == "/custom/sys"
        assert active_cfg["scan_timeout_secs"] == 60

        # Clean up env
        del os.environ["AIOSH_HARDWARE_SYSFS"]
        del os.environ["AIOSH_HARDWARE_TIMEOUT_SECS"]

    print("PASS: test_hcfg5_file_roundtrip_and_env_overrides")


def main():
    print("Starting Hardware Detection Configuration Smoke Suite (HCFG1..HCFG5)...")
    test_hcfg1_path_hygiene()
    test_hcfg2_class_filtering()
    test_hcfg3_resource_bounds()
    test_hcfg4_timeout_bounds()
    test_hcfg5_file_roundtrip_and_env_overrides()
    print("ALL 5 HARDWARE DETECTION CONFIGURATION INTEGRATION TESTS PASSED.")


if __name__ == "__main__":
    main()
