//! AIOS System Update Core Service (USVC1..USVC6).
//!
//! Coordinates A/B partition updates, artifact staging, cryptographic verification,
//! atomic boot slot switching, and automated rollback orchestration.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::system_update::{
    PartitionTarget, SystemSlotStatus, SystemUpdateStatus,
    UpdateManifest, UpdateSlot, UpdateState, MAX_UPDATE_PAYLOAD_SIZE,
    UPD_DIGEST_ERROR, UPD_SLOT_ERROR, UPD_STATE_ERROR, UPD_VALIDATION_ERROR,
};

/// Configuration options for the system update service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemUpdateServiceConfig {
    pub state_dir: PathBuf,
    pub staging_dir: PathBuf,
    pub max_payload_bytes: u64,
    pub auto_rollback_on_failure: bool,
}

impl Default for SystemUpdateServiceConfig {
    fn default() -> Self {
        Self {
            state_dir: PathBuf::from("/var/lib/aiosh/updates"),
            staging_dir: PathBuf::from("/var/lib/aiosh/updates/staging"),
            max_payload_bytes: MAX_UPDATE_PAYLOAD_SIZE,
            auto_rollback_on_failure: true,
        }
    }
}

/// Core System Update Service orchestrating the update lifecycle.
#[derive(Debug, Clone)]
pub struct SystemUpdateService {
    pub config: SystemUpdateServiceConfig,
    pub slot_status: SystemSlotStatus,
    pub update_status: SystemUpdateStatus,
    pub active_manifest: Option<UpdateManifest>,
    pub staged_artifacts: HashMap<PartitionTarget, PathBuf>,
}

impl SystemUpdateService {
    /// Initializes a new SystemUpdateService with specified version, active slot, and configuration.
    pub fn new(
        current_version: impl Into<String>,
        active_slot: UpdateSlot,
        config: SystemUpdateServiceConfig,
        timestamp: impl Into<String>,
    ) -> Self {
        let ver = current_version.into();
        let ts = timestamp.into();
        let slot_status = SystemSlotStatus::new(active_slot, &ver);
        let update_status = SystemUpdateStatus::new(&ver, active_slot, &ts);

        Self {
            config,
            slot_status,
            update_status,
            active_manifest: None,
            staged_artifacts: HashMap::new(),
        }
    }

    /// Evaluates an incoming update manifest and initiates checking/downloading lifecycle (USVC1, USVC3).
    pub fn check_manifest(&mut self, manifest: UpdateManifest) -> Result<(), String> {
        manifest.validate()?;

        if self.update_status.state != UpdateState::Idle {
            return Err(format!(
                "{}: cannot check manifest while state is {:?}",
                UPD_STATE_ERROR, self.update_status.state
            ));
        }

        self.update_status.transition(UpdateState::Checking)?;
        self.update_status.target_version = Some(manifest.version.clone());

        // Ensure staging directory exists
        if !self.config.staging_dir.exists() {
            fs::create_dir_all(&self.config.staging_dir)
                .map_err(|e| format!("{}: failed to create staging directory: {}", UPD_VALIDATION_ERROR, e))?;
        }

        self.active_manifest = Some(manifest);
        self.staged_artifacts.clear();
        self.update_status.transition(UpdateState::Downloading)?;
        self.update_status.set_progress(10);
        Ok(())
    }

