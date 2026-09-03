#!/usr/bin/env python3
"""CLI Smoke & Invariant Test for Distro Selection & Justification (T-01023..T-01025)."""

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

def get_binary_path():
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

def run_aiosh(*args):
    bin_path = get_binary_path()
    cmd = [bin_path, *args]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    return res

def parse_json_output(res):
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)

def test_distro_list_prose():
    res = run_aiosh("distro", "list")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    assert "debian-12-minimal-x86_64" in res.stdout
    assert "alpine-319-container-x86_64" in res.stdout
    assert "Total distro profiles: 2" in res.stdout
    print("PASS: aiosh distro list prose")

def test_distro_list_json():
    res = run_aiosh("distro", "list", "--json")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    data = parse_json_output(res)
    assert isinstance(data, list)
    assert len(data) >= 2
    ids = [item["id"] for item in data]
    assert "debian-12-minimal-x86_64" in ids
    assert "alpine-319-container-x86_64" in ids
    print("PASS: aiosh distro list --json")

def test_distro_show_prose():
    res = run_aiosh("distro", "show", "debian-12-minimal-x86_64")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    assert "Debian GNU/Linux 12 (Bookworm) Minimal" in res.stdout
    assert "Systemd" in res.stdout
    print("PASS: aiosh distro show prose")

def test_distro_show_json():
    res = run_aiosh("distro", "show", "debian-12-minimal-x86_64", "--json")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    data = parse_json_output(res)
    assert data["id"] == "debian-12-minimal-x86_64"
    assert data["family"] == "Debian"
    assert data["recommended"] is True
    print("PASS: aiosh distro show --json")

def test_distro_evaluate_all():
    res = run_aiosh("distro", "evaluate", "--json")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    data = parse_json_output(res)
    assert isinstance(data, list)
    assert len(data) >= 2
    assert data[0]["overall_score"] >= data[1]["overall_score"]
    print("PASS: aiosh distro evaluate --json")

def test_distro_evaluate_single():
    res = run_aiosh("distro", "evaluate", "alpine-319-container-x86_64", "--json")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    data = parse_json_output(res)
    assert data["profile_id"] == "alpine-319-container-x86_64"
    assert "binary_compatibility_score" in data
    assert "overall_score" in data
    print("PASS: aiosh distro evaluate <id> --json")

def test_distro_recommend():
    res = run_aiosh("distro", "recommend", "--json")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    data = parse_json_output(res)
    assert data["id"] == "debian-12-minimal-x86_64"
    assert data["recommended"] is True
    print("PASS: aiosh distro recommend --json")

def test_distro_help():
    res = run_aiosh("distro", "--help")
    assert res.returncode == 0
    assert "aiosh distro" in res.stdout
    print("PASS: aiosh distro --help")

def test_distro_missing_id():
    res = run_aiosh("distro", "show")
    assert res.returncode == 2
    print("PASS: aiosh distro show missing id returns 2")

def test_distro_not_found():
    res = run_aiosh("distro", "show", "nonexistent-distro")
    assert res.returncode == 1
    print("PASS: aiosh distro show nonexistent returns 1")

def test_distro_config_prose():
    res = run_aiosh("distro", "config")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    assert "AIOS Distro Configuration:" in res.stdout
    assert "Store Path:" in res.stdout
    print("PASS: aiosh distro config (prose)")

def test_distro_config_json():
    res = run_aiosh("distro", "config", "--json")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    data = parse_json_output(res)
    assert "store_path" in data
    assert "pinned_reference_id" in data
    assert "weights" in data
    print("PASS: aiosh distro config --json")

def main():
    test_distro_list_prose()
    test_distro_list_json()
    test_distro_show_prose()
    test_distro_show_json()
    test_distro_evaluate_all()
    test_distro_evaluate_single()
    test_distro_recommend()
    test_distro_config_prose()
    test_distro_config_json()
    test_distro_help()
    test_distro_missing_id()
    test_distro_not_found()
    print("\nALL DISTRO CLI SMOKE TESTS PASSED!")
    return 0

if __name__ == "__main__":
    sys.exit(main())
