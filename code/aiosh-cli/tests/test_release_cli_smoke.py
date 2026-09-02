import subprocess
import json
import pytest
import os
import shutil

AIOSH_BIN = "cargo run -q -p aiosh-cli --"

def run_cli(*args):
    """Helper to run the CLI, assuming cargo is in path and we build dynamically for tests."""
    cmd = ["cargo", "run", "-q", "-p", "aiosh-cli", "--"] + list(args)
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, check=False)
        return res
    except Exception as e:
        pytest.fail(f"Could not run cargo: {e}")

@pytest.mark.skipif(os.name == 'nt', reason="Rust libc deps (genisoimage/waitpid) block compilation on Windows")
def test_release_generate_happy_path():
    res = run_cli("release", "generate", "--os", "testos", "--version", "1.0.0")
    assert res.returncode == 0
    try:
        data = json.loads(res.stdout)
        assert data["ok"] is True
        assert data["subcommand"] == "release generate"
        assert "artifact_path" in data["data"]
        assert "hash" in data["data"]
        assert data["data"]["artifact_path"] == "output/release/aios_testos_1.0.0.iso"
    except json.JSONDecodeError:
        pytest.fail(f"Invalid JSON output: {res.stdout}")

@pytest.mark.skipif(os.name == 'nt', reason="Rust libc deps (genisoimage/waitpid) block compilation on Windows")
def test_release_generate_missing_args():
    res = run_cli("release", "generate")
    assert res.returncode == 2
    assert "usage: aiosh release generate --os <target_os> --version <version>" in res.stderr

@pytest.mark.skipif(os.name == 'nt', reason="Rust libc deps (genisoimage/waitpid) block compilation on Windows")
def test_backup_create_happy_path(tmp_path):
    target_dir = tmp_path / "test_backup_dir"
    target_dir.mkdir()
    (target_dir / "file.txt").write_text("hello")
    
    res = run_cli("backup", "create", "--target-path", str(target_dir))
    assert res.returncode == 0
    
    try:
        data = json.loads(res.stdout)
        assert data["ok"] is True
        assert data["subcommand"] == "backup create"
        assert "backup_path" in data["data"]
    except json.JSONDecodeError:
        pytest.fail(f"Invalid JSON output: {res.stdout}")

@pytest.mark.skipif(os.name == 'nt', reason="Rust libc deps (genisoimage/waitpid) block compilation on Windows")
def test_backup_create_missing_args():
    res = run_cli("backup", "create")
    assert res.returncode == 2
    assert "usage: aiosh backup create --target-path <path>" in res.stderr