    /// Stages a single artifact payload, computing and verifying its cryptographic SHA-256 digest (USVC2, USVC5).
    pub fn stage_artifact(&mut self, target: PartitionTarget, data: &[u8]) -> Result<PathBuf, String> {
        if self.update_status.state != UpdateState::Downloading {
            return Err(format!(
                "{}: cannot stage artifact in state {:?}",
                UPD_STATE_ERROR, self.update_status.state
            ));
        }

        let manifest = self.active_manifest.as_ref().ok_or_else(|| {
            format!("{}: no active update manifest", UPD_VALIDATION_ERROR)
        })?;

        let declared_artifact = manifest.find_artifact(target).ok_or_else(|| {
            format!("{}: target '{:?}' not found in manifest", UPD_VALIDATION_ERROR, target)
        })?;

        if (data.len() as u64) != declared_artifact.size_bytes {
            return Err(format!(
                "{}: artifact size mismatch for {:?}: declared {} bytes, received {} bytes",
                UPD_VALIDATION_ERROR, target, declared_artifact.size_bytes, data.len()
            ));
        }

        // Verify SHA-256 digest
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest_bytes = hasher.finalize();
        let computed_digest: String = digest_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        if !computed_digest.eq_ignore_ascii_case(&declared_artifact.sha256) {
            self.fail(format!(
                "{}: SHA-256 mismatch for {:?}: expected {}, got {}",
                UPD_DIGEST_ERROR, target, declared_artifact.sha256, computed_digest
            ))?;
            return Err(format!("{}: digest verification failed", UPD_DIGEST_ERROR));
        }

        // Write artifact to staging directory
        let dest_path = self.config.staging_dir.join(&declared_artifact.file_name);
        fs::write(&dest_path, data)
            .map_err(|e| format!("{}: failed to write artifact to {:?}: {}", UPD_VALIDATION_ERROR, dest_path, e))?;

        self.staged_artifacts.insert(target, dest_path.clone());

        let total = manifest.artifacts.len();
        let staged = self.staged_artifacts.len();
        let progress = 10 + ((staged as u8 * 50) / total as u8);
        self.update_status.set_progress(progress);

        Ok(dest_path)
    }

    /// Verifies all declared artifacts are staged and transitions to Verifying state (USVC2).
    pub fn verify_staged(&mut self) -> Result<(), String> {
        let manifest = self.active_manifest.as_ref().ok_or_else(|| {
            format!("{}: no active update manifest", UPD_VALIDATION_ERROR)
        })?;

        for artifact in &manifest.artifacts {
            if !self.staged_artifacts.contains_key(&artifact.target) {
                return Err(format!(
                    "{}: missing staged artifact for partition target {:?}",
                    UPD_VALIDATION_ERROR, artifact.target
                ));
            }
        }

        self.update_status.transition(UpdateState::Verifying)?;
        self.update_status.set_progress(80);
        Ok(())
    }

    /// Applies the update by validating inactive target slot and toggling boot indicator (USVC3).
    pub fn apply_update(&mut self) -> Result<UpdateSlot, String> {
        if self.update_status.state != UpdateState::Verifying {
            return Err(format!(
                "{}: cannot apply update from state {:?}",
                UPD_STATE_ERROR, self.update_status.state
            ));
        }

        self.slot_status.validate()?;

        self.update_status.transition(UpdateState::Applying)?;
        self.update_status.set_progress(95);

        // Switch to target slot for next boot
        self.slot_status.switch_slot();
        let next_boot_slot = self.slot_status.current_slot;

        self.update_status.active_slot = next_boot_slot;
        self.update_status.transition(UpdateState::ReadyToReboot)?;
        self.update_status.set_progress(100);

        Ok(next_boot_slot)
    }

    /// Confirms successful boot of the new version on the updated slot.
    pub fn confirm_boot(&mut self, running_version: &str) -> Result<(), String> {
        if self.update_status.state != UpdateState::ReadyToReboot {
            return Err(format!(
                "{}: cannot confirm boot from state {:?}",
                UPD_STATE_ERROR, self.update_status.state
            ));
        }

        let active_slot = self.slot_status.current_slot;
        self.slot_status.mark_slot_success(active_slot, running_version);
        self.update_status.current_version = running_version.to_string();

        self.update_status.transition(UpdateState::Verified)?;
        self.update_status.transition(UpdateState::Idle)?;
        self.active_manifest = None;
        self.staged_artifacts.clear();

        Ok(())
    }

