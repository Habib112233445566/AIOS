#!/usr/bin/env python3
"""CLI Smoke & Integration Test for Network Bootstrap (T-01826).

Exercises the operator surface `aiosh net <subcommand>` / `aiosh network <subcommand>`
end-to-end through the real compiled binary: exit codes, JSON result envelopes,
path hygiene, and domain behavior.

Coverage:
- Help output and unknown subcommands.
- Path hygiene (length > 1024, control characters).
- Argument validation (missing interface, invalid interface name).
- Mock filesystem operations: list, show, routes, dns, state (JSON & human output).
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def get_binary_path() -> str:
    candidates = [
        ROOT / "code/aiosh-rust/target/debug/aiosh.exe",
        ROOT / "code/aiosh-rust/target/debug/aiosh",
        ROOT / "target/debug/aiosh.exe",
        ROOT / "target/debug/aiosh",
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    return "aiosh"


def run_aiosh(*args: str) -> subprocess.CompletedProcess:
    res = subprocess.run([get_binary_path(), *args], capture_output=True, text=True, timeout=60)
    return res


def parse_json_output(res: subprocess.CompletedProcess) -> dict | list:
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)


def expect_code(res: subprocess.CompletedProcess, code: int, context: str) -> None:
    assert res.returncode == code, (
        f"{context}: expected exit code {code}, got {res.returncode}\n"
        f"stdout: {res.stdout.strip()}\nstderr: {res.stderr.strip()}"
    )


def json_envelope(res: subprocess.CompletedProcess, expected_code: int, context: str) -> dict:
    payload = parse_json_output(res)
    assert isinstance(payload, dict), f"{context}: expected a JSON object envelope"
    assert payload.get("code") == expected_code, (
        f"{context}: envelope code {payload.get('code')} != {expected_code}"
    )
    assert "data" in payload and "error" in payload, f"{context}: envelope missing data/error keys"
    return payload


def test_network_help_and_unknown() -> None:
    for cmd in ("net", "network"):
        res = run_aiosh(cmd, "--help")
        expect_code(res, 0, f"{cmd} --help")
        for token in ("aiosh net", "list", "show", "routes", "dns", "state", "up", "down"):
            assert token in res.stdout, f"{cmd} --help missing token {token!r}"

    res_unknown = run_aiosh("net", "bogus_subcmd", "--json")
    expect_code(res_unknown, 2, "net bogus_subcmd")
    assert json_envelope(res_unknown, 2, "unknown subcommand")["error"]["code"] == "UNKNOWN_SUBCOMMAND"
    print("PASS: test_network_help_and_unknown")


def test_network_path_hygiene() -> None:
    # 1. Path length > 1024
    res_long_sys = run_aiosh("net", "list", "--sysfs", "a" * 1025, "--json")
    expect_code(res_long_sys, 2, "net list sysfs too long")
    assert json_envelope(res_long_sys, 2, "sysfs too long")["error"]["code"] == "PATH_TOO_LONG"

    # 2. Control character in procfs
    res_ctrl_proc = run_aiosh("net", "routes", "--procfs", "bad\x07path", "--json")
    expect_code(res_ctrl_proc, 2, "net routes procfs control char")
    assert json_envelope(res_ctrl_proc, 2, "procfs control char")["error"]["code"] == "PATH_CONTAINS_CONTROL_CHAR"

    # 3. Control character in resolv
    res_ctrl_resolv = run_aiosh("net", "dns", "--resolv", "bad\x07file", "--json")
    expect_code(res_ctrl_resolv, 2, "net dns resolv control char")
    assert json_envelope(res_ctrl_resolv, 2, "resolv control char")["error"]["code"] == "PATH_CONTAINS_CONTROL_CHAR"
    print("PASS: test_network_path_hygiene")


def test_network_arg_validation() -> None:
    # 1. Missing interface name for show
    res_no_iface = run_aiosh("net", "show", "--json")
    expect_code(res_no_iface, 2, "net show no iface")
    assert json_envelope(res_no_iface, 2, "missing iface")["error"]["code"] == "MISSING_INTERFACE_NAME"

    # 2. Invalid interface name (characters)
    res_bad_chars = run_aiosh("net", "show", "eth0;reboot", "--json")
    expect_code(res_bad_chars, 2, "net show invalid name chars")
    assert json_envelope(res_bad_chars, 2, "invalid name chars")["error"]["code"] == "INVALID_INTERFACE_NAME"

    # 3. Invalid interface name (too long > 15 chars)
    res_too_long = run_aiosh("net", "show", "interface_too_long_12345", "--json")
    expect_code(res_too_long, 2, "net show name too long")
    assert json_envelope(res_too_long, 2, "name too long")["error"]["code"] == "INVALID_INTERFACE_NAME"

    # 4. Missing interface for up / down
    res_up_missing = run_aiosh("net", "up", "--json")
    expect_code(res_up_missing, 2, "net up missing iface")
    assert json_envelope(res_up_missing, 2, "up missing iface")["error"]["code"] == "MISSING_INTERFACE_NAME"

    res_down_missing = run_aiosh("net", "down", "--json")
    expect_code(res_down_missing, 2, "net down missing iface")
    assert json_envelope(res_down_missing, 2, "down missing iface")["error"]["code"] == "MISSING_INTERFACE_NAME"
    print("PASS: test_network_arg_validation")


def test_network_mock_operations() -> None:
    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp = Path(tmp_dir)
        sysfs = tmp / "sys" / "class" / "net"
        proc = tmp / "proc" / "net"
        resolv = tmp / "etc" / "resolv.conf"

        eth0 = sysfs / "eth0"
        eth0.mkdir(parents=True)
        proc.mkdir(parents=True)
        resolv.parent.mkdir(parents=True)

        (eth0 / "operstate").write_text("up\n", encoding="utf-8")
        (eth0 / "type").write_text("1\n", encoding="utf-8")
        (eth0 / "address").write_text("52:54:00:12:34:56\n", encoding="utf-8")
        (eth0 / "mtu").write_text("1500\n", encoding="utf-8")
        (eth0 / "flags").write_text("0x1003\n", encoding="utf-8")

        route_content = (
            "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n"
            "eth0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n"
            "eth0\t0001A8C0\t00000000\t0001\t0\t0\t50\t00FFFFFF\t0\t0\t0\n"
        )
        (proc / "route").write_text(route_content, encoding="utf-8")

        resolv_content = "nameserver 1.1.1.1\nnameserver 8.8.8.8\nsearch aios.local\n"
        resolv.write_text(resolv_content, encoding="utf-8")

        # 1. list
        res_list = run_aiosh("net", "list", "--sysfs", str(sysfs), "--json")
        expect_code(res_list, 0, "net list json")
        env_list = json_envelope(res_list, 0, "net list json")
        assert env_list["data"]["count"] == 1
        assert env_list["data"]["interfaces"][0]["name"] == "eth0"

        res_list_human = run_aiosh("net", "list", "--sysfs", str(sysfs))
        expect_code(res_list_human, 0, "net list human")
        assert "eth0" in res_list_human.stdout

        # 2. show
        res_show = run_aiosh("net", "show", "eth0", "--sysfs", str(sysfs), "--json")
        expect_code(res_show, 0, "net show json")
        env_show = json_envelope(res_show, 0, "net show json")
        assert env_show["data"]["interface"]["name"] == "eth0"
        assert env_show["data"]["interface"]["mac_address"] == "52:54:00:12:34:56"

        res_show_human = run_aiosh("net", "show", "eth0", "--sysfs", str(sysfs))
        expect_code(res_show_human, 0, "net show human")
        assert "52:54:00:12:34:56" in res_show_human.stdout

        # 3. show not found
        res_notfound = run_aiosh("net", "show", "nonexistent", "--sysfs", str(sysfs), "--json")
        expect_code(res_notfound, 1, "net show not found json")
        assert json_envelope(res_notfound, 1, "show not found")["error"]["code"] == "INTERFACE_NOT_FOUND"

        # 4. routes
        res_routes = run_aiosh("net", "routes", "--procfs", str(proc), "--json")
        expect_code(res_routes, 0, "net routes json")
        env_routes = json_envelope(res_routes, 0, "net routes json")
        assert env_routes["data"]["count"] == 2
        assert env_routes["data"]["routes"][0]["metric"] == 50

        res_routes_human = run_aiosh("net", "routes", "--procfs", str(proc))
        expect_code(res_routes_human, 0, "net routes human")
        assert "Routing Table" in res_routes_human.stdout

        # 5. dns
        res_dns = run_aiosh("net", "dns", "--resolv", str(resolv), "--json")
        expect_code(res_dns, 0, "net dns json")
        env_dns = json_envelope(res_dns, 0, "net dns json")
        assert env_dns["data"]["dns"]["nameservers"] == ["1.1.1.1", "8.8.8.8"]
        assert env_dns["data"]["dns"]["search_domains"] == ["aios.local"]

        res_dns_human = run_aiosh("net", "dns", "--resolv", str(resolv))
        expect_code(res_dns_human, 0, "net dns human")
        assert "1.1.1.1" in res_dns_human.stdout

        # 6. state
        res_state = run_aiosh(
            "net", "state",
            "--sysfs", str(sysfs),
            "--procfs", str(proc),
            "--resolv", str(resolv),
            "--json"
        )
        expect_code(res_state, 0, "net state json")
        env_state = json_envelope(res_state, 0, "net state json")
        assert len(env_state["data"]["state"]["interfaces"]) == 1
        assert len(env_state["data"]["state"]["routes"]) == 2
        assert env_state["data"]["state"]["dns"]["nameservers"] == ["1.1.1.1", "8.8.8.8"]

        res_state_human = run_aiosh(
            "net", "state",
            "--sysfs", str(sysfs),
            "--procfs", str(proc),
            "--resolv", str(resolv)
        )
        expect_code(res_state_human, 0, "net state human")
        assert "Network State" in res_state_human.stdout

    print("PASS: test_network_mock_operations")


def main():
    print("Starting Network Bootstrap CLI Surface Smoke Suite...")
    test_network_help_and_unknown()
    test_network_path_hygiene()
    test_network_arg_validation()
    test_network_mock_operations()
    print("ALL NETWORK BOOTSTRAP CLI SURFACE INTEGRATION TESTS PASSED.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
