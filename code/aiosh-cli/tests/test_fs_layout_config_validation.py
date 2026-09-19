#!/usr/bin/env python3
"""Configuration Validation Unit Test for Filesystem Layout (T-01545 / epic T-01542..T-01551).

Wire-level unit tests for the Filesystem Layout *configuration contract*
(T-01542 spec §5 V-1..V-7, error matrix §4 E-1..E-7), exercised through the
real `aiosh` binary — the same surface an operator drives. The library-level
wording is pinned in-tree by `aiosh-core/tests/test_fs_layout_data_model.rs`
(T-01543) and the store-parse change by `test_fs_layout_service.rs` (T-01544);
this file pins the *observable* behaviour: exit codes, JSON envelope codes,
exact refusal wording, ordering of the first reported error, and the
fail-closed store-integrity guarantee on every refusal path.

Coverage (each case = valid input, invalid input, boundary values, and the
primary failure mode where one exists):
  U1  valid documents         both built-ins; spec-file form; compat payloads
                              (custom enum variants) and boundary values
  U2  E-1 unknown fields      top-level, nested, unknown enum variant
  U3  E-2 directory mode      0 and > 0o7777 refused; 0o7777 legal boundary
  U4  E-3 FL4 noexec          /tmp and /dev/shm; exact-token boundary
  U5  E-4 symlink_target      UsrMerge shape (absolute / dot / .. / non-usr)
  U6  E-5 created_at          RFC 3339 UTC only (offset form refused)
  U7  E-6 dump bound          dump in {0,1}; 2 refused; 1 legal boundary
  U8  E-7 FL6                 at least one required mount; ordering rule
  U9  spec-file form          the contract holds through the second input path
  U10 register-path refusal  SPEC_PARSE_FAILED + fail-closed store (md5, ids,
                              no staged residue)
  U11 store document         the T-01544 refusal: unknown top-level and nested
                              fields in the store file are refused everywhere,
                              nothing is mutated, recovery is external
  U12 human (non-JSON) path  INVALID:/load-failure lines on stderr, exit 1

Assertions target observable behaviour (process exit status, envelope, store
bytes), never implementation details.

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_config_validation.py
"""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

UEFI_ID = "aios-uefi-standard-v1"
CONTAINER_ID = "aios-container-minimal-v1"


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
    return subprocess.run([get_binary_path(), *args], capture_output=True, text=True, timeout=60)


def parse_json_output(res: subprocess.CompletedProcess):
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)


def expect_code(res: subprocess.CompletedProcess, code: int, context: str) -> None:
    assert res.returncode == code, (
        f"{context}: expected exit code {code}, got {res.returncode}\n"
        f"stdout: {res.stdout.strip()}\nstderr: {res.stderr.strip()}"
    )


def json_envelope(res: subprocess.CompletedProcess, expected_code: int, context: str) -> dict:
    """Assert a canonical {code,data,error} envelope and return it."""
    payload = parse_json_output(res)
    assert isinstance(payload, dict), f"{context}: expected a JSON object envelope"
    assert payload.get("code") == expected_code, (
        f"{context}: envelope code {payload.get('code')} != {expected_code}"
    )
    assert "data" in payload and "error" in payload, f"{context}: envelope missing data/error keys"
    return payload


def base_layout(spec_id: str = UEFI_ID) -> dict:
    """Fetch a schema-valid layout from the binary itself (avoids hand-written fixtures)."""
    res = run_aiosh("layout", "show", spec_id, "--json")
    expect_code(res, 0, f"show {spec_id}")
    return json_envelope(res, 0, f"show {spec_id}")["data"]


def spec_error(spec: dict, context: str, *, expect_rc: int = 1) -> tuple[dict, str]:
    """Run `layout validate --spec <inline>` on a mutated spec; return (envelope, message)."""
    res = run_aiosh("layout", "validate", "--spec", json.dumps(spec), "--json")
    env = json_envelope(res, expect_rc, context)
    assert env["data"]["valid"] is False, f"{context}: data.valid must be false on a refusal"
    assert env["data"]["id"] == "unknown", f"{context}: parse-stage refusals cannot name an id"
    assert env["error"]["code"] == "VALIDATION_FAILED", f"{context}: unexpected error code"
    return env, env["error"]["message"]


def _strip_noexec(spec: dict, path: str) -> None:
    for m in spec["mounts"]:
        if m["path"] == path:
            m["options"] = [o for o in m["options"] if o != "noexec"]


