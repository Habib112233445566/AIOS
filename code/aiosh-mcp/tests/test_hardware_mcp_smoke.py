#!/usr/bin/env python3
"""Cross-Surface Integration Smoke Test for Hardware Detection MCP Surface (T-01736).

Proves that all 5 Hardware Detection MCP tools are exposed via JSON-RPC,
adhere to schema contracts, enforce bounds and security sanitization, and achieve
cross-surface state parity with the operator CLI.

Covers:
- Tool registration: All 5 `aios.hardware.*` tools advertised via tools/list.
- Full lifecycle: scan -> list -> get -> summary -> verify.
- Cross-surface parity: Device inventory discovered via CLI matches MCP.
- Security bounds: Refusal of paths and IDs with control characters or exceeding size limits.

Run standalone:
    python code/aiosh-mcp/tests/test_hardware_mcp_smoke.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

HARDWARE_TOOLS = (
    "aios.hardware.scan",
    "aios.hardware.list",
    "aios.hardware.get",
    "aios.hardware.summary",
    "aios.hardware.verify",
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


def list_mcp_tools() -> dict[str, dict]:
    resp = run_mcp({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}})
    return {t["name"]: t for t in resp.get("result", {}).get("tools", [])}


def run_cli(*args: str, expect: int = 0) -> subprocess.CompletedProcess:
    cp = subprocess.run([get_cli_binary(), *args], capture_output=True, text=True, timeout=60)
    assert cp.returncode == expect, (
        f"{' '.join(args)}: expected exit {expect}, got {cp.returncode}\n"
        f"stdout: {cp.stdout.strip()}\nstderr: {cp.stderr.strip()}"
    )
    return cp


def test_tools_list_advertising() -> None:
    tools = list_mcp_tools()
    for expected_tool in HARDWARE_TOOLS:
        assert expected_tool in tools, f"Missing MCP tool {expected_tool}"
        tool_def = tools[expected_tool]
        assert "description" in tool_def
        schema = tool_def.get("inputSchema", {})
        assert schema.get("type") == "object"
        assert schema.get("additionalProperties") is False, f"{expected_tool} must set additionalProperties: false"
    print("PASS: test_tools_list_advertising (all 5 tools advertised)")


def test_mcp_hardware_lifecycle() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        sys_dir = Path(tmpdir) / "sys"
        proc_dir = Path(tmpdir) / "proc"
        pci_dir = sys_dir / "bus/pci/devices/0000_01_00.0"
        pci_dir.mkdir(parents=True, exist_ok=True)
        proc_dir.mkdir(parents=True, exist_ok=True)

        (pci_dir / "vendor").write_text("0x10de\n")
        (pci_dir / "device").write_text("0x2684\n")
        (pci_dir / "class").write_text("0x030000\n")

        sys_str = str(sys_dir)
        proc_str = str(proc_dir)

        # 1. scan
        res_scan = call_mcp_tool("aios.hardware.scan", {"sysfs_path": sys_str, "procfs_path": proc_str})
        assert res_scan.get("ok") is True, f"scan failed: {res_scan}"
        data = res_scan.get("data", {})
        devices = data.get("devices", [])
        assert len(devices) == 1
        assert devices[0]["id"] == "pci:0000:01:00.0"
        assert devices[0]["class"] == "gpu"
        assert data.get("summary", {}).get("gpu") == 1

        # 2. list
        res_list = call_mcp_tool("aios.hardware.list", {"sysfs_path": sys_str, "procfs_path": proc_str})
        assert res_list.get("ok") is True
        assert res_list.get("data", {}).get("count") == 1

        # 3. list with class filter
        res_list_gpu = call_mcp_tool("aios.hardware.list", {
            "sysfs_path": sys_str,
            "procfs_path": proc_str,
            "classes": ["gpu"],
        })
        assert res_list_gpu.get("data", {}).get("count") == 1

        res_list_block = call_mcp_tool("aios.hardware.list", {
            "sysfs_path": sys_str,
            "procfs_path": proc_str,
            "classes": ["block"],
        })
        assert res_list_block.get("data", {}).get("count") == 0

        # 4. get existing device
        res_get = call_mcp_tool("aios.hardware.get", {
            "device_id": "pci:0000:01:00.0",
            "sysfs_path": sys_str,
            "procfs_path": proc_str,
        })
        assert res_get.get("ok") is True
        assert res_get.get("data", {}).get("device", {}).get("id") == "pci:0000:01:00.0"

        # 5. get missing device
        res_miss = call_mcp_tool("aios.hardware.get", {
            "device_id": "pci:nonexistent",
            "sysfs_path": sys_str,
            "procfs_path": proc_str,
        })
        assert res_miss.get("ok") is False

        # 6. summary
        res_sum = call_mcp_tool("aios.hardware.summary", {"sysfs_path": sys_str, "procfs_path": proc_str})
        assert res_sum.get("ok") is True
        assert res_sum.get("data", {}).get("total") == 1
        assert res_sum.get("data", {}).get("summary", {}).get("gpu") == 1

        # 7. verify live
        res_ver_live = call_mcp_tool("aios.hardware.verify", {"sysfs_path": sys_str, "procfs_path": proc_str})
        assert res_ver_live.get("ok") is True
        assert res_ver_live.get("data", {}).get("valid") is True

        # 8. verify file valid
        inv_file = Path(tmpdir) / "valid_inv.json"
        inv_file.write_text(json.dumps({
            "timestamp": "2026-09-20T07:00:00Z",
            "hostname": "test-node",
            "architecture": "x86_64",
            "kernel_version": "6.6.13",
            "devices": [],
            "summary": {},
        }))
        res_ver_file = call_mcp_tool("aios.hardware.verify", {"file_path": str(inv_file)})
        assert res_ver_file.get("ok") is True
        assert res_ver_file.get("data", {}).get("valid") is True

        # 9. verify file corrupt
        corrupt_file = Path(tmpdir) / "corrupt_inv.json"
        corrupt_file.write_text("{ corrupt json")
        res_ver_corrupt = call_mcp_tool("aios.hardware.verify", {"file_path": str(corrupt_file)})
        assert res_ver_corrupt.get("ok") is False

    print("PASS: test_mcp_hardware_lifecycle (all 5 tools operational)")


def test_cross_surface_parity() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        sys_dir = Path(tmpdir) / "sys"
        proc_dir = Path(tmpdir) / "proc"
        pci_dir = sys_dir / "bus/pci/devices/0000_01_00.0"
        pci_dir.mkdir(parents=True, exist_ok=True)
        proc_dir.mkdir(parents=True, exist_ok=True)

        (pci_dir / "vendor").write_text("0x10de\n")
        (pci_dir / "device").write_text("0x2684\n")
        (pci_dir / "class").write_text("0x030000\n")

        sys_str = str(sys_dir)
        proc_str = str(proc_dir)

        # Query via CLI JSON
        cli_scan = run_cli("hw", "scan", "--sysfs", sys_str, "--procfs", proc_str, "--json")
        cli_data = json.loads(cli_scan.stdout)["data"]

        # Query via MCP
        mcp_res = call_mcp_tool("aios.hardware.scan", {"sysfs_path": sys_str, "procfs_path": proc_str})
        mcp_data = mcp_res["data"]

        assert len(cli_data["devices"]) == len(mcp_data["devices"])
        assert cli_data["devices"][0]["id"] == mcp_data["devices"][0]["id"]
        assert cli_data["summary"] == mcp_data["summary"]

        # Show via CLI vs get via MCP
        cli_show = run_cli("hw", "show", "pci:0000:01:00.0", "--sysfs", sys_str, "--procfs", proc_str, "--json")
        cli_dev = json.loads(cli_show.stdout)["data"]["device"]

        mcp_get = call_mcp_tool("aios.hardware.get", {
            "device_id": "pci:0000:01:00.0",
            "sysfs_path": sys_str,
            "procfs_path": proc_str,
        })
        mcp_dev = mcp_get["data"]["device"]

        assert cli_dev["id"] == mcp_dev["id"]
        assert cli_dev["vendor_id"] == mcp_dev["vendor_id"]
        assert cli_dev["device_id"] == mcp_dev["device_id"]

    print("PASS: test_cross_surface_parity (CLI and MCP return identical state)")


def test_security_bounds() -> None:
    # 1. Path length > 1024
    long_path = "a" * 1025
    res_long = call_mcp_tool("aios.hardware.scan", {"sysfs_path": long_path})
    assert res_long.get("ok") is False

    # 2. Control character in path
    res_ctrl = call_mcp_tool("aios.hardware.scan", {"procfs_path": "bad\x07proc"})
    assert res_ctrl.get("ok") is False

    # 3. Missing / empty device_id
    res_empty_id = call_mcp_tool("aios.hardware.get", {"device_id": "   "})
    assert res_empty_id.get("ok") is False

    # 4. Control character in device_id
    res_ctrl_id = call_mcp_tool("aios.hardware.get", {"device_id": "bad\x07id"})
    assert res_ctrl_id.get("ok") is False

    # 5. Device ID > 256
    res_long_id = call_mcp_tool("aios.hardware.get", {"device_id": "x" * 257})
    assert res_long_id.get("ok") is False

    # 6. Unrecognized device class
    res_bad_class = call_mcp_tool("aios.hardware.scan", {"classes": ["unknown_class_xyz"]})
    assert res_bad_class.get("ok") is False

    print("PASS: test_security_bounds (all boundary violations refused)")


def main() -> None:
    print(f"Running Hardware Detection MCP smoke suite with binary: {get_mcp_binary()}")
    test_tools_list_advertising()
    test_mcp_hardware_lifecycle()
    test_cross_surface_parity()
    test_security_bounds()
    print("ALL TESTS PASSED: aios.hardware MCP smoke test suite.")


if __name__ == "__main__":
    main()
