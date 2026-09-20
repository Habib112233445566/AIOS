#!/usr/bin/env python3
"""Smoke and Integration test for Network Bootstrap Data Model (T-01806).

Validates:
- Invariants NET1..NET6:
  - NET1: Interface names must be non-empty, <= 15 chars, alphanumeric plus '.', '_', '-'.
  - NET2: MAC addresses must be 6 colon-delimited hex octets (or empty).
  - NET3: IP addresses must have valid prefix lengths (IPv4 <= 32, IPv6 <= 128).
  - NET4: MTU must be between 68 and 65535.
  - NET5: Route must have non-empty destination, metric >= 0, and at least gateway or interface.
  - NET6: Deterministic ordering: interfaces sorted by name, routes sorted by metric then destination.
- JSON schema roundtrip and state inspection helpers.
"""

from __future__ import annotations

import ipaddress
import json
import re
import sys

INTERFACE_NAME_REGEX = re.compile(r"^[a-zA-Z0-9_.-]+$")
MAC_REGEX = re.compile(r"^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$")


def validate_interface_name(name: str) -> bool:
    if not name or len(name) > 15:
        return False
    return bool(INTERFACE_NAME_REGEX.match(name))


def validate_mac_address(mac: str | None) -> bool:
    if mac is None or mac == "":
        return True
    return bool(MAC_REGEX.match(mac))


def validate_ip_address(addr: dict) -> bool:
    address_str = addr.get("address", "")
    prefix = addr.get("prefix_len", 0)
    family = addr.get("family", "")

    try:
        ip = ipaddress.ip_address(address_str)
    except ValueError:
        return False

    if family == "ipv4":
        return ip.version == 4 and 0 <= prefix <= 32
    elif family == "ipv6":
        return ip.version == 6 and 0 <= prefix <= 128
    return False


def validate_mtu(mtu: int) -> bool:
    return 68 <= mtu <= 65535


def validate_route(route: dict) -> bool:
    dst = route.get("destination", "").strip()
    if not dst:
        return False
    # Validate destination CIDR
    try:
        ipaddress.ip_network(dst, strict=False)
    except ValueError:
        return False

    metric = route.get("metric", 0)
    if metric < 0:
        return False

    gateway = route.get("gateway")
    interface = route.get("interface")
    if not gateway and not interface:
        return False

    return True


def validate_network_state(state: dict) -> bool:
    # Validate interfaces
    interfaces = state.get("interfaces", [])
    seen_names = set()
    for iface in interfaces:
        name = iface.get("name", "")
        if not validate_interface_name(name) or name in seen_names:
            return False
        seen_names.add(name)

        if not validate_mac_address(iface.get("mac_address")):
            return False

        if not validate_mtu(iface.get("mtu", 1500)):
            return False

        for addr in iface.get("addresses", []):
            if not validate_ip_address(addr):
                return False

    # Check interface alphabetical ordering
    iface_names = [i.get("name", "") for i in interfaces]
    if iface_names != sorted(iface_names):
        return False

    # Validate routes
    routes = state.get("routes", [])
    for r in routes:
        if not validate_route(r):
            return False

    # Check route ordering: metric ascending, then destination ascending
    route_keys = [(r.get("metric", 0), r.get("destination", "")) for r in routes]
    if route_keys != sorted(route_keys):
        return False

    return True


def test_net1_interface_name():
    valid_names = ["eth0", "wlan0", "lo", "br0", "vlan.100", "enp3s0", "tun-aios"]
    invalid_names = [
        "",
        "a" * 16,  # 16 chars exceeds Linux IFNAMSIZ - 1
        "eth 0",   # contains space
        "eth/0",   # contains slash
        "eth\x00", # contains null char
        "eth@0",   # contains illegal char
    ]

    for name in valid_names:
        assert validate_interface_name(name), f"Expected valid: {name}"
    for name in invalid_names:
        assert not validate_interface_name(name), f"Expected invalid: {name}"
    print("PASS: test_net1_interface_name")


def test_net2_mac_address():
    valid_macs = [
        "00:1A:2B:3C:4D:5E",
        "52:54:00:12:34:56",
        "aa:bb:cc:dd:ee:ff",
        "",
        None,
    ]
    invalid_macs = [
        "00:1A:2B:3C:4D",        # 5 octets
        "00:1A:2B:3C:4D:5E:6F",  # 7 octets
        "GG:1A:2B:3C:4D:5E",     # non-hex
        "00-1A-2B-3C-4D-5E",     # dash delimited
        "001A.2B3C.4D5E",        # cisco format
    ]

    for mac in valid_macs:
        assert validate_mac_address(mac), f"Expected valid: {mac}"
    for mac in invalid_macs:
        assert not validate_mac_address(mac), f"Expected invalid: {mac}"
    print("PASS: test_net2_mac_address")