def _unrequire_all(spec: dict) -> None:
    for m in spec["mounts"]:
        m["required"] = False


def _md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def _staged_residue(directory: Path, store_name: str) -> list[str]:
    return [f.name for f in directory.iterdir() if f.name.startswith(f".{store_name}.tmp.")]


# ---------------------------------------------------------------------------
# U1 — valid documents (happy path + compat + boundary values)
# ---------------------------------------------------------------------------

def test_u1_valid_documents_and_boundary_values() -> None:
    for layout_id in (UEFI_ID, CONTAINER_ID):
        spec = base_layout(layout_id)
        env, = (json_envelope(run_aiosh("layout", "validate", "--spec", json.dumps(spec), "--json"), 0, f"validate {layout_id}"),)
        assert env["data"]["valid"] is True
        assert env["data"]["id"] == layout_id
        assert env["error"] is None

    # Boundary values the contract explicitly keeps legal: the top of the
    # 12-bit mode range, dump=1, and the UTC 'Z' designator.
    spec = base_layout()
    spec["directories"][0]["mode"] = 0o7777
    spec["mounts"][1]["dump"] = 1
    spec["created_at"] = "2026-09-16T00:00:00Z"
    json_envelope(run_aiosh("layout", "validate", "--spec", json.dumps(spec), "--json"), 0, "boundary values")

    # Compatibility (spec §9): documented custom enum payloads stay legal.
    custom = base_layout()
    custom["partitions"][1]["format_as"] = {"custom": "zfs"}
    json_envelope(run_aiosh("layout", "validate", "--spec", json.dumps(custom), "--json"), 0, "custom fs payload")

    print("PASS: U1 valid documents, boundary values, custom enum payloads")


# ---------------------------------------------------------------------------
# U2 — E-1 unknown fields (schema rejection names the first offender)
# ---------------------------------------------------------------------------

def test_u2_unknown_fields_refused_at_every_nesting_level() -> None:
    spec = base_layout()

    _, msg = spec_error({**spec, "dry_run": True}, "E-1 top-level")
    assert "failed to deserialize layout JSON" in msg, f"E-1 prefix, got: {msg}"
    assert "unknown field `dry_run`" in msg, f"E-1 names the offender, got: {msg}"

    nested = copy.deepcopy(spec)
    nested["mounts"][0]["no_such_mount_field"] = 1
    _, msg = spec_error(nested, "E-1 nested")
    assert "unknown field `no_such_mount_field`" in msg, f"E-1 nested offender, got: {msg}"

    enum = copy.deepcopy(spec)
    enum["partitions"][0]["format_as"] = "bogusfs"
    _, msg = spec_error(enum, "E-1 enum variant")
    assert "unknown variant `bogusfs`" in msg, f"E-1 enum refusal, got: {msg}"

    print("PASS: U2 unknown fields refused top-level, nested, and as enum variant")


# ---------------------------------------------------------------------------
# U3 — E-2 directory mode range
# ---------------------------------------------------------------------------

def test_u3_directory_mode_range_enforced() -> None:
    spec = base_layout()

    low = copy.deepcopy(spec)
    low["directories"][0]["mode"] = 0
    _, msg = spec_error(low, "E-2 mode 0")
    assert "directory '/var/lib/aios' mode must be in 1..=0o7777 (octal), found 0" in msg, f"got: {msg}"

    high = copy.deepcopy(spec)
    high["directories"][0]["mode"] = 0o10000  # first value beyond the 12 permission bits
    _, msg = spec_error(high, "E-2 mode 4096")
    assert "found 4096" in msg, f"boundary value echoed, got: {msg}"

    print("PASS: U3 directory mode 0 and >0o7777 refused (0o7777 legal, pinned in U1)")


# ---------------------------------------------------------------------------
# U4 — E-3 FL4 noexec on /tmp and /dev/shm (exact option token)
# ---------------------------------------------------------------------------

