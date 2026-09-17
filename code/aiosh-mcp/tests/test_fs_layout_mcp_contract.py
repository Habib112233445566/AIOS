#!/usr/bin/env python3
"""MCP Contract Unit Test for the Filesystem Layout surface (T-01535).

Focused, standalone unit-level contract checks for the three defects the audit
probed on the T-01534 surface. Every case is exercised through the real
`aiosh-mcp` binary over stdio (JSON-RPC 2.0), with the CLI used only to mint a
PEP grant and to read back audit rows:

C1 argument-contract  every argument an arm reads is advertised in `inputSchema`,
                      and nothing else is (all tools keep `additionalProperties:
                      false`). Pins `get.layout_id`/`get.store_path` and
                      `validate.store_path`, which were missing from the manifest.
C2 audit target       `register` records the layout id as the audit target for
                      BOTH accepted input forms (`spec` path and inline `layout`),
                      on the success row **and** on a body refusal, per spec §9 —
                      the spec-path form used to log `None` on both, and after the
                      first fix still logged `None` on a refusal, which left a
                      duplicate-id attempt with no layout-queryable row.
                      Documented boundary: a *pre-gate* refusal (no PEP grant)
                      cannot name a spec-file layout id, because resolving it would
                      require parsing the caller's path before authorization — the
                      F-1/FIFO hazard T-01531 closed. That boundary is asserted here
                      so it stays a decision rather than an untested accident.
C3 destructive verdict `set_active` reports `destructive_transition: true` for a
                      partition shrink and `false` when the transition only grows.
C4 negative cases     ungranted mutation, missing store_path, oversize store_path
                      and a missing tool name are asserted, not just the happy path.

Run standalone:
    python code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py

Exit code 0 = all criteria pass.
"""

import copy
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

UEFI_ID = "aios-uefi-standard-v1"

#: The single source of truth for what each arm reads. The manifest must advertise
#: exactly this set per tool (byte-for-byte the same table lives in the in-tree
#: `FS_LAYOUT_TOOL_ARGUMENTS`).
TOOL_ARGUMENTS = {
    "aios.fs_layout.get": ["layout_id", "profile", "store_path", "grant_id"],
    "aios.fs_layout.validate": ["spec", "layout", "store_path", "grant_id"],
    "aios.fs_layout.fstab": ["profile", "spec", "grant_id"],
    "aios.fs_layout.list": ["store_path", "grant_id"],
    "aios.fs_layout.probe": ["layout_id", "target_disk_bytes", "store_path", "grant_id"],
    "aios.fs_layout.diff": ["source_id", "target_id", "store_path", "grant_id"],
    "aios.fs_layout.register": ["layout", "spec", "store_path", "grant_id"],
    "aios.fs_layout.set_active": ["layout_id", "store_path", "grant_id"],
    "aios.fs_layout.remove": ["layout_id", "store_path", "grant_id"],
    "aios.fs_layout.import_fstab": [
        "layout_id", "name", "fstab", "base_layout_id", "store_path", "grant_id",
    ],
}


# ---------------------------------------------------------------------------
# Harness
# ---------------------------------------------------------------------------

def _find_binary(names):
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in names:
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return names[-1]


def get_mcp_binary():
    return _find_binary(["aiosh-mcp.exe", "aiosh-mcp"])


def get_cli_binary():
    return _find_binary(["aiosh.exe", "aiosh"])


def run_mcp(payload, timeout_s=30):
    p = subprocess.Popen([get_mcp_binary()], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
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
    return json.loads(stdout.strip())


def call_mcp_tool(tool_name, arguments=None, timeout_s=30):
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


def advertised_tools():
    resp = run_mcp({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}})
    return {t["name"]: t for t in resp.get("result", {}).get("tools", [])}


def create_pep_grant(tools="aios.fs_layout.*"):
    cp = subprocess.run(
        [get_cli_binary(), "grant", "create", "--to", "agent:mcp-contract", "--tools", tools],
        capture_output=True, text=True, timeout=60,
    )
    if cp.returncode == 0:
        try:
            return json.loads(cp.stdout).get("data", {}).get("grant_id")
        except Exception:
            return None
    return None


def audit_rows(n=60):
    """Read audit rows through the CLI (same audit DB the MCP server writes)."""
    cp = subprocess.run([get_cli_binary(), "audit", "tail", "--json", "-n", str(n)],
                        capture_output=True, text=True, timeout=60)
    data = json.loads(cp.stdout)["data"]
    return data if isinstance(data, list) else data.get("rows", [])


def err_text(res):
    return str(res.get("error") or res.get("reason") or "")


def base_layout():
    cp = subprocess.run([get_cli_binary(), "layout", "show", UEFI_ID, "--json"],
                        capture_output=True, text=True, timeout=60)
    return json.loads(cp.stdout)["data"]


# ---------------------------------------------------------------------------
# C1 — the advertised schema equals what the arms accept
# ---------------------------------------------------------------------------

