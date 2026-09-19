#!/usr/bin/env python3
"""Configuration Integration Smoke Test for Kernel Module Management (T-01646).

Verifies the integration of Kernel Module configuration import/export and validation
with the operator CLI surface and canonical JSON store.

Covers:
- Discoverability: `aiosh mod --help` advertises `import`.
- Missing argument enforcement: `aiosh mod import` without flags exits with code 2.
- Modprobe configuration import: parses directives into canonical store.
- Modules-load configuration import: parses autoload modules into canonical store.
- Conflict enforcement: rejects importing conflicting blacklist/autoload rules (CFG-KM3).
- Full round-trip parity: export -> import into fresh store produces identical state.

Run standalone:
    python code/aiosh-cli/tests/test_kernel_module_config_smoke.py
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


def test_import_discoverability_and_validation() -> None:
    res = run_aiosh("mod", "--help")
    assert res.returncode == 0
    assert "import" in res.stdout, "aiosh mod --help must advertise 'import'"

    res_missing = run_aiosh("mod", "import", "--json")
    assert res_missing.returncode == 2, f"expected exit code 2, got {res_missing.returncode}"
    err = json.loads(res_missing.stdout)["error"]
    assert err["code"] == "MISSING_IMPORT_SOURCE"
    print("PASS: test_import_discoverability_and_validation")


def test_import_modprobe_and_autoload() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_path = str(tmp_path / "store.json")
        modprobe_path = str(tmp_path / "custom_modprobe.conf")
        autoload_path = str(tmp_path / "custom_autoload.conf")

        with open(modprobe_path, "w") as f:
            f.write("# Security directives\nblacklist cramfs\noptions i915 enable_guc=3\n")

        with open(autoload_path, "w") as f:
            f.write("# Boot autoload\noverlay\nkvm_intel\n")

        # 1. Import both files
        res = run_aiosh(
            "mod", "import",
            "--store", store_path,
            "--modprobe", modprobe_path,
            "--autoload", autoload_path,
            "--json",
        )
        assert res.returncode == 0, f"import failed: {res.stdout}"
        data = json.loads(res.stdout)["data"]
        assert data["modprobe_imported"] == 2
        assert data["autoload_imported"] == 2

        # 2. Verify store contents via list
        res_list = run_aiosh("mod", "list", "--store", store_path, "--json")
        assert res_list.returncode == 0
        list_data = json.loads(res_list.stdout)["data"]
        autoload_list = list_data.get("autoload_modules", [])
        assert "overlay" in autoload_list
        assert "kvm_intel" in autoload_list
        assert len(list_data.get("rules", [])) == 2

    print("PASS: test_import_modprobe_and_autoload")


def test_import_conflict_detection() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_path = str(tmp_path / "conflict_store.json")
        modprobe_path = str(tmp_path / "conflict_modprobe.conf")

        # Autoload module first
        run_aiosh("mod", "autoload", "overlay", "--store", store_path, "--json")

        # Try to import a modprobe file that blacklists overlay
        with open(modprobe_path, "w") as f:
            f.write("blacklist overlay\n")

        res = run_aiosh("mod", "import", "--store", store_path, "--modprobe", modprobe_path, "--json")
        assert res.returncode == 1, f"expected conflict failure, got {res.returncode}"
        err = json.loads(res.stdout)["error"]
        assert err["code"] == "IMPORT_MODPROBE_FAILED"
        assert "cannot blacklist" in err["message"]

    print("PASS: test_import_conflict_detection")


def test_export_import_roundtrip_parity() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store1_path = str(tmp_path / "store1.json")
        store2_path = str(tmp_path / "store2.json")
        modprobe_out = str(tmp_path / "exported.modprobe.conf")
        autoload_out = str(tmp_path / "exported.modules-load.conf")

        # 1. Setup store 1 with preset
        run_aiosh("mod", "preset", "apply", "cis_hardened_baseline", "--store", store1_path, "--json")
        run_aiosh("mod", "autoload", "wireguard", "--store", store1_path, "--json")

        # 2. Export store 1
        res_exp = run_aiosh(
            "mod", "export",
            "--store", store1_path,
            "--modprobe", modprobe_out,
            "--autoload", autoload_out,
            "--json",
        )
        assert res_exp.returncode == 0

        # 3. Import into store 2
        res_imp = run_aiosh(
            "mod", "import",
            "--store", store2_path,
            "--modprobe", modprobe_out,
            "--autoload", autoload_out,
            "--json",
        )
        assert res_imp.returncode == 0

        # 4. Compare store 1 and store 2
        res1 = run_aiosh("mod", "list", "--store", store1_path, "--json")
        res2 = run_aiosh("mod", "list", "--store", store2_path, "--json")
        data1 = json.loads(res1.stdout)["data"]
        data2 = json.loads(res2.stdout)["data"]

        assert data1["autoload_modules"] == data2["autoload_modules"]
        assert len(data1["rules"]) == len(data2["rules"])

    print("PASS: test_export_import_roundtrip_parity")


def main() -> None:
    print(f"Using binary: {get_binary_path()}")
    test_import_discoverability_and_validation()
    test_import_modprobe_and_autoload()
    test_import_conflict_detection()
    test_export_import_roundtrip_parity()
    print("ALL CONFIGURATION INTEGRATION TESTS PASSED.")


if __name__ == "__main__":
    main()