def test_u4_fl4_noexec_required_and_exact_token() -> None:
    for path in ("/tmp", "/dev/shm"):
        spec = base_layout()
        _strip_noexec(spec, path)
        _, msg = spec_error(spec, f"E-3 {path}")
        assert f"FL4 violation: mount '{path}' missing mandatory security option 'noexec'" in msg, f"got: {msg}"

    # Token boundary: FL4 matches the exact option token, not a substring or
    # an '='-separated key=value spelling of it.
    for decoy in ("noexec2", "noexec=1"):
        spec = base_layout()
        _strip_noexec(spec, "/tmp")
        spec["mounts"][2]["options"].append(decoy)
        _, msg = spec_error(spec, f"FL4 token boundary {decoy}")
        assert "missing mandatory security option 'noexec'" in msg, f"{decoy} must not satisfy FL4: {msg}"

    print("PASS: U4 FL4 noexec on /tmp and /dev/shm, exact-token boundary")


# ---------------------------------------------------------------------------
# U5 — E-4 symlink_target UsrMerge shape
# ---------------------------------------------------------------------------

def test_u5_symlink_target_usrmerge_shape() -> None:
    spec = base_layout()
    assert spec["directories"][-1]["symlink_target"], "fixture expects a usrmerge entry"

    for bad in ("/usr/bin", "bin/usr", "usr/../etc", "usr/./bin"):
        bad_spec = copy.deepcopy(spec)
        bad_spec["directories"][-1]["symlink_target"] = bad
        _, msg = spec_error(bad_spec, f"E-4 symlink {bad!r}")
        assert "must be a relative path under 'usr' (UsrMerge)" in msg, f"{bad!r}: got: {msg}"

    good = copy.deepcopy(spec)
    good["directories"][-1]["symlink_target"] = "usr/bin"
    json_envelope(run_aiosh("layout", "validate", "--spec", json.dumps(good), "--json"), 0, "valid usrmerge")

    print("PASS: U5 symlink_target absolute/dot/../non-usr refused, usr/bin legal")


# ---------------------------------------------------------------------------
# U6 — E-5 created_at must be RFC 3339 UTC
# ---------------------------------------------------------------------------

def test_u6_created_at_rfc3339_utc_only() -> None:
    spec = base_layout()
    for bad in ("not-a-date", "2026-09-16T00:00:00", "2026-09-16T00:00:00+02:00"):
        bad_spec = copy.deepcopy(spec)
        bad_spec["created_at"] = bad
        _, msg = spec_error(bad_spec, f"E-5 created_at {bad!r}")
        assert "layout 'created_at' must be an RFC 3339 UTC timestamp" in msg, f"{bad!r}: got: {msg}"
        assert bad in msg, f"the offending value is echoed verbatim: {msg}"

    print("PASS: U6 created_at refuses junk, naive, and offset forms (Z pinned in U1)")


# ---------------------------------------------------------------------------
# U7 — E-6 dump bound
# ---------------------------------------------------------------------------

def test_u7_dump_bounded_to_zero_or_one() -> None:
    spec = base_layout()
    bad = copy.deepcopy(spec)
    bad["mounts"][1]["dump"] = 2
    _, msg = spec_error(bad, "E-6 dump 2")
    assert "mount '/boot/efi' dump must be 0 or 1, found 2" in msg, f"got: {msg}"

    print("PASS: U7 dump=2 refused (dump=1 legal, pinned in U1)")


# ---------------------------------------------------------------------------
# U8 — E-7 FL6 + the spec §4 ordering rule
# ---------------------------------------------------------------------------

def test_u8_fl6_required_mount_and_error_ordering() -> None:
    spec = base_layout()
    _unrequire_all(spec)
    _, msg = spec_error(spec, "E-7 FL6")
    assert "FL6 violation: at least one mount must be marked required" in msg, f"got: {msg}"

    # One required mount is enough (floor semantics, not readiness proof).
    one = copy.deepcopy(spec)
    _unrequire_all(one)
    one["mounts"][0]["required"] = True
    json_envelope(run_aiosh("layout", "validate", "--spec", json.dumps(one), "--json"), 0, "one required mount")

    # Ordering (T-01542 §4): top-level shape (E-5) precedes FL6; FL6 precedes
    # the per-element directory checks (E-2). First-error-wins.
    e5_first = copy.deepcopy(spec)
    _unrequire_all(e5_first)
    e5_first["created_at"] = "not-a-date"
    _, msg = spec_error(e5_first, "ordering E-5 > FL6")
    assert "RFC 3339 UTC" in msg and "FL6" not in msg, f"E-5 must win: {msg}"

    fl6_first = copy.deepcopy(spec)
    _unrequire_all(fl6_first)
    fl6_first["directories"][0]["mode"] = 0
    _, msg = spec_error(fl6_first, "ordering FL6 > E-2")
    assert "FL6 violation" in msg and "mode must be" not in msg, f"FL6 must win: {msg}"

    print("PASS: U8 FL6 floor + first-error ordering (E-5 > FL6 > E-2)")


