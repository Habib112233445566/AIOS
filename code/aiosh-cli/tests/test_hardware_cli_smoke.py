#!/usr/bin/env python3
"""CLI Smoke & Boundary Test for Hardware Detection (T-01726).

Exercises the operator surface `aiosh hw <subcommand>` end-to-end through the real
binary: exit codes, JSON result envelopes, and domain behavior.

Coverage: valid input, invalid input, boundary values, and primary failure modes for
each subcommand: scan, list, show, summary, verify.
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


def test_hw_help_and_unknown() -> None:
    res = run_aiosh("hw", "--help")
    expect_code(res, 0, "hw --help")
    for token in ("aiosh hw", "scan", "list", "show", "summary", "verify"):
        assert token in res.stdout, f"hw --help missing token {token!r}"

    res_unknown = run_aiosh("hw", "bogus_subcmd", "--json")
    expect_code(res_unknown, 2, "hw bogus_subcmd")
    assert json_envelope(res_unknown, 2, "unknown subcommand")["error"]["code"] == "UNKNOWN_SUBCOMMAND"
    print("PASS: aiosh hw --help and unknown subcommand")


def test_hw_path_hygiene_and_validation() -> None:
    # 1. Path length > 1024
    res_long_sys = run_aiosh("hw", "scan", "--sysfs", "a" * 1025, "--json")
    expect_code(res_long_sys, 2, "hw scan sysfs too long")
    assert json_envelope(res_long_sys, 2, "sysfs too long")["error"]["code"] == "PATH_TOO_LONG"

    # 2. Control character in procfs
    res_ctrl_proc = run_aiosh("hw", "scan", "--procfs", "bad\x07path", "--json")
    expect_code(res_ctrl_proc, 2, "hw scan procfs control char")
    assert json_envelope(res_ctrl_proc, 2, "procfs control char")["error"]["code"] == "PATH_CONTAINS_CONTROL_CHAR"

    # 3. Control character in file
    res_ctrl_file = run_aiosh("hw", "verify", "--file", "bad\x07file", "--json")
    expect_code(res_ctrl_file, 2, "hw verify file control char")
    assert json_envelope(res_ctrl_file, 2, "file control char")["error"]["code"] == "PATH_CONTAINS_CONTROL_CHAR"

    # 4. Invalid device class
    res_bad_class = run_aiosh("hw", "scan", "--class", "invalid_class", "--json")
    expect_code(res_bad_class, 2, "hw scan invalid class")
    assert json_envelope(res_bad_class, 2, "invalid class")["error"]["code"] == "INVALID_DEVICE_CLASS"

    # 5. Missing device ID for show
    res_no_id = run_aiosh("hw", "show", "--json")
    expect_code(res_no_id, 2, "hw show no id")
    assert json_envelope(res_no_id, 2, "missing device id")["error"]["code"] == "MISSING_DEVICE_ID"

    print("PASS: aiosh hw path hygiene and validation")


def test_hw_mock_subsystems() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        sysfs = tmp_path / "sys"
        procfs = tmp_path / "proc"

        # Mock PCI device (0000:00:02.0 GPU)
        pci_dir = sysfs / "bus/pci/devices/0000_00_02.0"
        pci_dir.mkdir(parents=True)
        (pci_dir / "vendor").write_text("0x8086\n")
        (pci_dir / "device").write_text("0x9bc4\n")
        (pci_dir / "class").write_text("0x030000\n")
        (pci_dir / "driver").write_text("i915\n")

        # Mock Network device (eth0)
        net_dir = sysfs / "class/net/eth0"
        net_dir.mkdir(parents=True)
        (net_dir / "address").write_text("52:54:00:12:34:56\n")
        (net_dir / "operstate").write_text("up\n")
        (net_dir / "speed").write_text("1000\n")

        # Mock CPU info
        procfs.mkdir(parents=True)
        (procfs / "cpuinfo").write_text("model name : AMD EPYC\nprocessor : 0\n")

        sysfs_str = str(sysfs)
        procfs_str = str(procfs)

        # 1. Scan
        res_scan = run_aiosh("hw", "scan", "--sysfs", sysfs_str, "--procfs", procfs_str, "--json")
        expect_code(res_scan, 0, "hw scan")
        scan_data = json_envelope(res_scan, 0, "hw scan")["data"]
        assert len(scan_data["devices"]) >= 2
        assert "summary" in scan_data
        assert scan_data["summary"].get("gpu", 0) >= 1

        # 2. Scan with class filter
        res_filter = run_aiosh("hw", "scan", "--class", "gpu", "--sysfs", sysfs_str, "--procfs", procfs_str, "--json")
        expect_code(res_filter, 0, "hw scan --class gpu")
        filter_data = json_envelope(res_filter, 0, "hw scan class filter")["data"]
        assert all(d["class"] == "gpu" for d in filter_data["devices"])

        # 3. Scan with --no-attrs
        res_no_attrs = run_aiosh("hw", "scan", "--no-attrs", "--sysfs", sysfs_str, "--procfs", procfs_str, "--json")
        expect_code(res_no_attrs, 0, "hw scan --no-attrs")
        no_attrs_data = json_envelope(res_no_attrs, 0, "hw scan no attrs")["data"]
        for d in no_attrs_data["devices"]:
            assert len(d.get("attributes", {})) == 0

        # 4. List
        res_list = run_aiosh("hw", "list", "--sysfs", sysfs_str, "--procfs", procfs_str, "--json")
        expect_code(res_list, 0, "hw list")
        list_data = json_envelope(res_list, 0, "hw list")["data"]
        assert "devices" in list_data and "count" in list_data
        assert list_data["count"] >= 2

        # 5. Summary
        res_sum = run_aiosh("hw", "summary", "--sysfs", sysfs_str, "--procfs", procfs_str, "--json")
        expect_code(res_sum, 0, "hw summary")
        sum_data = json_envelope(res_sum, 0, "hw summary")["data"]
        assert "summary" in sum_data and "total" in sum_data
        assert sum_data["total"] >= 2

        # 6. Show existing device
        res_show = run_aiosh("hw", "show", "pci:0000:00:02.0", "--sysfs", sysfs_str, "--procfs", procfs_str, "--json")
        expect_code(res_show, 0, "hw show existing")
        show_data = json_envelope(res_show, 0, "hw show existing")["data"]
        assert show_data["device"]["id"] == "pci:0000:00:02.0"
        assert show_data["device"]["class"] == "gpu"

        # 7. Show nonexistent device
        res_show_miss = run_aiosh("hw", "show", "pci:nonexistent", "--sysfs", sysfs_str, "--procfs", procfs_str, "--json")
        expect_code(res_show_miss, 1, "hw show nonexistent")
        assert json_envelope(res_show_miss, 1, "show nonexistent")["error"]["code"] == "DEVICE_NOT_FOUND"

        # 8. Verify live scan
        res_verify = run_aiosh("hw", "verify", "--sysfs", sysfs_str, "--procfs", procfs_str, "--json")
        expect_code(res_verify, 0, "hw verify live")
        verify_data = json_envelope(res_verify, 0, "hw verify live")["data"]
        assert verify_data["valid"] is True
        assert verify_data["device_count"] >= 2

        # 9. Verify file
        inv_file = tmp_path / "valid_inventory.json"
        inv_file.write_text(json.dumps(scan_data))
        res_vf = run_aiosh("hw", "verify", "--file", str(inv_file), "--json")
        expect_code(res_vf, 0, "hw verify file valid")
        assert json_envelope(res_vf, 0, "verify valid file")["data"]["valid"] is True

        # 10. Verify corrupt file
        corrupt_file = tmp_path / "corrupt_inventory.json"
        corrupt_file.write_text("{ broken json ...")
        res_corrupt = run_aiosh("hw", "verify", "--file", str(corrupt_file), "--json")
        expect_code(res_corrupt, 1, "hw verify file corrupt")
        assert json_envelope(res_corrupt, 1, "verify corrupt file")["error"]["code"] == "VERIFICATION_FAILED"

        # 11. Verify missing file
        res_missing = run_aiosh("hw", "verify", "--file", str(tmp_path / "missing.json"), "--json")
        expect_code(res_missing, 1, "hw verify file missing")
        assert json_envelope(res_missing, 1, "verify missing file")["error"]["code"] == "VERIFICATION_FAILED"

    print("PASS: aiosh hw mock subsystems")


def main() -> None:
    print(f"Running Hardware Detection CLI smoke suite with binary: {get_binary_path()}")
    test_hw_help_and_unknown()
    test_hw_path_hygiene_and_validation()
    test_hw_mock_subsystems()
    print("ALL TESTS PASSED: aiosh hw CLI smoke test suite.")


if __name__ == "__main__":
    main()
