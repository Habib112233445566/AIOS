#!/usr/bin/env python3
"""Recovery & Validation Test Suite for Filesystem Layout (T-01593..T-01596).

Criteria FL14:
  R1: Corruption refusal & containment (invalid/truncated store refused; not overwritten)
  R2: Fallback to canonical presets (absent store returns in-memory presets cleanly)
  R3: Recovery via valid replacement (replacing corrupt store restores service)
  R4: Atomic write crash consistency (failed write leaves existing store intact)
  R5: Audit trail continuity (honest audit rows for failure and recovery)

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_recovery_validation.py
"""

from __future__ import annotations

import copy
import hashlib
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


def get_cli_binary() -> str:
    return _find_binary(["aiosh.exe", "aiosh"])


def get_mcp_binary() -> str:
    return _find_binary(["aiosh-mcp.exe", "aiosh-mcp"])


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


def test_r1_corruption_refusal_containment(tmp_dir: Path) -> None:
    """R1: Corruption refusal & containment (invalid/truncated store refused; not overwritten)."""
    corrupt_file = tmp_dir / "corrupted_store.json"
    corrupt_content = '{"active_layout": "foo", "layouts": [ invalid json'
    corrupt_file.write_text(corrupt_content, encoding="utf-8")
    orig_hash = hashlib.sha256(corrupt_file.read_bytes()).hexdigest()

    # CLI read attempt
    cp = subprocess.run(
        [get_cli_binary(), "layout", "list", "--store", str(corrupt_file), "--json"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert cp.returncode == 1, f"CLI list against corrupted store must return exit 1, got {cp.returncode}"
    res_cli = json.loads(cp.stdout)
    assert res_cli.get("error", {}).get("code") == "LOAD_STORE_FAILED"

    # MCP read attempt
    res_mcp = call_mcp_tool("aios.fs_layout.list", {"store_path": str(corrupt_file)})
    assert res_mcp.get("ok") is False, f"MCP list against corrupt store must fail: {res_mcp}"

    # Verify containment: corrupt file was NOT rewritten or modified
    assert hashlib.sha256(corrupt_file.read_bytes()).hexdigest() == orig_hash, (
        "Corruption containment failure: file was altered during load refusal"
    )

    print("PASS: R1 Corruption refusal & containment verified (tamper-resistant, fail-closed)")


def test_r2_fallback_to_canonical_presets(tmp_dir: Path) -> None:
    """R2: Fallback to canonical presets (absent store returns in-memory presets cleanly)."""
    # CLI call without store
    cp = subprocess.run(
        [get_cli_binary(), "layout", "list", "--json"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert cp.returncode == 0, f"CLI list failed: {cp.stderr}"
    data = json.loads(cp.stdout)["data"]
    layout_ids = [l["id"] for l in data.get("layouts", [])]
    assert UEFI_ID in layout_ids, f"Default preset {UEFI_ID} missing from in-memory fallback"
    assert CONTAINER_ID in layout_ids, f"Default preset {CONTAINER_ID} missing from in-memory fallback"

    # MCP call without store
    res_mcp = call_mcp_tool("aios.fs_layout.list", {})
    assert res_mcp.get("ok") is True, f"MCP list without store failed: {res_mcp}"
    mcp_layout_ids = [l["id"] for l in res_mcp.get("layouts", [])]
    assert UEFI_ID in mcp_layout_ids and CONTAINER_ID in mcp_layout_ids

    print("PASS: R2 Fallback to canonical presets verified on CLI and MCP")


def test_r3_recovery_via_valid_replacement(tmp_dir: Path) -> None:
    """R3: Recovery via valid replacement (replacing corrupt store restores service)."""
    store_file = tmp_dir / "recovery_store.json"
    # Start corrupted
    store_file.write_text('{"bad": "data"}', encoding="utf-8")

    # Confirm it fails
    res_bad = call_mcp_tool("aios.fs_layout.list", {"store_path": str(store_file)})
    assert res_bad.get("ok") is False

    # Perform external recovery by replacing with valid store
    valid_store = {
        "layouts": {UEFI_ID: base_layout()},
        "active_layout_id": UEFI_ID,
    }
    store_file.write_text(json.dumps(valid_store), encoding="utf-8")

    # Verify immediate recovery on both surfaces
    res_good_mcp = call_mcp_tool("aios.fs_layout.list", {"store_path": str(store_file)})
    assert res_good_mcp.get("ok") is True, f"Recovery failed on MCP: {res_good_mcp}"

    cp = subprocess.run(
        [get_cli_binary(), "layout", "list", "--store", str(store_file), "--json"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert cp.returncode == 0, f"Recovery failed on CLI: {cp.stderr}"

    print("PASS: R3 Recovery via valid replacement restores full functionality")


def test_r4_atomic_write_crash_consistency(tmp_dir: Path) -> None:
    """R4: Atomic write crash consistency (failed write leaves existing store intact)."""
    store_file = tmp_dir / "atomic_store.json"
    grant = create_pep_grant()
    assert grant, "Failed to mint PEP grant"

    # Seed initial valid store
    init_layout = copy.deepcopy(base_layout())
    init_layout["id"] = "init-atomic-v1"
    res_init = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": init_layout, "store_path": str(store_file), "grant_id": grant},
    )
    assert res_init.get("ok") is True
    orig_hash = hashlib.sha256(store_file.read_bytes()).hexdigest()

    # Attempt a mutation that fails validation (mode 0)
    bad_layout = copy.deepcopy(base_layout())
    bad_layout["id"] = "bad-atomic-v1"
    bad_layout["directories"][0]["mode"] = 0

    res_fail = call_mcp_tool(
        "aios.fs_layout.register",
        {"layout": bad_layout, "store_path": str(store_file), "grant_id": grant},
    )
    assert res_fail.get("ok") is False, "Bad layout mutation must be refused"

    # Verify existing store is completely intact
    assert hashlib.sha256(store_file.read_bytes()).hexdigest() == orig_hash, (
        "Crash consistency failure: store was modified during rejected mutation"
    )

    # Verify no staging file leaks
    staged = [p.name for p in tmp_dir.iterdir() if ".tmp." in p.name]
    assert not staged, f"Staged temporary files leaked: {staged}"

    print("PASS: R4 Atomic write crash consistency verified (zero staging leaks, unmodified store)")


def test_r5_audit_trail_continuity(tmp_dir: Path) -> None:
    """R5: Audit trail continuity (honest audit rows for failure and recovery)."""
    store_file = tmp_dir / "audit_recovery_store.json"
    store_file.write_text('{"corrupted": true}', encoding="utf-8")

    # 1. Failed operation
    res_fail = call_mcp_tool("aios.fs_layout.list", {"store_path": str(store_file)})
    assert res_fail.get("ok") is False
    fail_audit_id = res_fail.get("audit_id")

    # 2. Recover store
    valid_store = {
        "layouts": {UEFI_ID: base_layout()},
        "active_layout_id": UEFI_ID,
    }
    store_file.write_text(json.dumps(valid_store), encoding="utf-8")

    # 3. Successful operation
    res_ok = call_mcp_tool("aios.fs_layout.list", {"store_path": str(store_file)})
    assert res_ok.get("ok") is True
    ok_audit_id = res_ok.get("audit_id")

    # Verify audit rows
    rows_by_id = {r.get("id"): r for r in audit_rows(n=50)}
    assert fail_audit_id in rows_by_id, f"Audit row {fail_audit_id} not found"
    assert rows_by_id[fail_audit_id].get("outcome") == "error"

    assert ok_audit_id in rows_by_id, f"Audit row {ok_audit_id} not found"
    assert rows_by_id[ok_audit_id].get("outcome") in ("ok", "success")

    print("PASS: R5 Audit trail continuity verified for both failure and recovery operations")


def main() -> int:
    print("=== RUNNING FILESYSTEM LAYOUT RECOVERY & VALIDATION TEST SUITE (FL14) ===")
    with tempfile.TemporaryDirectory() as td:
        tmp_dir = Path(td)
        test_r1_corruption_refusal_containment(tmp_dir)
        test_r2_fallback_to_canonical_presets(tmp_dir)
        test_r3_recovery_via_valid_replacement(tmp_dir)
        test_r4_atomic_write_crash_consistency(tmp_dir)
        test_r5_audit_trail_continuity(tmp_dir)
    print("PASS: All Recovery & Validation criteria R1..R5 passed successfully.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