# ---------------------------------------------------------------------------
# U9 — the contract holds through the spec-file input form
# ---------------------------------------------------------------------------

def test_u9_spec_file_form_matches_inline_form() -> None:
    with tempfile.TemporaryDirectory() as td:
        spec_file = Path(td) / "spec.json"
        spec = base_layout()
        spec["created_at"] = "not-a-date"
        spec_file.write_text(json.dumps(spec), encoding="utf-8")

        res = run_aiosh("layout", "validate", "--spec", str(spec_file), "--json")
        env = json_envelope(res, 1, "spec-file E-5")
        assert env["data"]["valid"] is False
        assert "layout 'created_at' must be an RFC 3339 UTC timestamp" in env["error"]["message"], (
            f"file form must carry the same contract wording, got: {env['error']['message']}"
        )

    print("PASS: U9 spec-file form carries the identical contract wording")


# ---------------------------------------------------------------------------
# U10 — register-path refusals are fail-closed
# ---------------------------------------------------------------------------

def test_u10_register_refusals_are_fail_closed() -> None:
    with tempfile.TemporaryDirectory() as td:
        store = Path(td) / "fs_layouts.json"
        store_arg = str(store)
        spec = base_layout()

        # Warm the store through a mutation verb (read-only verbs never persist).
        json_envelope(
            run_aiosh("layout", "set-active", UEFI_ID, "--store", store_arg, "--json"),
            0, "warm store",
        )
        md5_before = _md5(store)

        e2 = copy.deepcopy(spec)
        e2["directories"][0]["mode"] = 0
        e2["id"] = "t01545-e2"
        res = run_aiosh("layout", "register", "--spec", json.dumps(e2), "--store", store_arg, "--json")
        env = json_envelope(res, 1, "register E-2")
        assert env["error"]["code"] == "SPEC_PARSE_FAILED", f"register-path code, got: {env['error']['code']}"
        assert "mode must be in 1..=0o7777 (octal), found 0" in env["error"]["message"]

        e1 = copy.deepcopy(spec)
        e1["id"] = "t01545-e1"
        e1["dry_run"] = True
        res = run_aiosh("layout", "register", "--spec", json.dumps(e1), "--store", store_arg, "--json")
        env = json_envelope(res, 1, "register E-1")
        assert env["error"]["code"] == "SPEC_PARSE_FAILED"
        assert "unknown field `dry_run`" in env["error"]["message"]

        # Fail-closed: the store is byte-identical, the refused ids never
        # appeared, and no staged file was created.
        assert _md5(store) == md5_before, "a refused registration must not touch the store"
        persisted = json.loads(store.read_text(encoding="utf-8"))
        for refused in ("t01545-e1", "t01545-e2"):
            assert refused not in persisted["layouts"], f"{refused} leaked into the store"
        assert not _staged_residue(Path(td), "fs_layouts.json"), "a refusal must not stage a file"

    print("PASS: U10 register-path refusals (SPEC_PARSE_FAILED, byte-identical store, no residue)")


# ---------------------------------------------------------------------------
# U11 — the store document itself is parsed under the same contract (T-01544)
# ---------------------------------------------------------------------------

