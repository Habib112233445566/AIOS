#!/usr/bin/env python3
"""Cross-Surface Integration Smoke Test for Kernel Module Management MCP Surface (T-01636).

Proves that all 10 Kernel Module Management MCP tools are exposed via JSON-RPC,
adhere to schema contracts, enforce bounds and security sanitization, and achieve
cross-surface state parity with the operator CLI.

Covers:
- Tool registration: All 10 `aios.kernel_module.*` tools advertised via tools/list.
- Full lifecycle: list -> get -> blacklist -> autoload conflict -> unblacklist ->
  autoload -> blacklist conflict -> unautoload -> options -> preset.list ->
  preset.apply -> export.
- Cross-surface parity: Modifications made via CLI are visible to MCP, and vice versa.
- Security bounds: Refusal of paths with control characters and paths exceeding 1024 bytes.

Run standalone:
    python code/aiosh-mcp/tests/test_kernel_module_mcp_smoke.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

KERNEL_MODULE_TOOLS = (
    "aios.kernel_module.list",
    "aios.kernel_module.get",
    "aios.kernel_module.blacklist",
    "aios.kernel_module.unblacklist",
    "aios.kernel_module.options",
    "aios.kernel_module.autoload",
    "aios.kernel_module.unautoload",
    "aios.kernel_module.preset.list",
    "aios.kernel_module.preset.apply",
    "aios.kernel_module.export",
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
    for expected_tool in KERNEL_MODULE_TOOLS:
        assert expected_tool in tools, f"Missing MCP tool {expected_tool}"
        tool_def = tools[expected_tool]
        assert "description" in tool_def
        assert "inputSchema" in tool_def
    print("PASS: test_tools_list_advertising (all 10 tools advertised)")


def test_mcp_kernel_module_lifecycle() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = str(Path(tmpdir) / "km_store.json")
        proc_path = str(Path(tmpdir) / "proc_modules.txt")
        with open(proc_path, "w") as f:
            f.write("overlay 151552 1 - Live 0x0000000000000000\next4 983040 2 - Live 0x0000000000000000\n")

        # 1. list
        res = call_mcp_tool("aios.kernel_module.list", {
            "store_path": store_path,
            "proc_modules_path": proc_path,
        })
        assert res.get("ok") is True, f"list failed: {res}"
        loaded = res.get("data", {}).get("loaded_modules", [])
        assert len(loaded) == 2, f"expected 2 loaded modules, got {loaded}"
        assert loaded[0]["name"] == "overlay"

        # 2. get live module
        res = call_mcp_tool("aios.kernel_module.get", {
            "module": "overlay",
            "store_path": store_path,
            "proc_modules_path": proc_path,
        })
        assert res.get("ok") is True, f"get failed: {res}"
        mod = res.get("data", {}).get("module", {})
        assert mod.get("name") == "overlay"

        # 3. get missing module
        res = call_mcp_tool("aios.kernel_module.get", {
            "module": "nonexistent_mod",
            "store_path": store_path,
            "proc_modules_path": proc_path,
        })
        assert res.get("ok") is False, f"expected failure for missing module: {res}"

        # 4. blacklist
        res = call_mcp_tool("aios.kernel_module.blacklist", {
            "module": "usb_storage",
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"blacklist failed: {res}"

        # 5. conflict: autoload blacklisted module
        res = call_mcp_tool("aios.kernel_module.autoload", {
            "module": "usb_storage",
            "store_path": store_path,
        })
        assert res.get("ok") is False, f"expected conflict error: {res}"

        # 6. unblacklist
        res = call_mcp_tool("aios.kernel_module.unblacklist", {
            "module": "usb_storage",
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"unblacklist failed: {res}"

        # 7. autoload
        res = call_mcp_tool("aios.kernel_module.autoload", {
            "module": "br_netfilter",
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"autoload failed: {res}"

        # 8. conflict: blacklist autoloaded module
        res = call_mcp_tool("aios.kernel_module.blacklist", {
            "module": "br_netfilter",
            "store_path": store_path,
        })
        assert res.get("ok") is False, f"expected conflict error: {res}"

        # 9. unautoload
        res = call_mcp_tool("aios.kernel_module.unautoload", {
            "module": "br_netfilter",
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"unautoload failed: {res}"

        # 10. options
        res = call_mcp_tool("aios.kernel_module.options", {
            "module": "e1000e",
            "options": ["InterruptThrottleRate=1"],
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"options failed: {res}"

        # 11. preset list
        res = call_mcp_tool("aios.kernel_module.preset.list", {})
        assert res.get("ok") is True, f"preset.list failed: {res}"
        presets = res.get("data", [])
        preset_names = [p.get("name") for p in presets]
        assert "cis_hardened_baseline" in preset_names

        # 12. preset apply
        res = call_mcp_tool("aios.kernel_module.preset.apply", {
            "preset_name": "cis_hardened_baseline",
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"preset.apply failed: {res}"

        # 13. export
        res = call_mcp_tool("aios.kernel_module.export", {
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"export failed: {res}"
        export_data = res.get("data", {})
        assert "install cramfs /bin/true" in export_data.get("modprobe_conf", "")

    print("PASS: test_mcp_kernel_module_lifecycle")


def test_cross_surface_cli_mcp_parity() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = str(Path(tmpdir) / "parity_store.json")

        # Step 1: Blacklist module via CLI
        run_cli("mod", "blacklist", "floppy", "--reason", "cli-legacy", "--store", store_path, "--json")

        # Step 2: Read via MCP
        res = call_mcp_tool("aios.kernel_module.list", {
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"MCP list failed after CLI blacklist: {res}"
        rules = res.get("data", {}).get("rules", [])
        floppy_blacklisted = any(
            isinstance(r, dict) and r.get("blacklist", {}).get("module") == "floppy"
            for r in rules
        ) or any(
            isinstance(r, dict) and r.get("module") == "floppy"
            for r in rules
        )
        assert floppy_blacklisted, f"floppy not found in rules: {rules}"

        # Step 3: Autoload module via MCP
        res = call_mcp_tool("aios.kernel_module.autoload", {
            "module": "overlay",
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"MCP autoload failed: {res}"

        # Step 4: Verify via CLI
        cp = run_cli("mod", "list", "--store", store_path, "--json")
        cli_data = json.loads(cp.stdout)["data"]
        autoload_names = [m["name"] if isinstance(m, dict) else m for m in cli_data.get("autoload_modules", [])]
        assert "overlay" in autoload_names

    print("PASS: test_cross_surface_cli_mcp_parity")


def test_security_bounds_and_error_handling() -> None:
    # 1. Path with control character
    res = call_mcp_tool("aios.kernel_module.list", {"store_path": "invalid\x07path.json"})
    assert res.get("ok") is False or "error" in res, f"Expected rejection for control char path: {res}"

    # 2. Oversized path (> 1024)
    res = call_mcp_tool("aios.kernel_module.list", {"store_path": "a" * 1025})
    assert res.get("ok") is False or "error" in res, f"Expected rejection for oversized path: {res}"

    # 3. Missing parameter for tool requiring module
    res = call_mcp_tool("aios.kernel_module.blacklist", {"store_path": "some_path.json"})
    assert res.get("ok") is False or "error" in res, f"Expected rejection for missing module: {res}"

    print("PASS: test_security_bounds_and_error_handling")


def main() -> None:
    print(f"Using MCP binary: {get_mcp_binary()}")
    print(f"Using CLI binary: {get_cli_binary()}")
    test_tools_list_advertising()
    test_mcp_kernel_module_lifecycle()
    test_cross_surface_cli_mcp_parity()
    test_security_bounds_and_error_handling()
    print("ALL TESTS PASSED.")


if __name__ == "__main__":
    main()
