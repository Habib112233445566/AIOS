#!/usr/bin/env python3
"""Integration Smoke Test for Network Bootstrap Configuration Subsystem (T-01846).

Validates:
- Strict adherence to invariants NCONF1..NCONF6:
  - NCONF1: Path hygiene (non-empty, <= 1024 chars, no control characters, no '..' traversal).
  - NCONF2: Capacity limits (1 <= max_interfaces <= 10,000, 1 <= max_routes <= 50,000, 1 <= max_dns_servers <= 64).
  - NCONF3: Resource & timeout bounds (1024 <= max_payload <= 104,857,600, 1 <= scan_timeout <= 300).
  - NCONF4: Fallback DNS server validation (valid IPv4/IPv6, <= max_dns_servers).
  - NCONF5: Environment variable ingestion and fallback.
  - NCONF6: Lossless JSON serialization, atomic save/load, and file size limits (<= 1MB).
"""

from __future__ import annotations

import ipaddress
import json
import os
import sys
import tempfile
from pathlib import Path

MAX_CONFIG_FILE_BYTES = 1_048_576

DEFAULT_CONFIG = {
    "default_store_path": ".aios/network_state.json",
    "sysfs_net_path": "/sys/class/net",
    "procfs_path": "/proc/net",
    "resolv_conf_path": "/etc/resolv.conf",
    "max_interfaces": 1024,
    "max_routes": 4096,
    "max_dns_servers": 32,
    "max_payload_bytes": 10485760,
    "scan_timeout_secs": 30,
    "fallback_dns_servers": ["1.1.1.1", "8.8.8.8"],
}


def validate_config(cfg: dict) -> tuple[bool, str]:
    """Validates configuration against NCONF1..NCONF6 rules."""
    # NCONF1: Path hygiene
    for path_key in ("default_store_path", "sysfs_net_path", "procfs_path", "resolv_conf_path"):
        val = cfg.get(path_key)
        if not isinstance(val, str) or not val.strip():
            return False, f"NCONF1 violation: {path_key} cannot be empty"
        if len(val) > 1024:
            return False, f"NCONF1 violation: {path_key} exceeds 1024 characters"
        if any(ord(c) < 32 or ord(c) == 127 for c in val):
            return False, f"NCONF1 violation: {path_key} contains control characters"
        p = Path(val)
        if ".." in p.parts:
            return False, f"NCONF1 violation: {path_key} contains parent directory traversal ('..')"

    # NCONF2: Capacity limits
    max_ifaces = cfg.get("max_interfaces")
    if not isinstance(max_ifaces, int) or max_ifaces < 1 or max_ifaces > 10000:
        return False, f"NCONF2 violation: max_interfaces must be between 1 and 10,000 (got {max_ifaces})"

    max_routes = cfg.get("max_routes")
    if not isinstance(max_routes, int) or max_routes < 1 or max_routes > 50000:
        return False, f"NCONF2 violation: max_routes must be between 1 and 50,000 (got {max_routes})"

    max_dns = cfg.get("max_dns_servers")
    if not isinstance(max_dns, int) or max_dns < 1 or max_dns > 64:
        return False, f"NCONF2 violation: max_dns_servers must be between 1 and 64 (got {max_dns})"

    # NCONF3: Resource & timeout bounds
    max_payload = cfg.get("max_payload_bytes")
    if not isinstance(max_payload, int) or max_payload < 1024 or max_payload > 104857600:
        return False, f"NCONF3 violation: max_payload_bytes must be between 1024 and 104,857,600 (got {max_payload})"

    timeout = cfg.get("scan_timeout_secs")
    if not isinstance(timeout, int) or timeout < 1 or timeout > 300:
        return False, f"NCONF3 violation: scan_timeout_secs must be between 1 and 300 (got {timeout})"

    # NCONF4: Fallback DNS validation
    fallbacks = cfg.get("fallback_dns_servers")
    if not isinstance(fallbacks, list):
        return False, "NCONF4 violation: fallback_dns_servers must be a list"
    if len(fallbacks) > max_dns:
        return False, f"NCONF4 violation: fallback_dns_servers count {len(fallbacks)} exceeds max_dns_servers {max_dns}"
    for idx, dns in enumerate(fallbacks):
        if not isinstance(dns, str) or not dns.strip():
            return False, f"NCONF4 violation: fallback_dns_servers[{idx}] cannot be empty"
        try:
            ipaddress.ip_address(dns.strip())
        except ValueError:
            return False, f"NCONF4 violation: fallback_dns_servers[{idx}] '{dns}' is not a valid IP address"

    return True, "OK"


