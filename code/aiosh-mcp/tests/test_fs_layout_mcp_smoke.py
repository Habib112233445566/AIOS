#!/usr/bin/env python3
"""Cross-Surface Integration Smoke Test for Filesystem Layout (T-01526 / T-01534).

Proves the Filesystem Layout surfaces are reachable through their production
entry points and that the operator CLI and the agent MCP surface agree on the
shared canonical JSON layout store.

Covers:
- Discoverability: root `aiosh --help`, the `aiosh layout` / `aiosh fs-layout` routes, and
  `tools/list` advertising all 10 `aios.fs_layout.*` MCP tools (6 read + 4 mutation),
  including the `store_path` parameter and the `store_path`-required rule for mutations.
- Cross-substrate parity on the built-in presets: fstab text, validation verdict, diff verdict,
  and probe evaluation must be identical between CLI and MCP.
- Shared canonical store: a layout registered by the CLI is visible to MCP, and an active-pointer
  switch made by the CLI is observed by MCP (and vice versa).
- Store path semantics/parity: missing file falls back to seeded presets on both surfaces,
  oversized/control-character paths are rejected, corrupt stores surface an error.
- Mutations over stdio (T-01534): all four mutation tools are gated by the PEP grant
  (`require_grant = true`) and require an explicit `store_path`; a granted register/set-active/
  import-fstab/remove round-trip is re-read from disk after every step, and semantic refusals
  leave the store byte-identical.
- CLI <-> MCP mutation parity: a layout registered through either surface is field-for-field
  identical, and each surface observes the other's active-pointer switch.
- Read hardening (T-01534 / F-1): a directory named by `spec`/`fstab` is refused by *type*
  instead of being read, and (on POSIX) a FIFO does not stall the single-threaded request loop.
- Contract + verdict parity (T-01536): the integrated path advertises the arguments it accepts
  (`get.layout_id`, `validate.store_path`), both surfaces record the same audit target for a
  `register`, and the destructive verdict of one transition agrees across `set_active` and the CLI.

Run standalone:
    python code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py
"""

import copy
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

UEFI_ID = "aios-uefi-standard-v1"
CONTAINER_ID = "aios-container-minimal-v1"
CUSTOM_ID = "cross-surface-v1"

READ_TOOLS = (
    "aios.fs_layout.get",
    "aios.fs_layout.validate",
    "aios.fs_layout.fstab",
    "aios.fs_layout.list",
    "aios.fs_layout.probe",
    "aios.fs_layout.diff",
)
MUTATION_TOOLS = (
    "aios.fs_layout.register",
    "aios.fs_layout.set_active",
    "aios.fs_layout.remove",
    "aios.fs_layout.import_fstab",
)
FS_LAYOUT_TOOLS = READ_TOOLS + MUTATION_TOOLS

SMALL_FSTAB = "/dev/sda2 / ext4 defaults 0 1\n"


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


def call_mcp_tool(tool_name, arguments=None, timeout_s=30):
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


def mcp_tools():
    resp = run_mcp({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}})
    return {t["name"]: t for t in resp.get("result", {}).get("tools", [])}


def run_cli(*args, expect=0, context=""):
    cp = subprocess.run([get_cli_binary(), *args], capture_output=True, text=True, timeout=60)
    assert cp.returncode == expect, (
        f"{context or ' '.join(args)}: expected exit {expect}, got {cp.returncode}\n"
        f"stdout: {cp.stdout.strip()}\nstderr: {cp.stderr.strip()}"
    )
    return cp


def cli_json(*args, expect=0, context=""):
    cp = run_cli(*args, expect=expect, context=context)
    return json.loads(cp.stdout.strip())


def create_pep_grant(tools="aios.fs_layout.*"):
    """Mint a real grant through the CLI; the MCP server reads the same audit DB."""
    cp = subprocess.run(
        [get_cli_binary(), "grant", "create", "--to", "agent:mcp-smoke", "--tools", tools],
        capture_output=True,
        text=True,
        timeout=60,
    )
    if cp.returncode == 0:
        try:
            return json.loads(cp.stdout).get("data", {}).get("grant_id")
        except Exception:
            return None
    return None


def err_text(res):
    """dispatch failures surface as `reason`; handler errors as `error`."""
    return str(res.get("error") or res.get("reason") or "")


def cli_audit_rows(n=60):
    """Read audit rows through the CLI (both surfaces share the default audit DB)."""
    cp = subprocess.run([get_cli_binary(), "audit", "tail", "--json", "-n", str(n)],
                        capture_output=True, text=True, timeout=60)
    data = json.loads(cp.stdout)["data"]
    return data if isinstance(data, list) else data.get("rows", [])


