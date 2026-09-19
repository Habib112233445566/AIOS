#!/usr/bin/env python3
"""Automated Lifecycle & Edge-Case Test Suite for Filesystem Layout (T-01553..T-01556).

Criteria FL10:
  A1: Full lifecycle state machine (list -> register -> set-active -> show -> diff -> probe -> remove)
  A2: Built-in layout deletion protection
  A3: Active layout deletion protection
  A4: fstab import & generation roundtrip
  A5: Target disk capacity feasibility probing (under, tight, generous)
  A6: Differential analysis and destructive change detection
  A7: Corrupted store tamper resistance
  A8: SQLite WAL audit trail verification (ADR-0035)

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_automated_cases.py
"""

from __future__ import annotations

import json
import os
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

UEFI_ID = "aios-uefi-standard-v1"
CONTAINER_ID = "aios-container-minimal-v1"
UEFI_MIN_BYTES = 64 * 1024 * 1024 * 1024


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


def run_aiosh(args: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess:
    base_env = os.environ.copy()
    if env:
        base_env.update(env)
    cmd = [get_binary_path(), *args]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=60, env=base_env)


def parse_json(res: subprocess.CompletedProcess) -> dict:
    raw = res.stdout.strip() or res.stderr.strip()
    try:
        return json.loads(raw)
    except json.JSONDecodeError as e:
        raise AssertionError(f"Failed to parse JSON from output: {raw!r}") from e


def make_valid_layout_spec(layout_id: str, root_size_mib: int = 50 * 1024) -> dict:
    return {
        "id": layout_id,
        "name": f"Test Layout {layout_id}",
        "description": "Automated test layout",
        "target_disk_min_bytes": 64 * 1024 * 1024 * 1024,
        "partitions": [
            {
                "index": 1,
                "label": "EFI",
                "partition_type": "efi_system",
                "size_mib": 512,
                "uuid": None,
                "bootable": True,
                "format_as": "vfat",
            },
            {
                "index": 2,
                "label": "AIOS_ROOT",
                "partition_type": "linux_root",
                "size_mib": root_size_mib,
                "uuid": None,
                "bootable": False,
                "format_as": "ext4",
            },
        ],
        "mounts": [
            {
                "path": "/",
                "device": "LABEL=AIOS_ROOT",
                "fs_type": "ext4",
                "options": ["rw", "relatime"],
                "dump": 0,
                "pass": 1,
                "required": True,
            },
            {
                "path": "/boot/efi",
                "device": "LABEL=EFI",
                "fs_type": "vfat",
                "options": ["rw", "nodev", "nosuid"],
                "dump": 0,
                "pass": 2,
                "required": True,
            },
            {
                "path": "/tmp",
                "device": "tmpfs",
                "fs_type": "tmpfs",
                "options": ["rw", "nodev", "nosuid", "noexec"],
                "dump": 0,
                "pass": 0,
                "required": True,
            },
            {
                "path": "/dev/shm",
                "device": "tmpfs",
                "fs_type": "tmpfs",
                "options": ["rw", "nodev", "nosuid", "noexec"],
                "dump": 0,
                "pass": 0,
                "required": True,
            },
        ],
        "directories": [
            {
                "path": "/var/lib/aios",
                "mode": 0o750,
                "owner": "root",
                "group": "aios",
                "description": "State directory",
                "symlink_target": None,
            },
            {
                "path": "/tmp",
                "mode": 0o1777,
                "owner": "root",
                "group": "root",
                "description": "Temp directory",
                "symlink_target": None,
            },
        ],
        "created_at": "2026-09-19T00:00:00Z",
    }