def test_net3_ip_address():
    valid_addrs = [
        {"family": "ipv4", "address": "192.168.1.100", "prefix_len": 24},
        {"family": "ipv4", "address": "10.0.0.1", "prefix_len": 8},
        {"family": "ipv4", "address": "127.0.0.1", "prefix_len": 8},
        {"family": "ipv4", "address": "0.0.0.0", "prefix_len": 0},
        {"family": "ipv6", "address": "fe80::1", "prefix_len": 64},
        {"family": "ipv6", "address": "::1", "prefix_len": 128},
        {"family": "ipv6", "address": "2001:db8::1", "prefix_len": 32},
    ]
    invalid_addrs = [
        {"family": "ipv4", "address": "192.168.1.100", "prefix_len": 33},  # prefix > 32
        {"family": "ipv4", "address": "999.999.999.999", "prefix_len": 24}, # bad IP
        {"family": "ipv6", "address": "fe80::1", "prefix_len": 129},        # prefix > 128
        {"family": "ipv6", "address": "192.168.1.1", "prefix_len": 24},     # wrong family
        {"family": "ipv4", "address": "fe80::1", "prefix_len": 64},         # wrong family
    ]

    for addr in valid_addrs:
        assert validate_ip_address(addr), f"Expected valid: {addr}"
    for addr in invalid_addrs:
        assert not validate_ip_address(addr), f"Expected invalid: {addr}"
    print("PASS: test_net3_ip_address")


def test_net4_mtu():
    assert validate_mtu(68)      # Min IPv4
    assert validate_mtu(1500)    # Standard Ethernet
    assert validate_mtu(9000)    # Jumbo frame
    assert validate_mtu(65535)   # Max 16-bit MTU
    assert not validate_mtu(67)  # Below min
    assert not validate_mtu(0)   # Zero
    assert not validate_mtu(65536) # Above max
    print("PASS: test_net4_mtu")


def test_net5_routes():
    valid_routes = [
        {"destination": "0.0.0.0/0", "gateway": "192.168.1.1", "interface": "eth0", "metric": 100},
        {"destination": "192.168.1.0/24", "interface": "eth0", "metric": 0},
        {"destination": "::/0", "gateway": "fe80::1", "interface": "eth0", "metric": 1024},
    ]
    invalid_routes = [
        {"destination": "", "gateway": "192.168.1.1", "interface": "eth0", "metric": 100}, # empty dst
        {"destination": "not_a_cidr", "gateway": "192.168.1.1", "interface": "eth0", "metric": 100},
        {"destination": "0.0.0.0/0", "gateway": "192.168.1.1", "interface": "eth0", "metric": -1}, # negative metric
        {"destination": "0.0.0.0/0", "metric": 100}, # no gateway and no interface
    ]

    for route in valid_routes:
        assert validate_route(route), f"Expected valid: {route}"
    for route in invalid_routes:
        assert not validate_route(route), f"Expected invalid: {route}"
    print("PASS: test_net5_routes")


def test_net6_network_state_schema_and_ordering():
    sample_state = {
        "timestamp": "2026-09-20T08:00:00Z",
        "hostname": "aios-node-01",
        "interfaces": [
            {
                "name": "eth0",
                "interface_type": "ethernet",
                "oper_state": "up",
                "mac_address": "52:54:00:12:34:56",
                "mtu": 1500,
                "addresses": [
                    {"family": "ipv4", "address": "192.168.1.50", "prefix_len": 24},
                    {"family": "ipv6", "address": "fe80::5054:ff:fe12:3456", "prefix_len": 64},
                ],
                "flags": ["up", "broadcast", "running", "multicast"],
            },
            {
                "name": "lo",
                "interface_type": "loopback",
                "oper_state": "up",
                "mac_address": None,
                "mtu": 65536,  # 65536 will fail MTU check -> test MTU bound
            },
        ],
        "routes": [
            {"destination": "0.0.0.0/0", "gateway": "192.168.1.1", "interface": "eth0", "metric": 100},
            {"destination": "192.168.1.0/24", "interface": "eth0", "metric": 100},
        ],
        "dns": {
            "nameservers": ["1.1.1.1", "8.8.8.8"],
            "search_domains": ["local"],
        },
    }

    # lo MTU 65536 is invalid
    assert not validate_network_state(sample_state)

    # Fix lo MTU to 65535
    sample_state["interfaces"][1]["mtu"] = 65535
    assert validate_network_state(sample_state)

    # Test out-of-order interfaces: "lo" before "eth0"
    sample_state["interfaces"] = [sample_state["interfaces"][1], sample_state["interfaces"][0]]
    assert not validate_network_state(sample_state)

    # Restore sorted interfaces
    sample_state["interfaces"].sort(key=lambda i: i["name"])
    assert validate_network_state(sample_state)

    # Roundtrip JSON test
    encoded = json.dumps(sample_state, indent=2)
    decoded = json.loads(encoded)
    assert decoded["hostname"] == "aios-node-01"
    assert len(decoded["interfaces"]) == 2
    assert decoded["interfaces"][0]["name"] == "eth0"
    assert decoded["interfaces"][1]["name"] == "lo"
    assert decoded["dns"]["nameservers"] == ["1.1.1.1", "8.8.8.8"]
    print("PASS: test_net6_network_state_schema_and_ordering")


def main():
    print("Starting Network Bootstrap Data Model Smoke Suite (NET1..NET6)...")
    test_net1_interface_name()
    test_net2_mac_address()
    test_net3_ip_address()
    test_net4_mtu()
    test_net5_routes()
    test_net6_network_state_schema_and_ordering()
    print("ALL 6 NETWORK BOOTSTRAP DATA MODEL INTEGRATION TESTS PASSED.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
