#!/usr/bin/env python3
"""CLI Smoke & Boundary Test for Kernel Module Management (T-01626 / epic T-01611..T-01630).

Exercises the operator surface `aiosh mod <subcommand>` end-to-end through the real
binary: exit codes, JSON result envelopes, and on-disk store state.

Coverage: valid input, invalid input, boundary values, and primary failure modes for
each subcommand. Assertions target observable behaviour (process exit status, stdout
envelope, persisted store JSON).

Run standalone:
    python code/aiosh-cli/tests/test_kernel_module_cli_smoke.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


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
    payload = parse_json_output(res)
    assert isinstance(payload, dict), f"{context}: expected a JSON object envelope"
    assert payload.get("code") == expected_code, (
        f"{context}: envelope code {payload.get('code')} != {expected_code}"
    )
    assert "data" in payload and "error" in payload, f"{context}: envelope missing data/error keys"
    return payload


def test_mod_help_and_unknown() -> None:
    res = run_aiosh("mod", "--help")
    expect_code(res, 0, "mod --help")
    for token in ("aiosh mod", "list", "show", "blacklist", "unblacklist", "options", "autoload", "unautoload", "preset", "export"):
        assert token in res.stdout, f"mod --help missing token {token!r}"

    res_unknown = run_aiosh("mod", "bogus_subcmd", "--json")
    expect_code(res_unknown, 2, "mod bogus_subcmd")
    assert json_envelope(res_unknown, 2, "unknown subcommand")["error"]["code"] == "UNKNOWN_SUBCOMMAND"
    print("PASS: aiosh mod --help and unknown subcommand")


def test_mod_list_and_show() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_file = str(tmp_path / "store.json")
        proc_file = str(tmp_path / "modules.txt")
        with open(proc_file, "w") as f:
            f.write("overlay 151552 1 - Live 0x0000000000000000\next4 983040 2 - Live 0x0000000000000000\n")

        # 1. List
        res_list = run_aiosh("mod", "list", "--store", store_file, "--proc-modules", proc_file, "--json")
        expect_code(res_list, 0, "mod list")
        data = json_envelope(res_list, 0, "mod list")["data"]
        assert len(data["loaded_modules"]) == 2
        assert data["loaded_modules"][0]["name"] == "overlay"

        # 2. Show live module
        res_show = run_aiosh("mod", "show", "overlay", "--store", store_file, "--proc-modules", proc_file, "--json")
        expect_code(res_show, 0, "mod show overlay")
        show_data = json_envelope(res_show, 0, "mod show overlay")["data"]
        assert show_data["module"]["name"] == "overlay"
        assert show_data["module"]["state"] == "live"

        # 3. Show missing module
        res_missing = run_aiosh("mod", "show", "nonexistent", "--store", store_file, "--proc-modules", proc_file, "--json")
        expect_code(res_missing, 1, "mod show nonexistent")
        assert json_envelope(res_missing, 1, "mod show nonexistent")["error"]["code"] == "MODULE_NOT_FOUND"

        # 4. Show without module name
        res_no_arg = run_aiosh("mod", "show", "--json")
        expect_code(res_no_arg, 2, "mod show no arg")
        assert json_envelope(res_no_arg, 2, "mod show no arg")["error"]["code"] == "MISSING_MODULE_NAME"

    print("PASS: aiosh mod list and show")


def test_mod_blacklist_and_unblacklist() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_file = str(tmp_path / "store.json")

        # 1. Missing arg
        res_no_arg = run_aiosh("mod", "blacklist", "--json")
        expect_code(res_no_arg, 2, "mod blacklist no arg")

        # 2. Valid blacklist
        res_bl = run_aiosh("mod", "blacklist", "usb_storage", "--store", store_file, "--json")
        expect_code(res_bl, 0, "mod blacklist usb_storage")
        assert json_envelope(res_bl, 0, "mod blacklist")["data"]["blacklisted"] is True

        # 3. Verify in show
        res_show = run_aiosh("mod", "show", "usb_storage", "--store", store_file, "--json")
        expect_code(res_show, 0, "mod show usb_storage")
        show_data = json_envelope(res_show, 0, "mod show usb_storage")["data"]
        assert any(r.get("type") == "blacklist" for r in show_data["rules"])

        # 4. Conflict: cannot autoload blacklisted module
        res_conflict = run_aiosh("mod", "autoload", "usb_storage", "--store", store_file, "--json")
        expect_code(res_conflict, 1, "mod autoload blacklisted module")

        # 5. Unblacklist
        res_unbl = run_aiosh("mod", "unblacklist", "usb_storage", "--store", store_file, "--json")
        expect_code(res_unbl, 0, "mod unblacklist usb_storage")
        assert json_envelope(res_unbl, 0, "mod unblacklist")["data"]["removed"] is True

    print("PASS: aiosh mod blacklist and unblacklist")


def test_mod_autoload_and_unautoload() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_file = str(tmp_path / "store.json")

        # 1. Missing arg
        res_no_arg = run_aiosh("mod", "autoload", "--json")
        expect_code(res_no_arg, 2, "mod autoload no arg")

        # 2. Valid autoload
        res_auto = run_aiosh("mod", "autoload", "br_netfilter", "--store", store_file, "--json")
        expect_code(res_auto, 0, "mod autoload br_netfilter")
        assert json_envelope(res_auto, 0, "mod autoload")["data"]["autoload"] is True

        # 3. Verify in show
        res_show = run_aiosh("mod", "show", "br_netfilter", "--store", store_file, "--json")
        expect_code(res_show, 0, "mod show br_netfilter")
        show_data = json_envelope(res_show, 0, "mod show br_netfilter")["data"]
        assert show_data["autoload"] is True

        # 4. Conflict: cannot blacklist autoloaded module
        res_conflict = run_aiosh("mod", "blacklist", "br_netfilter", "--store", store_file, "--json")
        expect_code(res_conflict, 1, "mod blacklist autoloaded module")

        # 5. Unautoload
        res_unauto = run_aiosh("mod", "unautoload", "br_netfilter", "--store", store_file, "--json")
        expect_code(res_unauto, 0, "mod unautoload br_netfilter")
        assert json_envelope(res_unauto, 0, "mod unautoload")["data"]["removed"] is True

    print("PASS: aiosh mod autoload and unautoload")


def test_mod_options() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_file = str(tmp_path / "store.json")

        # 1. Missing module arg
        res_no_mod = run_aiosh("mod", "options", "--json")
        expect_code(res_no_mod, 2, "mod options no mod")

        # 2. Missing option params
        res_no_opts = run_aiosh("mod", "options", "e1000e", "--store", store_file, "--json")
        expect_code(res_no_opts, 2, "mod options no opts")

        # 3. Valid options
        res_opts = run_aiosh("mod", "options", "e1000e", "InterruptThrottleRate=1", "--store", store_file, "--json")
        expect_code(res_opts, 0, "mod options valid")
        data = json_envelope(res_opts, 0, "mod options valid")["data"]
        assert data["module"] == "e1000e"
        assert data["options"] == ["InterruptThrottleRate=1"]

        # 4. Invalid parameter format
        res_invalid = run_aiosh("mod", "options", "e1000e", "bad;param=1", "--store", store_file, "--json")
        expect_code(res_invalid, 1, "mod options invalid")

    print("PASS: aiosh mod options")


def test_mod_preset() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_file = str(tmp_path / "store.json")

        # 1. Missing action
        res_no_action = run_aiosh("mod", "preset", "--json")
        expect_code(res_no_action, 2, "mod preset no action")

        # 2. List presets
        res_list = run_aiosh("mod", "preset", "list", "--json")
        expect_code(res_list, 0, "mod preset list")
        presets = json_envelope(res_list, 0, "mod preset list")["data"]
        assert len(presets) == 3
        preset_names = [p["name"] for p in presets]
        assert "cis_hardened_baseline" in preset_names
        assert "pentest_wireless_baseline" in preset_names
        assert "container_isolation_baseline" in preset_names

        # 3. Apply missing name
        res_apply_no_name = run_aiosh("mod", "preset", "apply", "--json")
        expect_code(res_apply_no_name, 2, "mod preset apply no name")

        # 4. Apply unknown preset
        res_unknown = run_aiosh("mod", "preset", "apply", "nonexistent_preset", "--store", store_file, "--json")
        expect_code(res_unknown, 1, "mod preset apply unknown")

        # 5. Apply valid preset
        res_ok = run_aiosh("mod", "preset", "apply", "cis_hardened_baseline", "--store", store_file, "--json")
        expect_code(res_ok, 0, "mod preset apply cis_hardened_baseline")
        assert json_envelope(res_ok, 0, "mod preset apply")["data"]["applied"] is True

    print("PASS: aiosh mod preset")


def test_mod_export() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_file = str(tmp_path / "store.json")
        modprobe_out = str(tmp_path / "modprobe.conf")
        autoload_out = str(tmp_path / "modules-load.conf")

        # Apply a preset first so there is content to export
        run_aiosh("mod", "preset", "apply", "cis_hardened_baseline", "--store", store_file, "--json")

        # Export stdout
        res_export = run_aiosh("mod", "export", "--store", store_file, "--json")
        expect_code(res_export, 0, "mod export stdout")
        data = json_envelope(res_export, 0, "mod export")["data"]
        assert "modprobe_conf" in data
        assert "modules_load_conf" in data
        assert "install cramfs /bin/true" in data["modprobe_conf"]

        # Export to files
        res_files = run_aiosh(
            "mod", "export",
            "--store", store_file,
            "--modprobe", modprobe_out,
            "--autoload", autoload_out,
            "--json"
        )
        expect_code(res_files, 0, "mod export files")
        assert os.path.exists(modprobe_out)
        assert os.path.exists(autoload_out)
        with open(modprobe_out, "r") as f:
            content = f.read()
            assert "install cramfs /bin/true" in content

    print("PASS: aiosh mod export")


def test_mod_path_hygiene() -> None:
    # 1. Path length > 1024
    res_long_store = run_aiosh("mod", "list", "--store", "a" * 1025, "--json")
    expect_code(res_long_store, 2, "mod list store too long")

    # 2. Control character in path (ASCII 7 BEL)
    res_ctrl_store = run_aiosh("mod", "list", "--store", "bad\x07path", "--json")
    expect_code(res_ctrl_store, 2, "mod list store control char")

    # 3. Control character in proc-modules
    res_ctrl_proc = run_aiosh("mod", "list", "--proc-modules", "bad\x07path", "--json")
    expect_code(res_ctrl_proc, 2, "mod list proc control char")

    print("PASS: aiosh mod path hygiene")


def main() -> None:
    print(f"Running Kernel Module CLI smoke suite with binary: {get_binary_path()}")
    test_mod_help_and_unknown()
    test_mod_list_and_show()
    test_mod_blacklist_and_unblacklist()
    test_mod_autoload_and_unautoload()
    test_mod_options()
    test_mod_preset()
    test_mod_export()
    test_mod_path_hygiene()
    print("ALL TESTS PASSED: aiosh mod CLI smoke test suite.")


if __name__ == "__main__":
    main()
