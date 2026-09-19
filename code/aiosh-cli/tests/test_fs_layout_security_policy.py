#!/usr/bin/env python3
"""Security Policy Test Suite for Filesystem Layout (T-01563..T-01566).

Criteria FL11:
  P1: Gated mutation without grant -> refused
  P2: Gated mutation with wrong tool scope -> refused
  P3: Gated mutation with out-of-scope path -> refused
  P4: Gated mutation with valid grant & in-scope paths -> allowed
  P5: Read-only tool execution without grant -> allowed

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_security_policy.py
"""

from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

UEFI_ID = "aios-uefi-standard-v1"
CONTAINER_ID = "aios-container-minimal-v1"


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
        print(f"FAIL: aiosh-mcp timed out after {timeout_s}s", file=sys.stderr)
        sys.exit(1)
    if p.returncode != 0:
        print(f"FAIL: aiosh-mcp returned {p.returncode}", file=sys.stderr)
        sys.exit(1)
    return json.loads(stdout.strip())


def call_mcp_tool(tool_name: str, arguments: dict | None = None, timeout_s: int = 30) -> dict:
    resp = run_mcp({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments or {}},
    }, timeout_s=timeout_s)
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


def create_pep_grant(tools: str = "aios.fs_layout.*", allow: str | None = None, deny: str | None = None) -> str | None:
    cmd = [get_cli_binary(), "grant", "create", "--to", "agent:mcp-contract", "--tools", tools]
    if allow:
        cmd += ["--allow", allow]
    if deny:
        cmd += ["--deny", deny]
    cp = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
    if cp.returncode == 0:
        try:
            return json.loads(cp.stdout).get("data", {}).get("grant_id")
        except Exception:
            return None
    return None


def audit_rows(n: int = 60) -> list[dict]:
    cp = subprocess.run(
        [get_cli_binary(), "audit", "tail", "--json", "-n", str(n)],
        capture_output=True,
        text=True,
        timeout=60,
    )
    data = json.loads(cp.stdout)["data"]
    return data if isinstance(data, list) else data.get("rows", [])


def base_layout() -> dict:
    cp = subprocess.run(
        [get_cli_binary(), "layout", "show", UEFI_ID, "--json"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    return json.loads(cp.stdout)["data"]


def err_text(res: dict) -> str:
    return str(res.get("error") or res.get("reason") or "")


def test_p1_mutation_without_grant(tmp_dir: Path) -> None:
    """P1: Gated mutation without grant -> refused."""
    store = str(tmp_dir / "store_p1.json")
    res = call_mcp_tool("aios.fs_layout.remove", {"layout_id": UEFI_ID, "store_path": store})
    assert res.get("ok") is False, f"P1: Expected failure without grant, got: {res}"
    assert res.get("gate") == "pep", f"P1: Expected PEP gate refusal, got: {res}"
    assert not Path(store).exists(), "P1: Refusal must not create store file"

    # Also verify register without grant
    layout = copy.deepcopy(base_layout())
    layout["id"] = "p1-test-layout"
    res_reg = call_mcp_tool("aios.fs_layout.register", {"layout": layout, "store_path": store})
    assert res_reg.get("ok") is False, f"P1: Expected register failure without grant, got: {res_reg}"
    assert res_reg.get("gate") == "pep", f"P1: Expected register PEP gate refusal, got: {res_reg}"
    assert not Path(store).exists(), "P1: Register refusal must not create store file"
    print("PASS: P1 Gated mutation without grant -> refused by PEP gate")


def test_p2_mutation_wrong_tool_scope(tmp_dir: Path) -> None:
    """P2: Gated mutation with wrong tool scope -> refused."""
    store = str(tmp_dir / "store_p2.json")
    wrong_grant = create_pep_grant(tools="pentest.*")
    assert wrong_grant, "P2: Failed to mint PEP grant for wrong scope"

    layout = copy.deepcopy(base_layout())
    layout["id"] = "p2-test-layout"
    res = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": layout, "store_path": store, "grant_id": wrong_grant},
    )
    assert res.get("ok") is False, f"P2: Expected failure with wrong tool scope, got: {res}"
    assert res.get("gate") == "pep", f"P2: Expected PEP gate refusal, got: {res}"
    assert not Path(store).exists(), "P2: Refusal with wrong scope must not create store file"
    print("PASS: P2 Gated mutation with wrong tool scope -> refused by PEP gate")


def test_p3_mutation_out_of_scope_path(tmp_dir: Path) -> None:
    """P3: Gated mutation with out-of-scope path -> refused."""
    allowed_dir = tmp_dir / "allowed"
    outside_dir = tmp_dir / "outside"
    allowed_dir.mkdir()
    outside_dir.mkdir()

    store_out = str(outside_dir / "store_p3.json")
    grant = create_pep_grant(tools="aios.fs_layout.*", allow=str(allowed_dir))
    assert grant, "P3: Failed to mint path-scoped PEP grant"

    layout = copy.deepcopy(base_layout())
    layout["id"] = "p3-test-layout"
    res = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": layout, "store_path": store_out, "grant_id": grant},
    )
    assert res.get("ok") is False, f"P3: Expected failure with out-of-scope path, got: {res}"
    assert res.get("gate") == "pep", f"P3: Expected PEP gate refusal, got: {res}"
    text = err_text(res)
    assert "path subject" in text and "scope.paths" in text, (
        f"P3: Expected scope.paths refusal naming path subject, got: {text!r}"
    )
    assert not Path(store_out).exists(), "P3: Out-of-scope refusal must not create store file"
    print("PASS: P3 Gated mutation with out-of-scope path -> refused by PEP gate")


