#!/usr/bin/env python3
"""Cross-Surface Integration Smoke Test for Network Bootstrap MCP Surface (T-01836).

Proves that all 7 Network Bootstrap MCP tools are exposed via JSON-RPC,
adhere to schema contracts, enforce bounds and security sanitization, and achieve
cross-surface state parity with the operator CLI.

Covers:
- Tool registration: All 7 `aios.network.*` tools advertised via tools/list.
- Full lifecycle: list -> show -> routes -> dns -> state -> up -> down.
- Cross-surface parity: Interface inventory discovered via CLI matches MCP.
- Security bounds: Refusal of paths and interface names with control characters or exceeding size limits.

Run standalone:
    python code/aiosh-mcp/tests/test_network_mcp_smoke.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

NETWORK_TOOLS = (
    "aios.network.list",
    "aios.network.show",
    "aios.network.routes",
    "aios.network.dns",
    "aios.network.state",
    "aios.network.up",
    "aios.network.down",
)


def _find_binary(names: list[str]) -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in names:
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return names[-1]


def get_mcp_binary() -> str:
    return _find_binary(["aiosh-mcp.exe", "aiosh-mcp"])


def get_cli_binary() -> str:
    return _find_binary(["aiosh.exe", "aiosh"])


def run_mcp(payload: dict, timeout_s: int = 30) -> dict:
    p = subprocess.Popen(
        [get_mcp_binary()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
    )
    try:
        stdout, _ = p.communicate(json.dumps(payload) + "\n", timeout=timeout_s)
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
        print(f"FAIL: aiosh-mcp timed out after {timeout_s}s")
        sys.exit(1)
    if p.returncode != 0:
        print(f"FAIL: aiosh-mcp returned {p.returncode}")
        sys.exit(1)
    try:
        return json.loads(stdout.strip())
    except Exception as e:
        print(f"FAIL: invalid JSON from aiosh-mcp: {e}")
        print(stdout)
        sys.exit(1)


def call_mcp_tool(tool_name: str, arguments: dict | None = None, timeout_s: int = 30) -> dict:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments or {}},
    }
    resp = run_mcp(payload, timeout_s=timeout_s)
    if "error" in resp:
        return {"ok": False, "error": resp["error"]}
    res = resp.get("result", {})
    if "structuredContent" in res and "result" in res["structuredContent"]:
        data = res["structuredContent"]["result"]
    elif "content" in res and res["content"]:
        try:
            data = json.loads(res["content"][0]["text"])
        except Exception:
            data = {"ok": False, "text": res["content"][0]["text"]}
    else:
        data = dict(res)
    if res.get("isError") is True and "ok" not in data:
        data["ok"] = False
    return data


def test_tool_registration() -> None:
    payload = {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}
    resp = run_mcp(payload)
    assert "result" in resp, f"tools/list missing result: {resp}"
    tools = resp["result"].get("tools", [])
    tool_names = {t["name"] for t in tools}
    for expected in NETWORK_TOOLS:
        assert expected in tool_names, f"tool {expected} not registered in MCP tools/list"
    print("PASS: test_tool_registration (all 7 network tools advertised)")


def test_path_hygiene_and_validation() -> None:
    # 1. Path length > 1024
    res_long = call_mcp_tool("aios.network.list", {"sysfs_path": "a" * 1025})
    assert res_long.get("ok") is False, f"expected failure for long path: {res_long}"

    # 2. Control character in procfs
    res_ctrl = call_mcp_tool("aios.network.routes", {"procfs_path": "bad\x07path"})
    assert res_ctrl.get("ok") is False, f"expected failure for control char: {res_ctrl}"

    # 3. Missing interface argument
    res_no_iface = call_mcp_tool("aios.network.show", {})
    assert res_no_iface.get("ok") is False, f"expected failure for missing interface: {res_no_iface}"

    # 4. Invalid interface characters
    res_bad_iface = call_mcp_tool("aios.network.show", {"interface": "eth0;reboot"})
    assert res_bad_iface.get("ok") is False, f"expected failure for bad interface chars: {res_bad_iface}"

    # 5. Up / Down missing interface
    assert call_mcp_tool("aios.network.up", {}).get("ok") is False
    assert call_mcp_tool("aios.network.down", {}).get("ok") is False
    print("PASS: test_path_hygiene_and_validation")


def test_mock_lifecycle_and_cross_surface_parity() -> None:
    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp = Path(tmp_dir)
        sys_dir = tmp / "sys" / "class" / "net"
        proc_dir = tmp / "proc" / "net"
        resolv_file = tmp / "etc" / "resolv.conf"

        eth0 = sys_dir / "eth0"
        eth0.mkdir(parents=True)
        proc_dir.mkdir(parents=True)
        resolv_file.parent.mkdir(parents=True)

        (eth0 / "operstate").write_text("up\n", encoding="utf-8")
        (eth0 / "type").write_text("1\n", encoding="utf-8")
        (eth0 / "address").write_text("52:54:00:12:34:56\n", encoding="utf-8")
        (eth0 / "mtu").write_text("1500\n", encoding="utf-8")
        (eth0 / "flags").write_text("0x1003\n", encoding="utf-8")

        route_content = (
            "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n"
            "eth0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n"
        )
        (proc_dir / "route").write_text(route_content, encoding="utf-8")

        resolv_content = "nameserver 1.1.1.1\nnameserver 8.8.8.8\nsearch aios.local\n"
        resolv_file.write_text(resolv_content, encoding="utf-8")

        sys_str = str(sys_dir)
        proc_str = str(proc_dir)
        resolv_str = str(resolv_file)

        # 1. MCP aios.network.list
        res_mcp_list = call_mcp_tool("aios.network.list", {
            "sysfs_path": sys_str,
            "procfs_path": proc_str,
            "resolv_path": resolv_str,
        })
        assert res_mcp_list.get("ok") is True, f"MCP list failed: {res_mcp_list}"
        mcp_data = res_mcp_list.get("data", {})
        assert mcp_data.get("count") == 1
        assert mcp_data["interfaces"][0]["name"] == "eth0"

        # 2. CLI cross-surface parity comparison
        cli_res = subprocess.run(
            [get_cli_binary(), "net", "list", "--sysfs", sys_str, "--json"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert cli_res.returncode == 0, f"CLI net list failed: {cli_res.stderr}"
        cli_data = json.loads(cli_res.stdout.strip())
        assert cli_data.get("code") == 0
        assert cli_data["data"]["count"] == mcp_data["count"]
        assert cli_data["data"]["interfaces"][0]["name"] == mcp_data["interfaces"][0]["name"]
        assert cli_data["data"]["interfaces"][0]["mac_address"] == mcp_data["interfaces"][0]["mac_address"]

        # 3. MCP aios.network.show
        res_mcp_show = call_mcp_tool("aios.network.show", {
            "interface": "eth0",
            "sysfs_path": sys_str,
        })
        assert res_mcp_show.get("ok") is True
        assert res_mcp_show["data"]["interface"]["mac_address"] == "52:54:00:12:34:56"

        res_mcp_show_nf = call_mcp_tool("aios.network.show", {
            "interface": "eth99",
            "sysfs_path": sys_str,
        })
        assert res_mcp_show_nf.get("ok") is False

        # 4. MCP aios.network.routes
        res_mcp_routes = call_mcp_tool("aios.network.routes", {
            "procfs_path": proc_str,
        })
        assert res_mcp_routes.get("ok") is True
        assert res_mcp_routes["data"]["count"] == 1
        assert res_mcp_routes["data"]["routes"][0]["interface"] == "eth0"

        # 5. MCP aios.network.dns
        res_mcp_dns = call_mcp_tool("aios.network.dns", {
            "resolv_path": resolv_str,
        })
        assert res_mcp_dns.get("ok") is True
        assert res_mcp_dns["data"]["dns"]["nameservers"] == ["1.1.1.1", "8.8.8.8"]

        # 6. MCP aios.network.state
        res_mcp_state = call_mcp_tool("aios.network.state", {
            "sysfs_path": sys_str,
            "procfs_path": proc_str,
            "resolv_path": resolv_str,
        })
        assert res_mcp_state.get("ok") is True
        state_data = res_mcp_state["data"]["state"]
        assert len(state_data["interfaces"]) == 1
        assert len(state_data["routes"]) == 1
        assert state_data["dns"]["nameservers"] == ["1.1.1.1", "8.8.8.8"]

        # 7. MCP aios.network.up & down
        res_up = call_mcp_tool("aios.network.up", {
            "interface": "eth0",
            "sysfs_path": sys_str,
        })
        assert res_up.get("ok") is True
        assert res_up["data"]["status"] == "up"

        res_down = call_mcp_tool("aios.network.down", {
            "interface": "eth0",
            "sysfs_path": sys_str,
        })
        assert res_down.get("ok") is True
        assert res_down["data"]["status"] == "down"

    print("PASS: test_mock_lifecycle_and_cross_surface_parity")


def main():
    print("Starting Network Bootstrap MCP Integration Smoke Suite...")
    test_tool_registration()
    test_path_hygiene_and_validation()
    test_mock_lifecycle_and_cross_surface_parity()
    print("ALL 7 NETWORK BOOTSTRAP MCP INTEGRATION TESTS PASSED.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