def base_layout_json():
    return cli_json("layout", "show", UEFI_ID, "--json", context="show base")["data"]


def custom_layout(base, new_id, new_name):
    spec = copy.deepcopy(base)
    spec["id"] = new_id
    spec["name"] = new_name
    return spec


def store_on_disk(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


# ---------------------------------------------------------------------------
# 1. Discoverability of the production surfaces
# ---------------------------------------------------------------------------

def test_manifest_and_store_path_discoverability():
    tools = mcp_tools()
    missing = [name for name in FS_LAYOUT_TOOLS if name not in tools]
    assert not missing, f"tools/list is missing Filesystem Layout tools: {missing}"

    # The three store-backed read surfaces must advertise the shared store parameter.
    for tool in ("aios.fs_layout.list", "aios.fs_layout.probe", "aios.fs_layout.diff"):
        props = tools[tool]["inputSchema"]["properties"]
        assert "store_path" in props, f"{tool} does not advertise store_path: {sorted(props)}"

    # Mutations advertise store_path as REQUIRED (spec 3.1) and grant_id for the PEP gate.
    for tool in MUTATION_TOOLS:
        schema = tools[tool]["inputSchema"]
        required = schema.get("required", [])
        assert "store_path" in required, f"{tool} must require store_path: {required}"
        assert "grant_id" in schema["properties"], f"{tool} must advertise grant_id"
    for tool in READ_TOOLS:
        assert "store_path" not in tools[tool]["inputSchema"].get("required", []), (
            f"read tool {tool} must not require store_path"
        )

    print(f"PASS: tools/list exposes all 10 aios.fs_layout.* tools; mutations require store_path")


def test_cli_surface_discoverability_and_alias_parity():
    # Root help advertises the surface (registration point).
    helptext = run_cli("--help").stdout
    assert "aiosh layout <list|" in helptext, "root help does not advertise the layout surface"

    # `fs-layout` is a documented compatibility alias for the same production command.
    primary = cli_json("layout", "list", "--json", context="layout list")
    alias = cli_json("fs-layout", "list", "--json", context="fs-layout list")
    assert primary["code"] == 0 and alias["code"] == 0
    assert primary["data"] == alias["data"], "alias produced different canonical data"

    print("PASS: aiosh layout is advertised in root help and reachable via the fs-layout alias")


# ---------------------------------------------------------------------------
# 2. Cross-substrate parity on built-in presets
# ---------------------------------------------------------------------------

def test_cross_surface_parity_on_presets():
    # fstab — canonical text must be byte-identical across surfaces.
    cli_fstab = cli_json("layout", "fstab", "--json", context="cli fstab")["data"]["fstab"]
    mcp_fstab = call_mcp_tool("aios.fs_layout.fstab", {"profile": "standard_uefi"})
    assert mcp_fstab.get("ok") is True, f"MCP fstab failed: {mcp_fstab}"
    assert cli_fstab == mcp_fstab["fstab"], "CLI and MCP fstab text diverged"

    # validate — same verdict for the same preset.
    cli_valid = cli_json("layout", "validate", "--json", context="cli validate")["data"]["valid"]
    mcp_valid = call_mcp_tool("aios.fs_layout.validate")
    assert mcp_valid.get("valid") is True, f"MCP validate failed: {mcp_valid}"
    assert cli_valid is True

    # diff — same destructive verdict and same partition delta.
    cli_diff = cli_json("layout", "diff", UEFI_ID, CONTAINER_ID, "--json",
                        context="cli diff")["data"]
    mcp_diff = call_mcp_tool("aios.fs_layout.diff",
                             {"source_id": UEFI_ID, "target_id": CONTAINER_ID})
    assert mcp_diff.get("ok") is True, f"MCP diff failed: {mcp_diff}"
    mcp_d = mcp_diff["diff"]
    assert cli_diff["destructive"] == mcp_d["destructive"] is True, "destructive verdict diverged"
    assert len(cli_diff["partitions_removed"]) == len(mcp_d["partitions_removed"])
    assert len(cli_diff["mounts_removed"]) == len(mcp_d["mounts_removed"])

    # probe — same viability and same partition budget arithmetic.
    target = 68719476736  # 64 GiB
    cli_probe = cli_json("layout", "probe", "--bytes", str(target), "--json",
                         context="cli probe")["data"]
    mcp_probe = call_mcp_tool("aios.fs_layout.probe", {"target_disk_bytes": target})
    assert mcp_probe.get("ok") is True, f"MCP probe failed: {mcp_probe}"
    mcp_eval = mcp_probe["evaluation"]
    assert cli_probe["is_viable"] == mcp_eval["is_viable"]
    assert cli_probe["partition_budget_bytes"] == mcp_eval["partition_budget_bytes"]
    assert cli_probe["required_disk_bytes"] == mcp_eval["required_disk_bytes"]

    print("PASS: CLI <-> MCP parity on fstab, validate, diff and probe")


# ---------------------------------------------------------------------------
# 3. Shared canonical JSON store
# ---------------------------------------------------------------------------

def test_cross_surface_store_sharing():
    with tempfile.TemporaryDirectory() as td:
        store = str(Path(td) / "fs_layouts.json")

        custom = custom_layout(base_layout_json(), CUSTOM_ID, "Cross Surface Layout")
        spec_file = Path(td) / "custom.json"
        spec_file.write_text(json.dumps(custom), encoding="utf-8")

        # 1. Operator registers a layout through the CLI.
        run_cli("layout", "register", "--spec", str(spec_file), "--store", store, "--json",
                context="cli register")

        # 2. Agent observes the operator-registered layout through the shared store.
        res = call_mcp_tool("aios.fs_layout.list", {"store_path": store})
        assert res.get("ok") is True, f"MCP list with store failed: {res}"
        ids = [layout["id"] for layout in res["layouts"]]
        assert CUSTOM_ID in ids, f"MCP did not observe CLI-registered layout: {ids}"
        assert res["count"] == 3, f"expected 3 layouts in shared store, got {res['count']}"
        assert res["active_layout_id"] == UEFI_ID

        # 3. Without store_path the agent sees only the seeded built-ins (no implicit leakage).
        isolated = call_mcp_tool("aios.fs_layout.list")
        assert isolated.get("ok") is True
        iso_ids = [layout["id"] for layout in isolated["layouts"]]
        assert CUSTOM_ID not in iso_ids, "default MCP store unexpectedly leaked custom layout"
        assert isolated["count"] == 2

        # 4. Agent can diff and probe the operator-registered layout.
        diff_res = call_mcp_tool("aios.fs_layout.diff",
                                 {"source_id": UEFI_ID, "target_id": CUSTOM_ID, "store_path": store})
        assert diff_res.get("ok") is True, f"MCP diff on custom layout failed: {diff_res}"
        assert diff_res["diff"]["target_layout_id"] == CUSTOM_ID

        probe_res = call_mcp_tool("aios.fs_layout.probe",
                                  {"layout_id": CUSTOM_ID,
                                   "target_disk_bytes": 128 * 1024 * 1024 * 1024,
                                   "store_path": store})
        assert probe_res.get("ok") is True, f"MCP probe on custom layout failed: {probe_res}"
        assert probe_res["evaluation"]["is_viable"] is True

        # 5. Operator switches the active pointer; agent observes the same canonical state.
        run_cli("layout", "set-active", CUSTOM_ID, "--store", store, "--json",
                context="cli set-active")
        after = call_mcp_tool("aios.fs_layout.list", {"store_path": store})
        assert after["active_layout_id"] == CUSTOM_ID, (
            f"MCP did not observe CLI active-pointer switch: {after['active_layout_id']}"
        )

        # 6. And the reverse direction: CLI observes what the agent-facing store holds.
        cli_after = cli_json("layout", "list", "--store", store, "--json", context="cli list")["data"]
        assert cli_after["active"] == CUSTOM_ID
        assert sorted(l["id"] for l in cli_after["layouts"]) == sorted(
            layout["id"] for layout in after["layouts"]
        )

        print("PASS: shared canonical store - CLI register/set-active visible to MCP and vice versa")


# ---------------------------------------------------------------------------
# 4. Store path semantics parity and failure modes
# ---------------------------------------------------------------------------

def test_store_path_semantics_and_failure_modes():
    # Missing store file: both surfaces fall back to the seeded presets rather than erroring.
    with tempfile.TemporaryDirectory() as td:
        absent = str(Path(td) / "does_not_exist.json")
        res = call_mcp_tool("aios.fs_layout.list", {"store_path": absent})
        assert res.get("ok") is True, f"missing store should fall back, got: {res}"
        assert res["count"] == 2, f"expected seeded presets, got {res['count']}"

        cli_res = cli_json("layout", "list", "--store", absent, "--json", context="cli missing store")
        assert cli_res["code"] == 0
        assert len(cli_res["data"]["layouts"]) == 2

        # Corrupt store: both surfaces must surface an explicit error, never silent success.
        corrupt = Path(td) / "corrupt.json"
        corrupt.write_text("NOT A JSON STORE", encoding="utf-8")
        res_bad = call_mcp_tool("aios.fs_layout.list", {"store_path": str(corrupt)})
        assert res_bad.get("ok") is False, f"corrupt store must fail, got: {res_bad}"
        cli_bad = run_cli("layout", "list", "--store", str(corrupt), "--json", expect=1,
                          context="cli corrupt store")
        assert json.loads(cli_bad.stdout)["code"] == 1

    # Oversized and control-character store paths are rejected (parity with CLI bounds).
    res_long = call_mcp_tool("aios.fs_layout.list", {"store_path": "a" * 1025})
    assert res_long.get("ok") is False, f"oversized store_path should fail: {res_long}"
    assert "1024" in err_text(res_long), f"expected 1024 error: {res_long}"

    res_ctrl = call_mcp_tool("aios.fs_layout.list", {"store_path": "bad\x01path"})
    assert res_ctrl.get("ok") is False, f"control-char store_path should fail: {res_ctrl}"
    assert "control characters" in err_text(res_ctrl), f"expected control char error: {res_ctrl}"

    # Read-only surfaces operate without a PEP grant (they are non-mutating).
    ungated = call_mcp_tool("aios.fs_layout.list")
    assert ungated.get("ok") is True, f"read-only tool should not require a grant: {ungated}"

    print("PASS: store_path semantics parity (fallback, corrupt, oversized, control chars)")


# ---------------------------------------------------------------------------
# 5. Mutation gating: granted vs ungranted, and store_path is mandatory
# ---------------------------------------------------------------------------

def test_mutation_gating_granted_versus_ungranted():
    with tempfile.TemporaryDirectory() as td:
        store = str(Path(td) / "gated.json")

        ungranted_calls = [
            ("aios.fs_layout.register",
             {"spec": json.dumps(base_layout_json()), "store_path": store}),
            ("aios.fs_layout.set_active", {"layout_id": UEFI_ID, "store_path": store}),
            ("aios.fs_layout.remove", {"layout_id": CONTAINER_ID, "store_path": store}),
            ("aios.fs_layout.import_fstab",
             {"layout_id": "x", "name": "x", "fstab": SMALL_FSTAB, "store_path": store}),
        ]
        for tool, args in ungranted_calls:
            res = call_mcp_tool(tool, args)
            assert res.get("ok") is False, f"{tool}: ungranted call must fail: {res}"
            assert res.get("gate") == "pep", f"{tool}: expected PEP gate refusal: {res}"
            assert res.get("audit_id"), f"{tool}: refusal must be audited: {res}"

        # An ungranted refusal must not create or touch the store.
        assert not Path(store).exists(), "ungranted refusals must not create the store"

        # Read tools stay ungated while mutations are gated: same process, same args shape.
        res_read = call_mcp_tool("aios.fs_layout.probe", {"store_path": store})
        assert res_read.get("ok") is True, f"read tool must not need a grant: {res_read}"

        grant = create_pep_grant("aios.fs_layout.*")
        assert grant, "failed to mint a PEP grant for aios.fs_layout.*"

        # store_path is required on every mutation even with a valid grant.
        for tool, args in [
            ("aios.fs_layout.register", {"spec": json.dumps(base_layout_json())}),
            ("aios.fs_layout.set_active", {"layout_id": UEFI_ID}),
            ("aios.fs_layout.remove", {"layout_id": CONTAINER_ID}),
            ("aios.fs_layout.import_fstab",
             {"layout_id": "x", "name": "x", "fstab": SMALL_FSTAB}),
        ]:
            args = dict(args, grant_id=grant)
            res = call_mcp_tool(tool, args)
            assert res.get("ok") is False, f"{tool}: missing store_path must fail: {res}"
            assert "store_path is required" in err_text(res), f"{tool}: {res}"

        # A grant is scoped: a grant for the session surface does not authorize fs_layout.
        other_grant = create_pep_grant("aios.session.*")
        if other_grant:
            res_wrong_scope = call_mcp_tool(
                "aios.fs_layout.remove",
                {"layout_id": CONTAINER_ID, "store_path": store, "grant_id": other_grant},
            )
            assert res_wrong_scope.get("ok") is False, f"out-of-scope grant must be refused: {res_wrong_scope}"

        print("PASS: mutations require BOTH a PEP grant and an explicit store_path; reads stay ungated")


# ---------------------------------------------------------------------------
# 6. Granted mutation round-trip with persistence re-read from disk
# ---------------------------------------------------------------------------

def test_mutation_roundtrip_over_stdio():
    with tempfile.TemporaryDirectory() as td:
        store = str(Path(td) / "roundtrip.json")
        reg_id = "mcp-reg-v1"
        import_id = "mcp-import-v1"
        custom = custom_layout(base_layout_json(), reg_id, "MCP Registered Layout")
        spec_file = Path(td) / "custom.json"
        spec_file.write_text(json.dumps(custom), encoding="utf-8")

        grant = create_pep_grant("aios.fs_layout.*")
        assert grant, "failed to mint a PEP grant for aios.fs_layout.*"

        # register (via a regular-file spec) -> persisted store holds it.
        res = call_mcp_tool("aios.fs_layout.register",
                            {"spec": str(spec_file), "store_path": store, "grant_id": grant})
        assert res.get("ok") is True and res.get("registered") is True, f"register failed: {res}"
        assert res.get("id") == reg_id, f"register id mismatch: {res}"
        assert isinstance(res.get("audit_id"), int), f"register must be audited: {res}"
        disk = store_on_disk(store)
        assert reg_id in disk["layouts"], f"layout not persisted: {sorted(disk['layouts'])}"
        assert disk["active_layout_id"] == UEFI_ID, "register must not hijack the active pointer"

        # get by id returns the exact stored body.
        res_get = call_mcp_tool("aios.fs_layout.get", {"layout_id": reg_id, "store_path": store})
        assert res_get.get("ok") is True, f"get by id failed: {res_get}"
        assert res_get["layout"] == custom, f"stored body differs: {res_get['layout']}"
        res_ghost = call_mcp_tool("aios.fs_layout.get", {"layout_id": "ghost", "store_path": store})
        assert res_ghost.get("ok") is False and "not found in store" in err_text(res_ghost)

        # list reflects the new layout without losing the presets.
        res_list = call_mcp_tool("aios.fs_layout.list", {"store_path": store})
        assert res_list["count"] == 3 and res_list["active_layout_id"] == UEFI_ID

        # set_active reports the transition and its destructive verdict (spec D-7).
        res_active = call_mcp_tool("aios.fs_layout.set_active",
                                   {"layout_id": reg_id, "store_path": store, "grant_id": grant})
        assert res_active.get("ok") is True, f"set_active failed: {res_active}"
        assert res_active["previous_active"] == UEFI_ID and res_active["active"] == reg_id
        assert res_active["destructive_transition"] is False, f"verdict: {res_active}"
        assert store_on_disk(store)["active_layout_id"] == reg_id

        # probe without layout_id now evaluates the ACTIVE layout (spec D-5).
        res_probe = call_mcp_tool("aios.fs_layout.probe",
                                  {"store_path": store, "target_disk_bytes": 128 * 1024 * 1024 * 1024})
        assert res_probe.get("ok") is True, f"probe default failed: {res_probe}"
        assert res_probe["evaluation"]["layout_id"] == reg_id, (
            f"probe defaulted to {res_probe['evaluation']['layout_id']}, expected {reg_id}"
        )

        # import_fstab inherits from the active layout and persists one atomic write.
        res_import = call_mcp_tool("aios.fs_layout.import_fstab",
                                   {"layout_id": import_id, "name": "MCP Imported",
                                    "fstab": SMALL_FSTAB, "store_path": store, "grant_id": grant})
        assert res_import.get("ok") is True, f"import_fstab failed: {res_import}"
        assert res_import["mounts"] == 1 and res_import["id"] == import_id
        assert import_id in store_on_disk(store)["layouts"]

        # Semantic refusals leave the store byte-identical on disk.
        before = Path(store).read_bytes()
        res_dup = call_mcp_tool("aios.fs_layout.register",
                                {"spec": str(spec_file), "store_path": store, "grant_id": grant})
        assert res_dup.get("ok") is False and "already registered" in err_text(res_dup), res_dup
        res_rm_active = call_mcp_tool("aios.fs_layout.remove",
                                      {"layout_id": reg_id, "store_path": store, "grant_id": grant})
        assert "cannot remove active layout" in err_text(res_rm_active), res_rm_active
        res_rm_builtin = call_mcp_tool("aios.fs_layout.remove",
                                       {"layout_id": UEFI_ID, "store_path": store, "grant_id": grant})
        assert "built-in canonical layout" in err_text(res_rm_builtin), res_rm_builtin
        res_unknown = call_mcp_tool("aios.fs_layout.set_active",
                                    {"layout_id": "ghost", "store_path": store, "grant_id": grant})
        assert "not found in store" in err_text(res_unknown), res_unknown
        res_no_rows = call_mcp_tool("aios.fs_layout.import_fstab",
                                    {"layout_id": "empty-v1", "name": "Empty", "fstab": "# only a comment\n",
                                     "store_path": store, "grant_id": grant})
        assert res_no_rows.get("ok") is False and "no valid mount entries" in err_text(res_no_rows)
        assert Path(store).read_bytes() == before, "refusals must not rewrite the store"

        # remove succeeds for a non-active custom layout; the CLI then observes the removal.
        res_rm = call_mcp_tool("aios.fs_layout.remove",
                               {"layout_id": import_id, "store_path": store, "grant_id": grant})
        assert res_rm.get("ok") is True and res_rm["removed"] is True, f"remove failed: {res_rm}"
        assert import_id not in store_on_disk(store)["layouts"]
        cli_after = cli_json("layout", "list", "--store", store, "--json",
                             context="cli list after MCP mutation")["data"]
        assert cli_after["active"] == reg_id, f"CLI did not observe MCP active switch: {cli_after}"

        print("PASS: granted register/get/list/set_active/probe/import_fstab/remove round-trip persisted")


# ---------------------------------------------------------------------------
# 7. CLI <-> MCP mutation parity
# ---------------------------------------------------------------------------

def test_cli_mcp_mutation_parity():
    with tempfile.TemporaryDirectory() as td:
        cli_store = str(Path(td) / "cli.json")
        mcp_store = str(Path(td) / "mcp.json")
        custom = custom_layout(base_layout_json(), CUSTOM_ID, "Parity Layout")
        spec_file = Path(td) / "custom.json"
        spec_file.write_text(json.dumps(custom), encoding="utf-8")
        grant = create_pep_grant("aios.fs_layout.*")
        assert grant, "failed to mint a PEP grant for aios.fs_layout.*"

        # The operator registers through the CLI; the agent registers the SAME spec through MCP.
        run_cli("layout", "register", "--spec", str(spec_file), "--store", cli_store, "--json",
                context="cli register")
        res = call_mcp_tool("aios.fs_layout.register",
                            {"spec": str(spec_file), "store_path": mcp_store, "grant_id": grant})
        assert res.get("ok") is True, f"MCP register failed: {res}"

        # Resulting layouts are identical field-for-field across the two surfaces.
        cli_spec = cli_json("layout", "show", CUSTOM_ID, "--store", cli_store, "--json",
                            context="cli show")["data"]
        mcp_spec = call_mcp_tool("aios.fs_layout.get",
                                 {"layout_id": CUSTOM_ID, "store_path": mcp_store})["layout"]
        assert cli_spec == mcp_spec == custom, f"CLI/MCP layouts diverged:\n{cli_spec}\n{mcp_spec}"

        # CLI set-active -> MCP observes it, and the MCP probe default follows it.
        run_cli("layout", "set-active", CUSTOM_ID, "--store", cli_store, "--json",
                context="cli set-active")
        after = call_mcp_tool("aios.fs_layout.list", {"store_path": cli_store})
        assert after["active_layout_id"] == CUSTOM_ID
        probe = call_mcp_tool("aios.fs_layout.probe",
                              {"store_path": cli_store, "target_disk_bytes": 128 * 1024 * 1024 * 1024})
        assert probe["evaluation"]["layout_id"] == CUSTOM_ID, (
            f"MCP probe default did not follow the CLI active layout: {probe['evaluation']}"
        )

        # MCP set-active -> CLI observes it (both directions of the pointer switch).
        res_active = call_mcp_tool("aios.fs_layout.set_active",
                                   {"layout_id": CUSTOM_ID, "store_path": mcp_store, "grant_id": grant})
        assert res_active["previous_active"] == UEFI_ID and res_active["active"] == CUSTOM_ID
        cli_after = cli_json("layout", "list", "--store", mcp_store, "--json",
                             context="cli list")["data"]
        assert cli_after["active"] == CUSTOM_ID, f"CLI did not observe MCP set-active: {cli_after}"

        res_back = call_mcp_tool("aios.fs_layout.set_active",
                                 {"layout_id": UEFI_ID, "store_path": mcp_store, "grant_id": grant})
        assert res_back["previous_active"] == CUSTOM_ID and res_back["active"] == UEFI_ID
        cli_back = cli_json("layout", "list", "--store", mcp_store, "--json",
                           context="cli list")["data"]
        assert cli_back["active"] == UEFI_ID, f"CLI did not observe MCP re-activation: {cli_back}"

        # fstab text stays byte-identical for the same layout across surfaces.
        cli_fstab = cli_json("layout", "fstab", CUSTOM_ID, "--store", cli_store, "--json",
                             context="cli fstab")["data"]["fstab"]
        mcp_fstab = call_mcp_tool("aios.fs_layout.fstab", {"spec": str(spec_file)})["fstab"]
        assert cli_fstab == mcp_fstab, "fstab text diverged between CLI store and MCP spec read"

        print("PASS: CLI <-> MCP mutation parity (identical layouts, mutual pointer visibility)")


# ---------------------------------------------------------------------------
# 8. Read hardening: type-checked, bounded, non-blocking spec/fstab reads (F-1)
# ---------------------------------------------------------------------------

def test_spec_read_hardening():
    with tempfile.TemporaryDirectory() as td:
        d = Path(td)
        store = str(d / "hardening.json")
        grant = create_pep_grant("aios.fs_layout.*")
        assert grant, "failed to mint a PEP grant for aios.fs_layout.*"

        # A directory named by spec/fstab is refused BY TYPE, never read (the F-1 defect class).
        for tool, args in [
            ("aios.fs_layout.validate", {"spec": str(d)}),
            ("aios.fs_layout.fstab", {"spec": str(d)}),
            ("aios.fs_layout.register", {"spec": str(d), "store_path": store, "grant_id": grant}),
            ("aios.fs_layout.import_fstab",
             {"layout_id": "dir-v1", "name": "Dir", "fstab": str(d),
              "store_path": store, "grant_id": grant}),
        ]:
            started = time.monotonic()
            res = call_mcp_tool(tool, args, timeout_s=15)
            elapsed = time.monotonic() - started
            assert res.get("ok") is False, f"{tool}: directory input must be refused: {res}"
            assert "is a directory" in err_text(res), f"{tool}: expected type refusal: {res}"
            assert elapsed < 10, f"{tool}: refusal took {elapsed:.1f}s (read was not bounded)"
        assert not Path(store).exists(), "a refused read must not write the store"

        # POSIX only: a FIFO named by spec must not stall the single-threaded request loop.
        if hasattr(os, "mkfifo"):
            fifo = d / "hang.fifo"
            os.mkfifo(fifo)
            started = time.monotonic()
            res = call_mcp_tool("aios.fs_layout.validate", {"spec": str(fifo)}, timeout_s=15)
            elapsed = time.monotonic() - started
            assert res.get("ok") is False, f"FIFO must be refused: {res}"
            assert "not a regular file" in err_text(res), f"expected FIFO type refusal: {res}"
            assert elapsed < 5, f"FIFO read was not bounded ({elapsed:.1f}s) — server stall regressed"
            print(f"PASS: FIFO named by spec refused by type in {elapsed:.2f}s (no server stall)")
        else:
            print("SKIP: FIFO stall case needs POSIX mkfifo (Windows host) — covered on POSIX CI")

        # The transport line cap still bounds an oversized request before parsing.
        oversize = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": "aios.fs_layout.validate",
                       "arguments": {"spec": "a" * (1024 * 1024 + 64)}},
        }
        resp = run_mcp(oversize)
        assert "error" in resp, f"oversized request must be refused by the transport: {resp}"
        assert resp["error"]["code"] == -32700, f"expected -32700, got {resp['error']}"

        print("PASS: spec/fstab reads are type-checked, bounded and non-blocking (F-1 closed)")