def test_a1_full_lifecycle(tmp_dir: Path) -> None:
    """A1: Full lifecycle state machine."""
    store = tmp_dir / "store_a1.json"
    spec_path = tmp_dir / "spec_a1.json"
    spec_data = make_valid_layout_spec("custom-srv-v1")
    spec_path.write_text(json.dumps(spec_data), encoding="utf-8")

    # 1. List initial store (seeds built-ins)
    res = run_aiosh(["layout", "list", "--store", str(store), "--json"])
    assert res.returncode == 0, f"list failed: {res.stderr}"
    data = parse_json(res)["data"]
    ids = [item["id"] for item in data["layouts"]]
    assert UEFI_ID in ids and CONTAINER_ID in ids

    # 2. Register custom layout
    res = run_aiosh(["layout", "register", "--spec", str(spec_path), "--store", str(store), "--json"])
    assert res.returncode == 0, f"register failed: {res.stderr}\n{res.stdout}"
    assert parse_json(res)["data"]["registered"] is True

    # 3. List contains custom layout
    res = run_aiosh(["layout", "list", "--store", str(store), "--json"])
    data = parse_json(res)["data"]
    ids = [item["id"] for item in data["layouts"]]
    assert "custom-srv-v1" in ids

    # 4. Set active
    res = run_aiosh(["layout", "set-active", "custom-srv-v1", "--store", str(store), "--json"])
    assert res.returncode == 0, f"set-active failed: {res.stderr}\n{res.stdout}"
    assert parse_json(res)["data"]["active"] == "custom-srv-v1"

    # 5. Show without ID returns active
    res = run_aiosh(["layout", "show", "--store", str(store), "--json"])
    assert res.returncode == 0, f"show failed: {res.stderr}\n{res.stdout}"
    assert parse_json(res)["data"]["id"] == "custom-srv-v1"

    # 6. Diff between standard and custom
    res = run_aiosh(["layout", "diff", UEFI_ID, "custom-srv-v1", "--store", str(store), "--json"])
    assert res.returncode == 0, f"diff failed: {res.stderr}\n{res.stdout}"
    diff = parse_json(res)["data"]
    assert diff["source_layout_id"] == UEFI_ID
    assert diff["target_layout_id"] == "custom-srv-v1"

    # 7. Probe feasibility on active
    res = run_aiosh(["layout", "probe", "--bytes", "100000000000", "--store", str(store), "--json"])
    assert res.returncode == 0, f"probe failed: {res.stderr}\n{res.stdout}"
    probe = parse_json(res)["data"]
    assert probe["is_viable"] is True

    # 8. Restore active to UEFI
    res = run_aiosh(["layout", "set-active", UEFI_ID, "--store", str(store), "--json"])
    assert res.returncode == 0

    # 9. Remove custom layout
    res = run_aiosh(["layout", "remove", "custom-srv-v1", "--store", str(store), "--json"])
    assert res.returncode == 0, f"remove failed: {res.stderr}\n{res.stdout}"
    assert parse_json(res)["data"]["removed"] is True

    # Confirm removed from list
    res = run_aiosh(["layout", "list", "--store", str(store), "--json"])
    ids = [item["id"] for item in parse_json(res)["data"]["layouts"]]
    assert "custom-srv-v1" not in ids
    print("  [+] A1: Full lifecycle state machine PASS")


def test_a2_builtin_protection(tmp_dir: Path) -> None:
    """A2: Built-in layout deletion protection."""
    store = tmp_dir / "store_a2.json"
    res = run_aiosh(["layout", "remove", CONTAINER_ID, "--store", str(store), "--json"])
    assert res.returncode == 1, f"expected failure removing builtin, got code {res.returncode}"
    err = parse_json(res)["error"]
    err_str = str(err).lower()
    assert "built-in" in err_str or "builtin" in err_str
    print("  [+] A2: Built-in protection PASS")


def test_a3_active_protection(tmp_dir: Path) -> None:
    """A3: Active layout deletion protection."""
    store = tmp_dir / "store_a3.json"
    # UEFI_ID is default active
    res = run_aiosh(["layout", "remove", UEFI_ID, "--store", str(store), "--json"])
    assert res.returncode == 1, f"expected failure removing active layout, got code {res.returncode}"
    err = parse_json(res)["error"]
    err_str = str(err).lower()
    assert "active" in err_str
    print("  [+] A3: Active layout protection PASS")


def test_a4_fstab_cycle(tmp_dir: Path) -> None:
    """A4: fstab import & generation roundtrip."""
    store = tmp_dir / "store_a4.json"
    fstab_path = tmp_dir / "test_fstab"
    fstab_content = (
        "LABEL=ROOT  /          ext4   rw,relatime                0 1\n"
        "LABEL=BOOT  /boot/efi  vfat   rw,nodev,nosuid            0 2\n"
        "tmpfs       /tmp       tmpfs  rw,nodev,nosuid,noexec     0 0\n"
        "tmpfs       /dev/shm   tmpfs  rw,nodev,nosuid,noexec     0 0\n"
    )
    fstab_path.write_text(fstab_content, encoding="utf-8")

    # Import fstab
    res = run_aiosh([
        "layout", "import-fstab", "imported-srv-v1", "Imported Layout",
        "--fstab", str(fstab_path), "--store", str(store), "--json"
    ])
    assert res.returncode == 0, f"import-fstab failed: {res.stderr}\n{res.stdout}"
    assert parse_json(res)["data"]["id"] == "imported-srv-v1"

    # Export fstab
    res = run_aiosh(["layout", "fstab", "imported-srv-v1", "--store", str(store), "--json"])
    assert res.returncode == 0, f"fstab export failed: {res.stderr}\n{res.stdout}"
    fstab_str = parse_json(res)["data"]["fstab"]
    mount_lines = [l for l in fstab_str.splitlines() if l and not l.startswith("#")]
    assert len(mount_lines) == 4
    for ml in mount_lines:
        fields = ml.split()
        assert len(fields) == 6, f"expected 6 fields in fstab line, got: {fields}"
    print("  [+] A4: fstab import & generation roundtrip PASS")