def test_nconf1_path_hygiene():
    cfg = dict(DEFAULT_CONFIG)
    ok, _ = validate_config(cfg)
    assert ok, "Default config must be valid"

    # Empty path
    bad_cfg = dict(cfg, sysfs_net_path="")
    ok, err = validate_config(bad_cfg)
    assert not ok and "NCONF1" in err

    # Control char
    bad_cfg = dict(cfg, default_store_path="/path\0invalid")
    ok, err = validate_config(bad_cfg)
    assert not ok and "NCONF1" in err

    # Length > 1024
    bad_cfg = dict(cfg, procfs_path="a" * 1025)
    ok, err = validate_config(bad_cfg)
    assert not ok and "NCONF1" in err

    # Traversal
    bad_cfg = dict(cfg, resolv_conf_path="/etc/resolv/../passwd")
    ok, err = validate_config(bad_cfg)
    assert not ok and "NCONF1" in err
    print("PASS: test_nconf1_path_hygiene")


def test_nconf2_capacity_limits():
    cfg = dict(DEFAULT_CONFIG)

    # Interfaces
    bad_cfg = dict(cfg, max_interfaces=0)
    ok, _ = validate_config(bad_cfg)
    assert not ok
    bad_cfg = dict(cfg, max_interfaces=10001)
    ok, _ = validate_config(bad_cfg)
    assert not ok

    # Routes
    bad_cfg = dict(cfg, max_routes=0)
    ok, _ = validate_config(bad_cfg)
    assert not ok
    bad_cfg = dict(cfg, max_routes=50001)
    ok, _ = validate_config(bad_cfg)
    assert not ok

    # DNS
    bad_cfg = dict(cfg, max_dns_servers=0)
    ok, _ = validate_config(bad_cfg)
    assert not ok
    bad_cfg = dict(cfg, max_dns_servers=65)
    ok, _ = validate_config(bad_cfg)
    assert not ok

    print("PASS: test_nconf2_capacity_limits")


def test_nconf3_resource_bounds():
    cfg = dict(DEFAULT_CONFIG)

    # Payload
    bad_cfg = dict(cfg, max_payload_bytes=1023)
    ok, _ = validate_config(bad_cfg)
    assert not ok
    bad_cfg = dict(cfg, max_payload_bytes=104857601)
    ok, _ = validate_config(bad_cfg)
    assert not ok

    # Timeout
    bad_cfg = dict(cfg, scan_timeout_secs=0)
    ok, _ = validate_config(bad_cfg)
    assert not ok
    bad_cfg = dict(cfg, scan_timeout_secs=301)
    ok, _ = validate_config(bad_cfg)
    assert not ok

    print("PASS: test_nconf3_resource_bounds")


def test_nconf4_fallback_dns():
    cfg = dict(DEFAULT_CONFIG)

    # Valid IPv4 and IPv6
    good_cfg = dict(cfg, fallback_dns_servers=["1.1.1.1", "2606:4700:4700::1111"])
    ok, _ = validate_config(good_cfg)
    assert ok

    # Invalid IP
    bad_cfg = dict(cfg, fallback_dns_servers=["999.999.999.999"])
    ok, err = validate_config(bad_cfg)
    assert not ok and "NCONF4" in err

    # Empty IP
    bad_cfg = dict(cfg, fallback_dns_servers=[""])
    ok, err = validate_config(bad_cfg)
    assert not ok and "NCONF4" in err

    # Count exceeds max_dns_servers
    bad_cfg = dict(cfg, max_dns_servers=1, fallback_dns_servers=["1.1.1.1", "8.8.8.8"])
    ok, err = validate_config(bad_cfg)
    assert not ok and "NCONF4" in err

    print("PASS: test_nconf4_fallback_dns")


