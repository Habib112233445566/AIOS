use serde::{Deserialize, Serialize};
use crate::audit::{AuditRing, AuditRowInput, CFlags};
use crate::canonical::{sha256_hex, utcnow_iso};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub target_os: String,
    pub components: Vec<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSnapshot {
    pub target_path: String,
    pub include_audit: bool,
    pub include_memory: bool,
}

pub struct ReleaseCtx<'a> {
    pub ring: &'a mut AuditRing,
    pub actor_id: &'a str,
    pub constitution_rev: &'a str,
}

pub fn generate_release(ctx: &mut ReleaseCtx, manifest: &PackageManifest) -> Result<(String, String), String> {
    let manifest_json = serde_json::to_string(manifest).map_err(|e| e.to_string())?;
    let hash = sha256_hex(&manifest_json);
    let artifact_path = format!("output/release/aios_{}_{}.iso", manifest.target_os, manifest.version);
    
    let args = json!({
        "target_os": manifest.target_os,
        "components": manifest.components,
        "version": manifest.version,
    });
    
    ctx.ring.write(AuditRowInput {
        ts: utcnow_iso(),
        actor: "user".into(),
        actor_id: ctx.actor_id.into(),
        tool: "aios.release.generate".into(),
        command: "Release Packaging".into(),
        args,
        target: Some(artifact_path.clone()),
        outcome: "success".into(),
        outcome_detail: Some(format!("Artifact generated with hash {}", hash)),
        constitution_rev: Some(ctx.constitution_rev.into()),
        grant_token: None,
        c_flags: CFlags { c1: 0, c2: 0, c3: 0, c4: 0 },
        policy_revision: None,
        classify_rule_ids: None,
        classify_evidence: None,
        classify_overall_verdict: None,
        classify_verdict_reason: None,
    }).map_err(|e| e.to_string())?;

    Ok((artifact_path, hash))
}

pub fn create_backup(ctx: &mut ReleaseCtx, snapshot: &BackupSnapshot) -> Result<String, String> {
    let timestamp = utcnow_iso().replace(":", "-").replace(".", "-");
    let backup_path = format!("aios_backup_{}.zip", timestamp);
    
    let args = json!({
        "target_path": snapshot.target_path,
        "include_audit": snapshot.include_audit,
        "include_memory": snapshot.include_memory,
    });
    
    ctx.ring.write(AuditRowInput {
        ts: utcnow_iso(),
        actor: "user".into(),
        actor_id: ctx.actor_id.into(),
        tool: "aios.backup.create".into(),
        command: "System Backup".into(),
        args,
        target: Some(backup_path.clone()),
        outcome: "success".into(),
        outcome_detail: Some(format!("Virtual backup snapshot created at {}", snapshot.target_path)),
        constitution_rev: Some(ctx.constitution_rev.into()),
        grant_token: None,
        c_flags: CFlags { c1: 0, c2: 0, c3: 0, c4: 0 },
        policy_revision: None,
        classify_rule_ids: None,
        classify_evidence: None,
        classify_overall_verdict: None,
        classify_verdict_reason: None,
    }).map_err(|e| e.to_string())?;

    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::OpenOptions;

    #[test]
    fn test_generate_release_stub() {
        let mut ring = AuditRing::open(OpenOptions::memory()).unwrap();
        let mut ctx = ReleaseCtx {
            ring: &mut ring,
            actor_id: "test",
            constitution_rev: "v1",
        };
        let manifest = PackageManifest {
            target_os: "debian-13".to_string(),
            components: vec![],
            version: "0.1.0".to_string(),
        };
        let res = generate_release(&mut ctx, &manifest).unwrap();
        assert!(res.0.starts_with("output/release/"));
    }

    #[test]
    fn test_create_backup_stub() {
        let mut ring = AuditRing::open(OpenOptions::memory()).unwrap();
        let mut ctx = ReleaseCtx {
            ring: &mut ring,
            actor_id: "test",
            constitution_rev: "v1",
        };
        let snapshot = BackupSnapshot {
            target_path: "/".to_string(),
            include_audit: true,
            include_memory: true,
        };
        let res = create_backup(&mut ctx, &snapshot).unwrap();
        assert!(res.starts_with("aios_backup_"));
    }
}