def test_c1_argument_contract():
    tools = advertised_tools()
    missing_tools = sorted(set(TOOL_ARGUMENTS) - set(tools))
    assert not missing_tools, f"tools/list is missing {missing_tools}"

    for name, accepted in TOOL_ARGUMENTS.items():
        schema = tools[name]["inputSchema"]
        props = schema.get("properties", {})
        advertised = sorted(props)
        assert advertised == sorted(accepted), (
            f"{name}: manifest advertises {advertised} but the arm accepts {sorted(accepted)}"
        )
        assert schema.get("additionalProperties") is False, (
            f"{name}: additionalProperties must stay false, got {schema.get('additionalProperties')!r}"
        )

    # The two specific regressions: `get` must advertise its widened parameters and
    # `validate` must advertise the parity-only store_path.
    get_props = sorted(tools["aios.fs_layout.get"]["inputSchema"]["properties"])
    assert "layout_id" in get_props and "store_path" in get_props, f"get schema: {get_props}"
    assert "store_path" in tools["aios.fs_layout.validate"]["inputSchema"]["properties"]

    # A mutation's required list still names store_path (unchanged rule).
    for name in ("register", "set_active", "remove", "import_fstab"):
        required = tools[f"aios.fs_layout.{name}"]["inputSchema"].get("required", [])
        assert "store_path" in required, f"aios.fs_layout.{name} must require store_path: {required}"

    print(f"PASS: C1 advertised inputSchema == accepted arguments for all {len(TOOL_ARGUMENTS)} tools")


# ---------------------------------------------------------------------------
# C2 — register records the layout id as the audit target for both input forms
# ---------------------------------------------------------------------------

def test_c2_register_audit_target_both_forms():
    with tempfile.TemporaryDirectory() as td:
        store = str(Path(td) / "store.json")
        grant = create_pep_grant()
        assert grant, "failed to mint a PEP grant for aios.fs_layout.*"

        spec_path_layout = copy.deepcopy(base_layout())
        spec_path_layout["id"] = "contract-spec-v1"
        spec_path_layout["name"] = "Contract Spec Form"
        spec_file = Path(td) / "spec.json"
        spec_file.write_text(json.dumps(spec_path_layout), encoding="utf-8")

        inline_layout = copy.deepcopy(base_layout())
        inline_layout["id"] = "contract-inline-v1"
        inline_layout["name"] = "Contract Inline Form"

        res_spec = call_mcp_tool("aios.fs_layout.register",
                                 {"spec": str(spec_file), "store_path": store, "grant_id": grant})
        assert res_spec.get("ok") is True, f"spec-path register failed: {res_spec}"
        res_inline = call_mcp_tool("aios.fs_layout.register",
                                   {"layout": inline_layout, "store_path": store, "grant_id": grant})
        assert res_inline.get("ok") is True, f"inline register failed: {res_inline}"

        # A pre-gate refusal: the caller supplies no grant, so the gate refuses before
        # the body runs and no layout id can be resolved without an unauthorized read.
        ungranted_spec = call_mcp_tool("aios.fs_layout.register",
                                       {"spec": str(spec_file), "store_path": store})
        assert ungranted_spec.get("ok") is False, ungranted_spec

        # Refusal rows: re-registering the same ids is refused by the store *after* the
        # spec is parsed, so both refusal rows must still name the layout they tried to
        # add — this is the half of defect 2 the first pass missed (only `Ok` consulted
        # the body-resolved target).
        dup_spec = call_mcp_tool("aios.fs_layout.register",
                                 {"spec": str(spec_file), "store_path": store, "grant_id": grant})
        assert dup_spec.get("ok") is False, f"duplicate spec register must be refused: {dup_spec}"
        dup_inline = call_mcp_tool("aios.fs_layout.register",
                                   {"layout": inline_layout, "store_path": store, "grant_id": grant})
        assert dup_inline.get("ok") is False, f"duplicate inline register must be refused: {dup_inline}"

        rows = {row["id"]: row for row in audit_rows() if row.get("tool") == "aios.fs_layout.register"}

        def assert_row(form, res, expected_target, expected_outcome):
            audit_id = res.get("audit_id")
            assert audit_id in rows, f"{form}: audit row {audit_id} not found; rows={sorted(rows)}"
            row = rows[audit_id]
            assert row.get("tool") == "aios.fs_layout.register", row
            assert row.get("outcome") == expected_outcome, (
                f"{form}: outcome {row.get('outcome')!r}, expected {expected_outcome!r}"
            )
            assert row.get("target") == expected_target, (
                f"{form}: audit target is {row.get('target')!r}, expected {expected_target!r}"
            )

        assert_row("spec path (success)", res_spec, "contract-spec-v1", "ok")
        assert_row("inline layout (success)", res_inline, "contract-inline-v1", "ok")
        assert_row("spec path (duplicate refusal)", dup_spec, "contract-spec-v1", "error")
        assert_row("inline layout (duplicate refusal)", dup_inline, "contract-inline-v1", "error")

        # Documented boundary (see module docstring C2): the pre-gate refusal ran no
        # body, so it carries no layout id. Asserted explicitly so the asymmetry is a
        # recorded decision, not a silent hole a later green run could hide.
        assert_row("spec path (pre-gate refusal)", ungranted_spec, None, "refused")

        print("PASS: C2 register records the layout id as the audit target for both input "
              "forms, on success and on body refusal (pre-gate boundary asserted)")