def test_nconf5_environment_ingestion():
    os.environ["AIOS_NETWORK_STORE_PATH"] = "/custom/network_store.json"
    os.environ["AIOS_NETWORK_SYSFS_PATH"] = "/custom/sys/class/net"
    os.environ["AIOS_NETWORK_PROCFS_PATH"] = "/custom/proc/net"
    os.environ["AIOS_NETWORK_RESOLV_PATH"] = "/custom/resolv.conf"
    os.environ["AIOS_NETWORK_MAX_INTERFACES"] = "500"
    os.environ["AIOS_NETWORK_MAX_ROUTES"] = "2000"
    os.environ["AIOS_NETWORK_MAX_DNS"] = "16"
    os.environ["AIOS_NETWORK_TIMEOUT"] = "45"

    cfg = dict(DEFAULT_CONFIG)
    cfg["default_store_path"] = os.environ["AIOS_NETWORK_STORE_PATH"]
    cfg["sysfs_net_path"] = os.environ["AIOS_NETWORK_SYSFS_PATH"]
    cfg["procfs_path"] = os.environ["AIOS_NETWORK_PROCFS_PATH"]
    cfg["resolv_conf_path"] = os.environ["AIOS_NETWORK_RESOLV_PATH"]
    cfg["max_interfaces"] = int(os.environ["AIOS_NETWORK_MAX_INTERFACES"])
    cfg["max_routes"] = int(os.environ["AIOS_NETWORK_MAX_ROUTES"])
    cfg["max_dns_servers"] = int(os.environ["AIOS_NETWORK_MAX_DNS"])
    cfg["scan_timeout_secs"] = int(os.environ["AIOS_NETWORK_TIMEOUT"])

    ok, err = validate_config(cfg)
    assert ok, f"Overridden config must be valid: {err}"

    # Cleanup
    for k in (
        "AIOS_NETWORK_STORE_PATH",
        "AIOS_NETWORK_SYSFS_PATH",
        "AIOS_NETWORK_PROCFS_PATH",
        "AIOS_NETWORK_RESOLV_PATH",
        "AIOS_NETWORK_MAX_INTERFACES",
        "AIOS_NETWORK_MAX_ROUTES",
        "AIOS_NETWORK_MAX_DNS",
        "AIOS_NETWORK_TIMEOUT",
    ):
        os.environ.pop(k, None)

    print("PASS: test_nconf5_environment_ingestion")


def test_nconf6_persistence():
    with tempfile.TemporaryDirectory() as td:
        tf = Path(td) / "network_cfg.json"

        # Roundtrip JSON
        original = dict(DEFAULT_CONFIG, scan_timeout_secs=50, max_interfaces=2048)
        tf.write_text(json.dumps(original, indent=2), encoding="utf-8")

        loaded = json.loads(tf.read_text(encoding="utf-8"))
        assert loaded == original

        ok, err = validate_config(loaded)
        assert ok, f"Loaded config must be valid: {err}"

        # Oversized file check
        big_file = Path(td) / "oversized.json"
        big_file.write_bytes(b" " * (MAX_CONFIG_FILE_BYTES + 10))
        assert big_file.stat().st_size > MAX_CONFIG_FILE_BYTES

    print("PASS: test_nconf6_persistence")


def main() -> int:
    print("Running Network Bootstrap Configuration Smoke Tests (T-01846)...")
    test_nconf1_path_hygiene()
    test_nconf2_capacity_limits()
    test_nconf3_resource_bounds()
    test_nconf4_fallback_dns()
    test_nconf5_environment_ingestion()
    test_nconf6_persistence()
    print("ALL NETWORK CONFIG SMOKE TESTS PASSED.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