def test_a5_probe_feasibility(tmp_dir: Path) -> None:
    """A5: Target disk capacity feasibility probing."""
    store = tmp_dir / "store_a5.json"

    # Undersized: 10 GiB vs 64 GiB min (exit code 1, is_viable is False)
    res = run_aiosh(["layout", "probe", "--standard", "--bytes", str(10 * 1024 * 1024 * 1024), "--store", str(store), "--json"])
    assert res.returncode == 1, f"expected code 1 for undersized probe, got {res.returncode}"
    probe = parse_json(res)["data"]
    assert probe["is_viable"] is False
    assert len(probe["errors"]) > 0

    # Viable at minimum standard: 64 GiB
    res = run_aiosh(["layout", "probe", "--standard", "--bytes", str(UEFI_MIN_BYTES), "--store", str(store), "--json"])
    assert res.returncode == 0
    probe = parse_json(res)["data"]
    assert probe["is_viable"] is True
    assert len(probe["errors"]) == 0

    # Tight capacity warning: register layout with 60 GiB partitions, probe with 61 GiB (<10% slack)
    spec_path = tmp_dir / "tight_spec.json"
    tight_spec = make_valid_layout_spec("tight-v1", root_size_mib=60 * 1024)
    spec_path.write_text(json.dumps(tight_spec), encoding="utf-8")
    run_aiosh(["layout", "register", "--spec", str(spec_path), "--store", str(store), "--json"])

    res = run_aiosh(["layout", "probe", "tight-v1", "--bytes", str(65 * 1024 * 1024 * 1024), "--store", str(store), "--json"])
    assert res.returncode == 0, f"probe tight-v1 failed: {res.stderr}\n{res.stdout}"
    probe = parse_json(res)["data"]
    assert probe["is_viable"] is True
    assert len(probe["warnings"]) > 0

    # Generous capacity: 128 GiB
    res = run_aiosh(["layout", "probe", "--standard", "--bytes", str(128 * 1024 * 1024 * 1024), "--store", str(store), "--json"])
    assert res.returncode == 0
    probe = parse_json(res)["data"]
    assert probe["is_viable"] is True
    assert len(probe["warnings"]) == 0
    print("  [+] A5: Probe capacity feasibility PASS")


def test_a6_diff_destructive(tmp_dir: Path) -> None:
    """A6: Differential analysis & destructive change detection."""
    store = tmp_dir / "store_a6.json"
    spec_path = tmp_dir / "spec_a6.json"
    spec_data = make_valid_layout_spec("shrunk-srv-v1", root_size_mib=20 * 1024)
    spec_path.write_text(json.dumps(spec_data), encoding="utf-8")

    run_aiosh(["layout", "register", "--spec", str(spec_path), "--store", str(store), "--json"])

    res = run_aiosh(["layout", "diff", UEFI_ID, "shrunk-srv-v1", "--store", str(store), "--json"])
    assert res.returncode == 0, f"diff failed: {res.stderr}\n{res.stdout}"
    diff = parse_json(res)["data"]
    assert diff["destructive"] is True
    print("  [+] A6: Differential analysis destructive detection PASS")


def test_a7_corrupt_store_recovery(tmp_dir: Path) -> None:
    """A7: Corrupted store tamper resistance."""
    store = tmp_dir / "corrupt_store.json"
    store.write_text('{"profiles": [ "corrupted_syntax_unclosed ...', encoding="utf-8")

    res = run_aiosh(["layout", "list", "--store", str(store), "--json"])
    assert res.returncode == 1, "expected error on corrupt store read"
    err = parse_json(res)["error"]
    assert err is not None
    assert store.read_text(encoding="utf-8").startswith('{"profiles":')
    print("  [+] A7: Corrupt store tamper resistance PASS")


def test_a8_audit_emission(tmp_dir: Path) -> None:
    """A8: SQLite WAL audit trail verification."""
    home_dir = tmp_dir / "aiosh_home"
    home_dir.mkdir(parents=True, exist_ok=True)
    env = {"AIOSH_HOME": str(home_dir)}

    res = run_aiosh(["layout", "validate", "--standard", "--json"], env=env)
    assert res.returncode == 0, f"validate failed: {res.stderr}"

    audit_db = home_dir / "audit.db"
    assert audit_db.exists(), "audit.db was not created in AIOSH_HOME"

    conn = sqlite3.connect(str(audit_db))
    cur = conn.cursor()
    cur.execute("SELECT tool, command, outcome, hash FROM audit_ring ORDER BY id DESC LIMIT 1")
    row = cur.fetchone()
    conn.close()

    assert row is not None, "no audit row found in audit_ring"
    tool, command, outcome, sha_hash = row
    assert tool == "fs_layout"
    assert command == "validate"
    assert outcome == "success"
    assert len(sha_hash) == 64, f"expected SHA-256 hex string, got {sha_hash}"
    print("  [+] A8: SQLite WAL audit trail emission PASS")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="aios_fl10_test_") as td:
        tmp_dir = Path(td)
        print("Running FL10 Automated Cases...")
        test_a1_full_lifecycle(tmp_dir)
        test_a2_builtin_protection(tmp_dir)
        test_a3_active_protection(tmp_dir)
        test_a4_fstab_cycle(tmp_dir)
        test_a5_probe_feasibility(tmp_dir)
        test_a6_diff_destructive(tmp_dir)
        test_a7_corrupt_store_recovery(tmp_dir)
        test_a8_audit_emission(tmp_dir)
        print("All FL10 automated cases passed successfully!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
