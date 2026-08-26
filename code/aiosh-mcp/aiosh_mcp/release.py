"""Release Packaging & Backup data model implementation."""
from dataclasses import dataclass
from typing import List, Tuple
from . import audit_client

@dataclass
class PackageManifest:
    target_os: str
    components: List[str]
    version: str

@dataclass
class BackupSnapshot:
    target_path: str
    include_audit: bool
    include_memory: bool

def generate_release(manifest: PackageManifest, actor_id: str, constitution_rev: str) -> Tuple[str, str]:
    manifest_dict = {
        "target_os": manifest.target_os,
        "components": manifest.components,
        "version": manifest.version
    }
    manifest_json = audit_client.canonical(manifest_dict)
    hash_val = audit_client.sha256_hex(manifest_json)
    artifact_path = f"output/release/aios_{manifest.target_os}_{manifest.version}.iso"
    
    with audit_client.open_db() as conn:
        audit_client.write_audit_row(conn, {
            "ts": audit_client.utcnow_iso(),
            "actor": "user",
            "actor_id": actor_id,
            "tool": "aios.release.generate",
            "command": "Release Packaging",
            "args": manifest_dict,
            "target": artifact_path,
            "outcome": "success",
            "outcome_detail": f"Artifact generated with hash {hash_val}",
            "constitution_rev": constitution_rev,
            "grant_token": None,
            "c_flags": {"c1": False, "c2": False, "c3": False, "c4": False},
        })
    return (artifact_path, hash_val)

def create_backup(snapshot: BackupSnapshot, actor_id: str, constitution_rev: str) -> str:
    timestamp = audit_client.utcnow_iso().replace(":", "-").replace(".", "-")
    backup_path = f"aios_backup_{timestamp}.zip"
    
    snapshot_dict = {
        "target_path": snapshot.target_path,
        "include_audit": snapshot.include_audit,
        "include_memory": snapshot.include_memory
    }
    
    with audit_client.open_db() as conn:
        audit_client.write_audit_row(conn, {
            "ts": audit_client.utcnow_iso(),
            "actor": "user",
            "actor_id": actor_id,
            "tool": "aios.backup.create",
            "command": "System Backup",
            "args": snapshot_dict,
            "target": backup_path,
            "outcome": "success",
            "outcome_detail": f"Virtual backup snapshot created at {snapshot.target_path}",
            "constitution_rev": constitution_rev,
            "grant_token": None,
            "c_flags": {"c1": False, "c2": False, "c3": False, "c4": False},
        })
    return backup_path
