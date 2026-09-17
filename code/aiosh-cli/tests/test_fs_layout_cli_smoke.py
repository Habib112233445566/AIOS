#!/usr/bin/env python3
"""CLI Smoke & Boundary Test for Filesystem Layout (T-01525 / epic T-01521..T-01530).

Exercises the operator surface `aiosh layout <subcommand>` end-to-end through the real
binary: exit codes, JSON result envelopes, and on-disk store state.

Coverage: valid input, invalid input, boundary values, and the primary failure mode for
each subcommand. Assertions target observable behaviour (process exit status, stdout
envelope, persisted store JSON) rather than implementation details.

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_cli_smoke.py
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

# Canonical UEFI minimum target (64 GiB) and its total partition allocation (55808 MiB).
UEFI_MIN_BYTES = 64 * 1024 * 1024 * 1024
UEFI_PARTITION_BYTES = (512 + 51200 + 4096) * 1024 * 1024


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


# ---------------------------------------------------------------------------
# Dispatch surface
# ---------------------------------------------------------------------------

def test_layout_help_and_unknown_subcommand() -> None:
    res = run_aiosh("layout", "--help")
    expect_code(res, 0, "layout --help")
    for token in ("aiosh layout", "list", "show", "validate", "probe", "diff", "fstab",
                  "register", "set-active", "remove", "import-fstab"):
        assert token in res.stdout, f"layout --help missing token {token!r}"

    # Unknown subcommand -> usage error.
    res_unknown = run_aiosh("layout", "bogus_subcommand", "--json")
    expect_code(res_unknown, 2, "layout bogus_subcommand")
    assert json_envelope(res_unknown, 2, "unknown subcommand")["error"]["code"] == "UNKNOWN_SUBCOMMAND"

    print("PASS: aiosh layout --help and unknown subcommand (exit 2)")


# ---------------------------------------------------------------------------
# list / show
# ---------------------------------------------------------------------------

def test_layout_list() -> None:
    res = run_aiosh("layout", "list")
    expect_code(res, 0, "layout list")
    assert "Registered Filesystem Layouts" in res.stdout
    for layout_id in (UEFI_ID, CONTAINER_ID):
        assert layout_id in res.stdout, f"layout list prose missing {layout_id}"

    payload = json_envelope(run_aiosh("layout", "list", "--json"), 0, "layout list --json")
    data = payload["data"]
    ids = [layout["id"] for layout in data["layouts"]]
    assert UEFI_ID in ids and CONTAINER_ID in ids, f"presets missing from {ids}"
    assert data["active"] == UEFI_ID, f"expected default active {UEFI_ID}, got {data['active']}"

    print("PASS: aiosh layout list (prose and JSON, presets present)")


def test_layout_show_valid_and_invalid() -> None:
    # Active layout is the default target when no ID is supplied.
    payload = json_envelope(run_aiosh("layout", "show", "--json"), 0, "show (active)")
    assert payload["data"]["id"] == UEFI_ID

    # Explicit ID.
    payload = json_envelope(run_aiosh("layout", "show", CONTAINER_ID, "--json"), 0, "show container")
    assert payload["data"]["id"] == CONTAINER_ID

    # Explicit preset selectors.
    payload = json_envelope(run_aiosh("layout", "show", "--container", "--json"), 0, "show --container")
    assert payload["data"]["id"] == CONTAINER_ID

    payload = json_envelope(run_aiosh("layout", "show", "--standard", "--json"), 0, "show --standard")
    assert payload["data"]["id"] == UEFI_ID

    # Primary failure mode: unknown layout ID.
    res_missing = run_aiosh("layout", "show", "no-such-layout-xyz", "--json")
    expect_code(res_missing, 1, "show unknown id")
    env = json_envelope(res_missing, 1, "show unknown id")
    assert env["error"]["code"] == "RESOLVE_FAILED"
    assert env["data"] is None

    print("PASS: aiosh layout show (valid, preset selector, unknown id -> exit 1)")


# ---------------------------------------------------------------------------
# validate / check
# ---------------------------------------------------------------------------

def test_layout_validate_valid_invalid_and_boundary() -> None:
    # Valid: shipped preset satisfies FL1..FL5.
    env = json_envelope(run_aiosh("layout", "validate", "--json"), 0, "validate")
    assert env["data"]["valid"] is True

    # `check` is a documented alias of `validate`.
    res_check = run_aiosh("layout", "check", "--json")
    expect_code(res_check, 0, "layout check")
    assert json_envelope(res_check, 0, "check alias")["data"]["valid"] is True

    # Invalid input: well-formed JSON that violates FL1 (no root mount).
    no_root = copy.deepcopy(base_layout())
    no_root["mounts"] = [m for m in no_root["mounts"] if m["path"] != "/"]
    res_invalid = run_aiosh("layout", "validate", "--spec", json.dumps(no_root), "--json")
    expect_code(res_invalid, 1, "validate FL1 violation")
    env_invalid = json_envelope(res_invalid, 1, "validate FL1 violation")
    assert env_invalid["data"]["valid"] is False
    assert env_invalid["error"]["code"] == "VALIDATION_FAILED"
    assert "FL1" in env_invalid["error"]["message"]

    # Boundary: root mount present but with the wrong fsck pass number (FL1).
    bad_pass = copy.deepcopy(base_layout())
    for mount in bad_pass["mounts"]:
        if mount["path"] == "/":
            mount["pass"] = 2
    res_pass = run_aiosh("layout", "validate", "--spec", json.dumps(bad_pass), "--json")
    expect_code(res_pass, 1, "validate root pass boundary")
    assert json_envelope(res_pass, 1, "root pass boundary")["data"]["valid"] is False

    # Primary failure mode: malformed JSON spec.
    res_malformed = run_aiosh("layout", "validate", "--spec", "{ this is not json", "--json")
    expect_code(res_malformed, 1, "validate malformed spec")
    assert json_envelope(res_malformed, 1, "malformed spec")["data"]["valid"] is False

    print("PASS: aiosh layout validate/check (valid, FL1 + pass boundary, malformed spec)")


# ---------------------------------------------------------------------------
# probe
# ---------------------------------------------------------------------------

def test_layout_probe_valid_boundary_and_invalid() -> None:
    # Viable: generous target.
    env = json_envelope(
        run_aiosh("layout", "probe", "--bytes", str(UEFI_MIN_BYTES * 2), "--json"), 0, "probe viable"
    )
    assert env["data"]["is_viable"] is True
    assert env["data"]["target_disk_bytes"] == UEFI_MIN_BYTES * 2
    assert env["data"]["errors"] == []

    # Boundary: exactly the minimum target is still viable.
    env = json_envelope(
        run_aiosh("layout", "probe", "--bytes", str(UEFI_MIN_BYTES), "--json"), 0, "probe at minimum"
    )
    assert env["data"]["is_viable"] is True
    assert env["data"]["required_disk_bytes"] == UEFI_MIN_BYTES

    # Boundary: one byte below the required minimum must fail viability.
    env = json_envelope(
        run_aiosh("layout", "probe", "--bytes", str(UEFI_MIN_BYTES - 1), "--json"),
        1, "probe below minimum",
    )
    assert env["data"]["is_viable"] is False
    assert env["data"]["errors"], "non-viable probe must report at least one error"
    assert env["error"]["code"] == "NOT_VIABLE"

    # Primary failure mode: target cannot fit the partition allocation.
    env = json_envelope(
        run_aiosh("layout", "probe", "--bytes", str(UEFI_PARTITION_BYTES // 2), "--json"),
        1, "probe cannot fit partitions",
    )
    assert env["data"]["is_viable"] is False

    # Invalid input: unparseable byte count is a usage error (T-01522 §3.4).
    for bad in ("abc", "-1", "1.5"):
        res_bad = run_aiosh("layout", "probe", "--bytes", bad, "--json")
        expect_code(res_bad, 2, f"probe --bytes {bad}")
        assert json_envelope(res_bad, 2, f"probe --bytes {bad}")["error"]["code"] == "ARGUMENT_ERROR"

    print("PASS: aiosh layout probe (viable, minimum boundary, below-minimum, invalid bytes)")


# ---------------------------------------------------------------------------
# diff
# ---------------------------------------------------------------------------

def test_layout_diff() -> None:
    # Different layouts: partitions/mounts are removed, so the transition is destructive.
    env = json_envelope(
        run_aiosh("layout", "diff", UEFI_ID, CONTAINER_ID, "--json"), 0, "diff presets"
    )
    assert env["data"]["destructive"] is True
    assert env["data"]["partitions_removed"], "expected partitions removed moving UEFI -> container"
    assert env["data"]["source_layout_id"] == UEFI_ID
    assert env["data"]["target_layout_id"] == CONTAINER_ID
    assert env["data"]["summary"]

    # Boundary: a layout against itself is a no-op and must not be flagged destructive.
    env = json_envelope(run_aiosh("layout", "diff", UEFI_ID, UEFI_ID, "--json"), 0, "diff self")
    assert env["data"]["destructive"] is False
    assert env["data"]["partitions_added"] == []
    assert env["data"]["partitions_removed"] == []
    assert env["data"]["mounts_removed"] == []

    # Primary failure mode: unknown source/target ID.
    res_missing = run_aiosh("layout", "diff", "no-such-a", "no-such-b", "--json")
    expect_code(res_missing, 1, "diff unknown ids")
    assert json_envelope(res_missing, 1, "diff unknown ids")["error"]["code"] == "DIFF_FAILED"

    print("PASS: aiosh layout diff (destructive, self-diff, unknown ids -> exit 1)")


# ---------------------------------------------------------------------------
# fstab
# ---------------------------------------------------------------------------

def test_layout_fstab() -> None:
    res = run_aiosh("layout", "fstab")
    expect_code(res, 0, "layout fstab")
    assert "/etc/fstab" in res.stdout
    assert UEFI_ID in res.stdout

    env = json_envelope(run_aiosh("layout", "fstab", "--json"), 0, "layout fstab --json")
    content = env["data"]["fstab"]
    assert env["data"]["id"] == UEFI_ID
    assert "/etc/fstab" in content
    assert "LABEL=AIOS_ROOT" in content
    # Six-field fstab rows: a root mount carrying fsck pass 1 must be emitted.
    root_lines = [ln for ln in content.splitlines() if ln.split() and ln.split()[1] == "/"]
    assert root_lines, "no root mount row in generated fstab"
    assert root_lines[0].split()[5] == "1", f"root row pass != 1: {root_lines[0]}"

    print("PASS: aiosh layout fstab (prose and JSON, six-field rows)")


# ---------------------------------------------------------------------------
# register / set-active / remove / import-fstab against a real store file
# ---------------------------------------------------------------------------

def _load_store(path: Path) -> dict:
    assert path.exists(), f"store file was not written: {path}"
    return json.loads(path.read_text(encoding="utf-8"))


def test_layout_store_lifecycle() -> None:
    with tempfile.TemporaryDirectory() as td:
        store = Path(td) / "fs_layouts.json"
        store_arg = str(store)

        custom = copy.deepcopy(base_layout())
        custom["id"] = "smoke-custom-v1"
        custom["name"] = "Smoke Custom Layout"

        spec_file = Path(td) / "custom_spec.json"
        spec_file.write_text(json.dumps(custom), encoding="utf-8")

        # --- register from a file path (documented form) ---
        env = json_envelope(
            run_aiosh("layout", "register", "--spec", str(spec_file),
                      "--store", store_arg, "--json"),
            0, "register from file",
        )
        assert env["data"]["registered"] is True
        assert env["data"]["id"] == "smoke-custom-v1"

        # Observable side effect: the store file now exists and contains the layout.
        persisted = _load_store(store)
        assert "smoke-custom-v1" in persisted["layouts"]

        # --- duplicate registration is rejected ---
        res_dup = run_aiosh("layout", "register", "--spec", json.dumps(custom),
                            "--store", store_arg, "--json")
        expect_code(res_dup, 1, "duplicate register")
        assert json_envelope(res_dup, 1, "duplicate register")["error"]["code"] == "REGISTER_FAILED"

        # --- inline JSON spec is accepted (documented form) ---
        inline = copy.deepcopy(base_layout())
        inline["id"] = "smoke-inline-v1"
        inline["name"] = "Smoke Inline Layout"
        env = json_envelope(
            run_aiosh("layout", "register", "--spec", json.dumps(inline),
                      "--store", store_arg, "--json"),
            0, "register inline",
        )
        assert env["data"]["id"] == "smoke-inline-v1"

        # --- list reflects persisted state ---
        env = json_envelope(run_aiosh("layout", "list", "--store", store_arg, "--json"), 0, "list store")
        ids = [layout["id"] for layout in env["data"]["layouts"]]
        assert "smoke-custom-v1" in ids and "smoke-inline-v1" in ids

        # --- set-active switches the pointer and persists it ---
        env = json_envelope(
            run_aiosh("layout", "set-active", "smoke-custom-v1", "--store", store_arg, "--json"),
            0, "set-active",
        )
        assert env["data"]["active"] == "smoke-custom-v1"
        assert env["data"]["previous_active"] == UEFI_ID
        assert _load_store(store)["active_layout_id"] == "smoke-custom-v1"

        # --- --standard names the canonical preset even when a custom layout is active ---
        env = json_envelope(
            run_aiosh("layout", "show", "--standard", "--store", store_arg, "--json"),
            0, "show --standard over active custom",
        )
        assert env["data"]["id"] == UEFI_ID

        # --- the active layout cannot be removed ---
        res_active = run_aiosh("layout", "remove", "smoke-custom-v1", "--store", store_arg, "--json")
        expect_code(res_active, 1, "remove active layout")
        assert json_envelope(res_active, 1, "remove active")["error"]["code"] == "REMOVE_FAILED"
        assert "smoke-custom-v1" in _load_store(store)["layouts"]

        # --- import-fstab creates a profile inheriting from the active layout ---
        fstab_text = "/dev/sda2 / ext4 defaults 1 1\n/dev/sda1 /boot/efi vfat umask=0077 0 2\n"
        env = json_envelope(
            run_aiosh("layout", "import-fstab", "smoke-import-v1", "Smoke Imported",
                      "--fstab", fstab_text, "--store", store_arg, "--json"),
            0, "import-fstab",
        )
        assert env["data"]["id"] == "smoke-import-v1"
        assert len(env["data"]["mounts"]) == 2
        assert "smoke-import-v1" in _load_store(store)["layouts"]

        # --- primary failure mode: fstab content with no usable mount rows ---
        res_empty = run_aiosh("layout", "import-fstab", "smoke-empty-v1", "Empty Import",
                              "--fstab", "# only a comment\n", "--store", store_arg, "--json")
        expect_code(res_empty, 1, "import-fstab empty content")
        assert json_envelope(res_empty, 1, "empty fstab")["error"]["code"] == "IMPORT_FAILED"

        # --- re-point and remove the custom layout ---
        expect_code(
            run_aiosh("layout", "set-active", UEFI_ID, "--store", store_arg, "--json"),
            0, "set-active back to preset",
        )
        expect_code(
            run_aiosh("layout", "remove", "smoke-custom-v1", "--store", store_arg, "--json"),
            0, "remove custom layout",
        )
        assert "smoke-custom-v1" not in _load_store(store)["layouts"]

        # --- repeat removal fails, and built-ins are protected ---
        res_again = run_aiosh("layout", "remove", "smoke-custom-v1", "--store", store_arg, "--json")
        expect_code(res_again, 1, "remove already-removed layout")
        res_builtin = run_aiosh("layout", "remove", CONTAINER_ID, "--store", store_arg, "--json")
        expect_code(res_builtin, 1, "remove built-in preset")
        assert CONTAINER_ID in _load_store(store)["layouts"]

    print("PASS: aiosh layout register/set-active/remove/import-fstab store lifecycle")


# ---------------------------------------------------------------------------
# Argument boundaries
# ---------------------------------------------------------------------------

def test_layout_argument_boundaries() -> None:
    # Missing mandatory arguments -> usage error.
    for argv in (
        ("layout", "register", "--json"),
        ("layout", "set-active", "--json"),
        ("layout", "remove", "--json"),
        ("layout", "import-fstab", "--json"),
    ):
        res = run_aiosh(*argv)
        expect_code(res, 2, f"{' '.join(argv)} (missing args)")
        assert json_envelope(res, 2, f"{' '.join(argv)}")["error"]["code"] == "ARGUMENT_ERROR"

    # Control characters in --store are rejected before any filesystem access.
    res_ctrl = run_aiosh("layout", "list", "--store", "bad\x01store", "--json")
    expect_code(res_ctrl, 2, "control char store path")
    assert json_envelope(res_ctrl, 2, "control char store")["error"]["code"] == "INVALID_ARGUMENT"

    # Boundary: 1024-character store path is accepted by the guard, 1025 is rejected.
    expect_code(run_aiosh("layout", "list", "--store", "x" * 1024, "--json"), 0, "store path == 1024")
    res_long = run_aiosh("layout", "list", "--store", "x" * 1025, "--json")
    expect_code(res_long, 2, "store path == 1025")
    assert json_envelope(res_long, 2, "over-long store path")["error"]["code"] == "INVALID_ARGUMENT"

    print("PASS: aiosh layout argument boundaries (missing args, control chars, path length)")


def main() -> int:
    test_layout_help_and_unknown_subcommand()
    test_layout_list()
    test_layout_show_valid_and_invalid()
    test_layout_validate_valid_invalid_and_boundary()
    test_layout_probe_valid_boundary_and_invalid()
    test_layout_diff()
    test_layout_fstab()
    test_layout_store_lifecycle()
    test_layout_argument_boundaries()
    print("\nALL FILESYSTEM LAYOUT CLI SMOKE TESTS PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
