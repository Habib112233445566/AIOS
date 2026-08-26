import pytest
from aiosh_mcp.release import PackageManifest, BackupSnapshot, generate_release, create_backup

def test_generate_release_stub():
    manifest = PackageManifest(target_os="debian-13", components=[], version="0.1.0")
    with pytest.raises(NotImplementedError):
        generate_release(manifest)

def test_create_backup_stub():
    snapshot = BackupSnapshot(target_path="/", include_audit=True, include_memory=True)
    with pytest.raises(NotImplementedError):
        create_backup(snapshot)