def test_manifest_contract_and_verdict_parity():
    tools = mcp_tools()

    # (a) The integrated surface advertises the arguments its arms accept. `layout_id`
    #     on `get` and `store_path` on `validate` were both missing before T-01535, so
    #     the widened capabilities were undiscoverable and strict clients rejected them.
    expected = {
        "aios.fs_layout.get": ["layout_id", "profile", "store_path", "grant_id"],
        "aios.fs_layout.validate": ["spec", "layout", "store_path", "grant_id"],
        "aios.fs_layout.fstab": ["profile", "spec", "grant_id"],
        "aios.fs_layout.list": ["store_path", "grant_id"],
        "aios.fs_layout.probe": ["layout_id", "target_disk_bytes", "store_path", "grant_id"],
        "aios.fs_layout.diff": ["source_id", "target_id", "store_path", "grant_id"],
        "aios.fs_layout.register": ["layout", "spec", "store_path", "grant_id"],
        "aios.fs_layout.set_active": ["layout_id", "store_path", "grant_id"],
        "aios.fs_layout.remove": ["layout_id", "store_path", "grant_id"],
        "aios.fs_layout.import_fstab": ["layout_id", "name", "fstab", "base_layout_id",
                                       "store_path", "grant_id"],
    }
    for name, accepted in expected.items():
        schema = tools[name]["inputSchema"]
        assert sorted(schema["properties"]) == sorted(accepted), (
            f"{name}: advertises {sorted(schema['properties'])} but accepts {sorted(accepted)}"
        )
        assert schema.get("additionalProperties") is False, f"{name}: additionalProperties"

    with tempfile.TemporaryDirectory() as td:
        cli_store = str(Path(td) / "cli.json")
        mcp_store = str(Path(td) / "mcp.json")
        grant = create_pep_grant("aios.fs_layout.*")
        assert grant, "failed to mint a PEP grant for aios.fs_layout.*"

        # (b) Both surfaces record the same audit target for the same operation: the
        #     layout id, per spec §9 (the MCP spec-path form used to log None).
        spec = custom_layout(base_layout_json(), "parity-target-v1", "Parity Target")
        spec_file = Path(td) / "parity.json"
        spec_file.write_text(json.dumps(spec), encoding="utf-8")

        cli_row_id_before = max([r["id"] for r in cli_audit_rows()] or [0])
        run_cli("layout", "register", "--spec", str(spec_file), "--store", cli_store, "--json",
                context="cli register")
        cli_register_rows = [r for r in cli_audit_rows()
                             if r.get("tool") == "fs_layout" and r.get("id", 0) > cli_row_id_before]
        assert cli_register_rows, "CLI register wrote no audit row"
        assert cli_register_rows[-1].get("target") == "parity-target-v1", (
            f"CLI register audit target: {cli_register_rows[-1].get('target')!r}"
        )

        res = call_mcp_tool("aios.fs_layout.register",
                            {"spec": str(spec_file), "store_path": mcp_store, "grant_id": grant})
        assert res.get("ok") is True, f"MCP register failed: {res}"
        mcp_row = next((r for r in cli_audit_rows() if r["id"] == res.get("audit_id")), None)
        assert mcp_row is not None, f"MCP audit row {res.get('audit_id')} not found"
        assert mcp_row.get("target") == "parity-target-v1", (
            f"MCP spec-path register audit target: {mcp_row.get('target')!r}"
        )

        # (c) One transition, one verdict: MCP set_active and CLI diff must agree that
        #     shrinking every partition is destructive.
        shrink = custom_layout(base_layout_json(), "parity-shrink-v1", "Parity Shrink")
        for part in shrink["partitions"]:
            part["size_mib"] = max(1, part["size_mib"] // 4)
        reg = call_mcp_tool("aios.fs_layout.register",
                            {"layout": shrink, "store_path": mcp_store, "grant_id": grant})
        assert reg.get("ok") is True, f"register shrink failed: {reg}"
        res_shrink = call_mcp_tool("aios.fs_layout.set_active",
                                   {"layout_id": "parity-shrink-v1",
                                    "store_path": mcp_store, "grant_id": grant})
        assert res_shrink.get("destructive_transition") is True, (
            f"MCP set_active must report the shrink as destructive: {res_shrink}"
        )
        cli_diff = cli_json("layout", "diff", UEFI_ID, "parity-shrink-v1",
                            "--store", mcp_store, "--json", context="cli diff")["data"]
        assert cli_diff["destructive"] is True, f"CLI diff disagrees with MCP: {cli_diff}"
        res_grow = call_mcp_tool("aios.fs_layout.set_active",
                                 {"layout_id": UEFI_ID, "store_path": mcp_store, "grant_id": grant})
        assert res_grow.get("destructive_transition") is False, (
            f"growing back is not destructive: {res_grow}"
        )

        print("PASS: advertised schema contract + audit-target and destructive-verdict parity")


def main():
    print("=== RUNNING FILESYSTEM LAYOUT CROSS-SURFACE INTEGRATION SMOKE TESTS ===")
    test_manifest_and_store_path_discoverability()
    test_cli_surface_discoverability_and_alias_parity()
    test_cross_surface_parity_on_presets()
    test_cross_surface_store_sharing()
    test_store_path_semantics_and_failure_modes()
    test_mutation_gating_granted_versus_ungranted()
    test_mutation_roundtrip_over_stdio()
    test_cli_mcp_mutation_parity()
    test_spec_read_hardening()
    test_manifest_contract_and_verdict_parity()
    print("\nALL FILESYSTEM LAYOUT CROSS-SURFACE INTEGRATION TESTS PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
