import pytest
from aiosh_mcp.release import PackageManifest, BackupSnapshot, generate_release, create_backup
import os
from aiosh_mcp.audit_client import default_db_path

@pytest.fixture(autouse=True)
def memory_db(monkeypatch):
    monkeypatch.setenv("AIOSH_HOME", "/tmp/aios_test_release_smoke")

def test_generate_release_stub():
    manifest = PackageManifest(target_os="debian-13", components=[], version="0.1.0")
    path, hash_val = generate_release(manifest, actor_id="test", constitution_rev="v1")
    assert path.startswith("output/release/aios_debian-13_0.1.0.iso")
    assert len(hash_val) == 64

def test_create_backup_stub():
    snapshot = BackupSnapshot(target_path="/", include_audit=True, include_memory=True)
    path = create_backup(snapshot, actor_id="test", constitution_rev="v1")
    assert path.startswith("aios_backup_")
    assert path.endswith(".zip")
