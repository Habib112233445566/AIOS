import os
import zipfile
import pytest
from pathlib import Path
from aiosh_mcp.release import physical_generate_iso, physical_create_zip, PackageManifest, BackupSnapshot

def test_physical_generate_iso(tmp_path):
    artifact_path = str(tmp_path / "mock.iso")
    manifest = PackageManifest(target_os="linux", components=["core"], version="1.0.0")
    
    # Run the generator
    physical_generate_iso(manifest, artifact_path)
    
    # Assert file was created
    assert os.path.exists(artifact_path)
    with open(artifact_path, "rb") as f:
        assert f.read() == b"AIOS_ISO_MOCK"

def test_physical_create_zip(tmp_path):
    # Setup mock source directory with some files
    target_dir = tmp_path / "source"
    target_dir.mkdir()
    
    # Create normal file
    (target_dir / "normal.txt").write_text("normal content")
    
    # Create audit directory to test exclusion
    audit_dir = target_dir / "audit"
    audit_dir.mkdir()
    (audit_dir / "audit.log").write_text("audit content")
    
    backup_path = str(tmp_path / "backup.zip")
    snapshot = BackupSnapshot(
        target_path=str(target_dir),
        include_audit=False,
        include_memory=False
    )
    
    physical_create_zip(snapshot, backup_path)
    
    # Verify zip contents
    assert os.path.exists(backup_path)
    with zipfile.ZipFile(backup_path, "r") as zf:
        namelist = zf.namelist()
        assert "normal.txt" in namelist
        assert "audit/audit.log" not in namelist

def test_physical_create_zip_include_audit(tmp_path):
    target_dir = tmp_path / "source2"
    target_dir.mkdir()
    
    audit_dir = target_dir / "audit"
    audit_dir.mkdir()
    (audit_dir / "audit.log").write_text("audit content")
    
    backup_path = str(tmp_path / "backup2.zip")
    snapshot = BackupSnapshot(
        target_path=str(target_dir),
        include_audit=True,
        include_memory=False
    )
    
    physical_create_zip(snapshot, backup_path)
    
    with zipfile.ZipFile(backup_path, "r") as zf:
        namelist = zf.namelist()
        # On windows separator is \, but zip always uses /
        # We need to normalize or check for either just to be safe
        found_audit = False
        for name in namelist:
            if "audit" in name and "audit.log" in name:
                found_audit = True
        assert found_audit, f"Audit log not found in {namelist}"

def test_physical_create_zip_missing_source(tmp_path):
    # Should handle missing target path gracefully by creating empty zip
    backup_path = str(tmp_path / "backup_missing.zip")
    snapshot = BackupSnapshot(
        target_path=str(tmp_path / "doesnotexist"),
        include_audit=False,
        include_memory=False
    )
    
    physical_create_zip(snapshot, backup_path)
    assert os.path.exists(backup_path)
    with zipfile.ZipFile(backup_path, "r") as zf:
        assert len(zf.namelist()) == 0