def test_p4_mutation_valid_grant(tmp_dir: Path) -> None:
    """P4: Gated mutation with valid grant & in-scope paths -> allowed."""
    allowed_dir = tmp_dir / "allowed_p4"
    allowed_dir.mkdir()
    store_in = str(allowed_dir / "store_p4.json")

    grant = create_pep_grant(tools="aios.fs_layout.*", allow=str(allowed_dir))
    assert grant, "P4: Failed to mint path-scoped PEP grant"

    layout = copy.deepcopy(base_layout())
    layout["id"] = "p4-test-layout"
    res = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": layout, "store_path": store_in, "grant_id": grant},
    )
    assert res.get("ok") is True, f"P4: Expected success with valid grant & in-scope path, got: {res}"
    assert Path(store_in).exists(), "P4: Successful register must write store file"

    # Verify audit row
    rows = {row["id"]: row for row in audit_rows() if row.get("tool") == "aios.fs_layout.register"}
    audit_id = res.get("audit_id")
    assert audit_id in rows, f"P4: Audit row {audit_id} not found in audit log"
    row = rows[audit_id]
    assert row.get("outcome") == "ok", f"P4: Audit outcome expected 'ok', got: {row.get('outcome')}"
    assert row.get("target") == "p4-test-layout", f"P4: Audit target expected 'p4-test-layout', got: {row.get('target')}"
    print("PASS: P4 Gated mutation with valid grant & in-scope paths -> allowed")


def test_p5_readonly_ungated(tmp_dir: Path) -> None:
    """P5: Read-only tool execution without grant -> allowed."""
    # get
    res_get = call_mcp_tool("aios.fs_layout.get", {"layout_id": UEFI_ID})
    assert res_get.get("ok") is True, f"P5: get failed without grant: {res_get}"

    # list
    res_list = call_mcp_tool("aios.fs_layout.list", {})
    assert res_list.get("ok") is True, f"P5: list failed without grant: {res_list}"

    # probe
    res_probe = call_mcp_tool(
        "aios.fs_layout.probe",
        {"layout_id": UEFI_ID, "target_disk_bytes": 100 * 1024 * 1024 * 1024},
    )
    assert res_probe.get("ok") is True, f"P5: probe failed without grant: {res_probe}"

    # diff
    res_diff = call_mcp_tool(
        "aios.fs_layout.diff",
        {"source_id": UEFI_ID, "target_id": CONTAINER_ID},
    )
    assert res_diff.get("ok") is True, f"P5: diff failed without grant: {res_diff}"

    # fstab
    res_fstab = call_mcp_tool("aios.fs_layout.fstab", {"profile": "uefi"})
    assert res_fstab.get("ok") is True, f"P5: fstab failed without grant: {res_fstab}"

    print("PASS: P5 Read-only tools (get, list, probe, diff, fstab) executed without grant -> allowed")


def main() -> int:
    print("=== RUNNING FILESYSTEM LAYOUT SECURITY POLICY TEST SUITE (FL11) ===")
    with tempfile.TemporaryDirectory() as td:
        tmp_dir = Path(td)
        test_p1_mutation_without_grant(tmp_dir)
        test_p2_mutation_wrong_tool_scope(tmp_dir)
        test_p3_mutation_out_of_scope_path(tmp_dir)
        test_p4_mutation_valid_grant(tmp_dir)
        test_p5_readonly_ungated(tmp_dir)
    print("PASS: All Security Policy criteria P1..P5 passed successfully.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