# ---------------------------------------------------------------------------
# C3 — destructive_transition is computed, not hard-coded
# ---------------------------------------------------------------------------

def test_c3_destructive_transition_verdict():
    with tempfile.TemporaryDirectory() as td:
        store = str(Path(td) / "destructive.json")
        grant = create_pep_grant()
        assert grant, "failed to mint a PEP grant for aios.fs_layout.*"

        # A quarter-size copy of every preset partition is a shrink -> destructive.
        shrink = copy.deepcopy(base_layout())
        shrink["id"] = "contract-shrink-v1"
        shrink["name"] = "Contract Shrunk"
        for part in shrink["partitions"]:
            part["size_mib"] = max(1, part["size_mib"] // 4)

        reg = call_mcp_tool("aios.fs_layout.register",
                            {"layout": shrink, "store_path": store, "grant_id": grant})
        assert reg.get("ok") is True, f"register failed: {reg}"

        res_shrink = call_mcp_tool("aios.fs_layout.set_active",
                                   {"layout_id": "contract-shrink-v1",
                                    "store_path": store, "grant_id": grant})
        assert res_shrink.get("ok") is True, f"set_active failed: {res_shrink}"
        assert res_shrink.get("previous_active") == UEFI_ID, res_shrink
        assert res_shrink.get("destructive_transition") is True, (
            f"shrink must report destructive_transition=true: {res_shrink}"
        )

        # Growing back to the preset is not destructive.
        res_grow = call_mcp_tool("aios.fs_layout.set_active",
                                 {"layout_id": UEFI_ID, "store_path": store, "grant_id": grant})
        assert res_grow.get("destructive_transition") is False, (
            f"growing back must report destructive_transition=false: {res_grow}"
        )

        # The shared diff verdict for the same transition agrees with set_active.
        diff = call_mcp_tool("aios.fs_layout.diff",
                             {"source_id": UEFI_ID, "target_id": "contract-shrink-v1",
                              "store_path": store})
        assert diff.get("ok") is True and diff["diff"]["destructive"] is True, diff

        print("PASS: C3 destructive_transition true on shrink, false on grow, matches diff")


# ---------------------------------------------------------------------------
# C4 — negative cases
# ---------------------------------------------------------------------------

def test_c4_negative_cases():
    with tempfile.TemporaryDirectory() as td:
        store = str(Path(td) / "negative.json")

        # Ungranted mutation is refused by the PEP gate and writes nothing.
        res = call_mcp_tool("aios.fs_layout.remove", {"layout_id": UEFI_ID, "store_path": store})
        assert res.get("ok") is False and res.get("gate") == "pep", f"ungranted: {res}"
        assert not Path(store).exists(), "a refusal must not create the store"

        grant = create_pep_grant()
        assert grant, "failed to mint a PEP grant for aios.fs_layout.*"

        # Missing store_path with a valid grant.
        res = call_mcp_tool("aios.fs_layout.set_active", {"layout_id": UEFI_ID, "grant_id": grant})
        assert res.get("ok") is False and "store_path is required" in err_text(res), res

        # Oversize store_path is rejected on both the mutation and the read schema parity path.
        res = call_mcp_tool("aios.fs_layout.remove",
                            {"layout_id": UEFI_ID, "store_path": "a" * 1025, "grant_id": grant})
        assert res.get("ok") is False and "1024" in err_text(res), res
        res = call_mcp_tool("aios.fs_layout.validate", {"spec": "{}", "store_path": "a" * 1025})
        assert res.get("ok") is False and "1024" in err_text(res), res

        # Unknown tool name fails loudly (isError), never silently succeeds.
        resp = run_mcp({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                        "params": {"name": "aios.fs_layout.nope", "arguments": {}}})
        result = resp.get("result", {})
        assert result.get("isError") is True, f"unknown tool must set isError: {resp}"
        body = result.get("structuredContent", {}).get("result", {})
        assert body.get("ok") is False and "unknown tool" in str(body.get("error")), resp

        print("PASS: C4 negative cases (ungranted, missing/oversize store_path, unknown tool)")


def main():
    print("=== RUNNING FILESYSTEM LAYOUT MCP CONTRACT UNIT TESTS ===")
    test_c1_argument_contract()
    test_c2_register_audit_target_both_forms()
    test_c3_destructive_transition_verdict()
    test_c4_negative_cases()
    print("\nALL FILESYSTEM LAYOUT MCP CONTRACT CRITERIA PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
