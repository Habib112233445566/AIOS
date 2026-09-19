#!/usr/bin/env python3
"""Observability Test Suite for Filesystem Layout (T-01573..T-01576).

Criteria FL12:
  O1: Telemetry emission completeness (CLI & MCP invocations emit audit rows)
  O2: Audit correlation & queryability (filter by tool and target via audit tail)
  O3: Outcome fidelity (ok/success, error, refused correctly logged)
  O4: State inspection parity (active layout & profile counts observable on CLI & MCP)
  O5: Destructive mutation flagging (destructive: true reported on partition shrink/delete)

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_observability.py
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


def test_o1_audit_emission_completeness(tmp_dir: Path) -> None:
    """O1: Telemetry emission completeness (CLI & MCP invocations emit audit rows)."""
    # 1. CLI call
    cp_cli = subprocess.run(
        [get_cli_binary(), "layout", "validate", "--standard", "--json"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert cp_cli.returncode == 0, f"CLI validate failed: {cp_cli.stderr}"
    res_cli = json.loads(cp_cli.stdout)
    assert res_cli.get("code") == 0

    # 2. MCP call
    res_mcp = call_mcp_tool("aios.fs_layout.get", {"layout_id": UEFI_ID})
    assert res_mcp.get("ok") is True, f"MCP get failed: {res_mcp}"
    mcp_audit_id = res_mcp.get("audit_id")

    # 3. Check audit log
    rows = audit_rows(n=30)
    mcp_row = next((r for r in rows if r.get("id") == mcp_audit_id), None)
    assert mcp_row is not None, f"MCP audit row {mcp_audit_id} not found in audit log"
    assert mcp_row.get("tool") == "aios.fs_layout.get"

    cli_rows = [r for r in rows if r.get("tool") == "fs_layout" and r.get("command") == "validate"]
    assert len(cli_rows) > 0, "CLI validate audit row not found in audit log"

    print("PASS: O1 Telemetry emission completeness verified for CLI and MCP")


def test_o2_audit_correlation_queryability(tmp_dir: Path) -> None:
    """O2: Audit correlation & queryability (filter by tool and target via audit tail)."""
    store = str(tmp_dir / "store_o2.json")
    grant = create_pep_grant()
    assert grant, "Failed to mint PEP grant"

    # Register a unique layout
    layout = copy.deepcopy(base_layout())
    layout_id = "o2-obs-layout-v1"
    layout["id"] = layout_id
    layout["name"] = "Observability Test Layout"

    res_reg = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": layout, "store_path": store, "grant_id": grant},
    )
    assert res_reg.get("ok") is True, f"register failed: {res_reg}"
    audit_id = res_reg.get("audit_id")

    # Query audit rows and correlate by target
    rows = audit_rows(n=50)
    matched = [r for r in rows if r.get("target") == layout_id]
    assert len(matched) > 0, f"No audit rows found for target {layout_id}"
    row = matched[-1]
    assert row.get("id") == audit_id
    assert row.get("tool") == "aios.fs_layout.register"
    assert row.get("outcome") in ("ok", "success")

    print("PASS: O2 Audit correlation & queryability verified by target and tool")


def test_o3_outcome_fidelity(tmp_dir: Path) -> None:
    """O3: Outcome fidelity (ok/success, error, refused correctly logged)."""
    store = str(tmp_dir / "store_o3.json")
    grant = create_pep_grant()
    assert grant, "Failed to mint PEP grant"

    # 1. Success outcome
    layout = copy.deepcopy(base_layout())
    layout["id"] = "o3-success-v1"
    res_ok = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": layout, "store_path": store, "grant_id": grant},
    )
    assert res_ok.get("ok") is True
    ok_id = res_ok.get("audit_id")

    # 2. Error outcome (duplicate registration)
    res_err = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": layout, "store_path": store, "grant_id": grant},
    )
    assert res_err.get("ok") is False
    err_id = res_err.get("audit_id")

    # 3. Refused outcome (ungranted mutation)
    res_refused = call_mcp_tool(
        "aios.fs_layout.remove",
        {"layout_id": "o3-success-v1", "store_path": store},
    )
    assert res_refused.get("ok") is False
    assert res_refused.get("gate") == "pep"
    refused_id = res_refused.get("audit_id")

    # Query rows
    rows_by_id = {r.get("id"): r for r in audit_rows(n=60)}
    assert ok_id in rows_by_id, f"Audit row {ok_id} missing"
    assert rows_by_id[ok_id].get("outcome") in ("ok", "success")

    assert err_id in rows_by_id, f"Audit row {err_id} missing"
    assert rows_by_id[err_id].get("outcome") == "error"

    assert refused_id in rows_by_id, f"Audit row {refused_id} missing"
    assert rows_by_id[refused_id].get("outcome") == "refused"

    print("PASS: O3 Outcome fidelity verified (ok, error, refused)")


def test_o4_state_inspection_parity(tmp_dir: Path) -> None:
    """O4: State inspection parity (active layout & profile counts observable on CLI & MCP)."""
    # CLI inspection
    cp_cli = subprocess.run(
        [get_cli_binary(), "layout", "list", "--json"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert cp_cli.returncode == 0, f"CLI list failed: {cp_cli.stderr}"
    data_cli = json.loads(cp_cli.stdout)["data"]
    cli_active = data_cli.get("active_layout")
    cli_count = len(data_cli.get("layouts", []))

    # MCP inspection
    res_mcp = call_mcp_tool("aios.fs_layout.list", {})
    assert res_mcp.get("ok") is True, f"MCP list failed: {res_mcp}"
    mcp_active = res_mcp.get("active_layout")
    mcp_count = len(res_mcp.get("layouts", []))

    assert cli_active == mcp_active, f"Active layout mismatch: CLI={cli_active} vs MCP={mcp_active}"
    assert cli_count == mcp_count, f"Layout count mismatch: CLI={cli_count} vs MCP={mcp_count}"

    print(f"PASS: O4 State inspection parity verified (active={cli_active}, count={cli_count})")


def test_o5_destructive_mutation_flagging(tmp_dir: Path) -> None:
    """O5: Destructive mutation flagging (destructive: true reported on partition shrink/delete)."""
    store = str(tmp_dir / "store_o5.json")
    grant = create_pep_grant()
    assert grant, "Failed to mint PEP grant"

    # Create a shrunk layout
    shrunk = copy.deepcopy(base_layout())
    shrunk["id"] = "o5-shrunk-v1"
    shrunk["name"] = "Shrunk Layout"
    for part in shrunk["partitions"]:
        part["size_mib"] = max(1, part["size_mib"] // 2)

    res_reg = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": shrunk, "store_path": store, "grant_id": grant},
    )
    assert res_reg.get("ok") is True

    # Diff against UEFI
    res_diff = call_mcp_tool(
        "aios.fs_layout.diff",
        {"source_id": UEFI_ID, "target_id": "o5-shrunk-v1", "store_path": store},
    )
    assert res_diff.get("ok") is True
    diff_data = res_diff.get("diff", {})
    assert diff_data.get("destructive") is True, f"Expected destructive: true, got: {diff_data}"

    # Non-destructive diff (same layout)
    res_same = call_mcp_tool(
        "aios.fs_layout.diff",
        {"source_id": UEFI_ID, "target_id": UEFI_ID, "store_path": store},
    )
    assert res_same.get("ok") is True
    assert res_same.get("diff", {}).get("destructive") is False

    print("PASS: O5 Destructive mutation flagging verified (true on shrink, false on identical)")


def main() -> int:
    print("=== RUNNING FILESYSTEM LAYOUT OBSERVABILITY TEST SUITE (FL12) ===")
    with tempfile.TemporaryDirectory() as td:
        tmp_dir = Path(td)
        test_o1_audit_emission_completeness(tmp_dir)
        test_o2_audit_correlation_queryability(tmp_dir)
        test_o3_outcome_fidelity(tmp_dir)
        test_o4_state_inspection_parity(tmp_dir)
        test_o5_destructive_mutation_flagging(tmp_dir)
    print("PASS: All Observability criteria O1..O5 passed successfully.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