    /// Executes rollback to the previous functional slot (UPD5).
    pub fn rollback(&mut self) -> Result<UpdateSlot, String> {
        let fallback = self.slot_status.rollback_slot.ok_or_else(|| {
            format!("{}: no rollback slot available", UPD_SLOT_ERROR)
        })?;

        self.slot_status.current_slot = fallback;
        self.slot_status.target_slot = fallback.other();
        self.update_status.active_slot = fallback;

        self.update_status.transition(UpdateState::RolledBack)?;
        self.update_status.transition(UpdateState::Idle)?;
        self.active_manifest = None;
        self.staged_artifacts.clear();

        Ok(fallback)
    }

    /// Marks current update attempt as failed and transitions to Failed state.
    pub fn fail(&mut self, reason: impl Into<String>) -> Result<(), String> {
        let err_msg = reason.into();
        self.update_status.set_error(err_msg);
        Ok(())
    }

    /// Atomically persists slot status and update status to directory (USVC4).
    pub fn save_state_to_dir(&self, dir: &Path) -> Result<(), String> {
        if !dir.exists() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("failed to create state dir {:?}: {}", dir, e))?;
        }

        let slot_path = dir.join("slot_status.json");
        let slot_tmp = dir.join("slot_status.json.tmp");
        let slot_json = serde_json::to_string_pretty(&self.slot_status)
            .map_err(|e| format!("failed to serialize slot status: {}", e))?;
        fs::write(&slot_tmp, slot_json.as_bytes())
            .map_err(|e| format!("failed to write {:?}: {}", slot_tmp, e))?;
        fs::rename(&slot_tmp, &slot_path)
            .map_err(|e| format!("failed to atomic rename to {:?}: {}", slot_path, e))?;

        let update_path = dir.join("update_status.json");
        let update_tmp = dir.join("update_status.json.tmp");
        let update_json = serde_json::to_string_pretty(&self.update_status)
            .map_err(|e| format!("failed to serialize update status: {}", e))?;
        fs::write(&update_tmp, update_json.as_bytes())
            .map_err(|e| format!("failed to write {:?}: {}", update_tmp, e))?;
        fs::rename(&update_tmp, &update_path)
            .map_err(|e| format!("failed to atomic rename to {:?}: {}", update_path, e))?;

        Ok(())
    }

    /// Creates a SystemUpdateService using default configuration.
    pub fn with_defaults(
        current_version: impl Into<String>,
        active_slot: UpdateSlot,
        timestamp: impl Into<String>,
    ) -> Self {
        Self::new(current_version, active_slot, SystemUpdateServiceConfig::default(), timestamp)
    }

    /// Loads persisted slot status and update status from a state directory.
    pub fn load_state_from_dir(dir: &Path, config: SystemUpdateServiceConfig) -> Result<Self, String> {
        let slot_path = dir.join("slot_status.json");
        let update_path = dir.join("update_status.json");

        if !slot_path.exists() || !update_path.exists() {
            return Err(format!("state files not found in {:?}", dir));
        }

        let slot_bytes = fs::read(&slot_path)
            .map_err(|e| format!("failed to read slot status {:?}: {}", slot_path, e))?;
        let slot_status: SystemSlotStatus = serde_json::from_slice(&slot_bytes)
            .map_err(|e| format!("failed to parse slot status: {}", e))?;

        let update_bytes = fs::read(&update_path)
            .map_err(|e| format!("failed to read update status {:?}: {}", update_path, e))?;
        let update_status: SystemUpdateStatus = serde_json::from_slice(&update_bytes)
            .map_err(|e| format!("failed to parse update status: {}", e))?;

        Ok(Self {
            config,
            slot_status,
            update_status,
            active_manifest: None,
            staged_artifacts: HashMap::new(),
        })
    }

    /// Purges all staged files in the staging directory.
    pub fn clean_staging(&mut self) -> Result<(), String> {
        if self.config.staging_dir.exists() {
            fs::remove_dir_all(&self.config.staging_dir)
                .map_err(|e| format!("failed to clean staging dir: {}", e))?;
            fs::create_dir_all(&self.config.staging_dir)
                .map_err(|e| format!("failed to re-create staging dir: {}", e))?;
        }
        self.staged_artifacts.clear();
        Ok(())
    }
}