def test_u11_store_document_unknown_fields_refused_everywhere() -> None:
    with tempfile.TemporaryDirectory() as td:
        store = Path(td) / "fs_layouts.json"
        store_arg = str(store)
        spec = base_layout()

        json_envelope(
            run_aiosh("layout", "set-active", UEFI_ID, "--store", store_arg, "--json"),
            0, "warm store",
        )
        fresh = json.loads(store.read_text(encoding="utf-8"))
        assert sorted(fresh.keys()) == ["active_layout_id", "layouts"], "fresh store schema"

        # Unknown top-level field: the store that means something other than
        # what it says must be refused by every verb that loads it.
        bad_top = copy.deepcopy(fresh)
        bad_top["active_layout"] = CONTAINER_ID
        store.write_text(json.dumps(bad_top), encoding="utf-8")
        md5_bad = _md5(store)

        res = run_aiosh("layout", "list", "--store", store_arg, "--json")
        env = json_envelope(res, 1, "store top-level refusal (list)")
        assert env["error"]["code"] == "LOAD_STORE_FAILED", f"got: {env['error']['code']}"
        msg = env["error"]["message"]
        assert "failed to deserialize layout store from" in msg, f"names the document, got: {msg}"
        assert "unknown field `active_layout`" in msg, f"names the offender, got: {msg}"
        assert CONTAINER_ID not in msg, "no value from a refused store may be echoed as loaded"

        # A mutation verb against the same store is refused too, and the store
        # is left byte-identical (fail-closed; recovery is external).
        res = run_aiosh("layout", "set-active", CONTAINER_ID, "--store", store_arg, "--json")
        env = json_envelope(res, 1, "store top-level refusal (set-active)")
        assert env["error"]["code"] == "LOAD_STORE_FAILED"
        assert _md5(store) == md5_bad, "a refused load must not rewrite the store"
        assert not _staged_residue(Path(td), "fs_layouts.json")

        # Unknown nested field inside a stored layout: same refusal.
        bad_nested = copy.deepcopy(fresh)
        bad_nested["layouts"][UEFI_ID]["dry_run"] = True
        store.write_text(json.dumps(bad_nested), encoding="utf-8")
        res = run_aiosh("layout", "set-active", UEFI_ID, "--store", store_arg, "--json")
        env = json_envelope(res, 1, "store nested refusal")
        assert env["error"]["code"] == "LOAD_STORE_FAILED"
        assert "unknown field `dry_run`" in env["error"]["message"], f"got: {env['error']['message']}"
        res = run_aiosh("layout", "register", "--spec", json.dumps(spec), "--store", store_arg, "--json")
        env = json_envelope(res, 1, "store nested refusal (register)")
        assert env["error"]["code"] == "LOAD_STORE_FAILED"
        assert not _staged_residue(Path(td), "fs_layouts.json")

        # Recovery is external exactly as documented: drop the unknown key and
        # the store loads again.
        store.write_text(json.dumps(fresh), encoding="utf-8")
        json_envelope(run_aiosh("layout", "list", "--store", store_arg, "--json"), 0, "recovered store")

    print("PASS: U11 store-document refusal (top+nested, fail-closed, external recovery)")


# ---------------------------------------------------------------------------
# U12 — the human (non-JSON) path reports the same refusals
# ---------------------------------------------------------------------------

def test_u12_human_path_reports_the_same_refusals() -> None:
    spec = base_layout()
    spec["directories"][0]["mode"] = 0
    res = run_aiosh("layout", "validate", "--spec", json.dumps(spec))
    expect_code(res, 1, "human validate refusal")
    assert res.stdout.strip() == "", "human refusal goes to stderr, not stdout"
    assert res.stderr.startswith("INVALID: "), f"human refusal prefix, got: {res.stderr!r}"
    assert "mode must be in 1..=0o7777 (octal), found 0" in res.stderr

    with tempfile.TemporaryDirectory() as td:
        store = Path(td) / "fs_layouts.json"
        store_arg = str(store)
        json_envelope(
            run_aiosh("layout", "set-active", UEFI_ID, "--store", store_arg, "--json"),
            0, "warm store",
        )
        fresh = json.loads(store.read_text(encoding="utf-8"))
        fresh["active_layout"] = "x"
        store.write_text(json.dumps(fresh), encoding="utf-8")
        res = run_aiosh("layout", "list", "--store", store_arg)
        expect_code(res, 1, "human store refusal")
        assert "unknown field `active_layout`" in res.stderr, f"got: {res.stderr!r}"

    print("PASS: U12 human path (INVALID:/load-failure on stderr, exit 1)")


def main() -> int:
    test_u1_valid_documents_and_boundary_values()
    test_u2_unknown_fields_refused_at_every_nesting_level()
    test_u3_directory_mode_range_enforced()
    test_u4_fl4_noexec_required_and_exact_token()
    test_u5_symlink_target_usrmerge_shape()
    test_u6_created_at_rfc3339_utc_only()
    test_u7_dump_bounded_to_zero_or_one()
    test_u8_fl6_required_mount_and_error_ordering()
    test_u9_spec_file_form_matches_inline_form()
    test_u10_register_refusals_are_fail_closed()
    test_u11_store_document_unknown_fields_refused_everywhere()
    test_u12_human_path_reports_the_same_refusals()
    print("\nALL FILESYSTEM LAYOUT CONFIG VALIDATION UNIT TESTS PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
