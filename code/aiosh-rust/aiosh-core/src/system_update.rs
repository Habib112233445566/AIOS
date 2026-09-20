//! System Update Mechanism Core Data Model (UPD1..UPD6).
//!
//! Provides A/B dual-slot abstractions, update manifest validation,
//! cryptographic payload digest checks, and linear state machine transitions.

use serde::{Deserialize, Serialize};

pub const MAX_UPDATE_VERSION_LEN: usize = 64;
pub const MAX_UPDATE_ID_LEN: usize = 128;
pub const MAX_UPDATE_PAYLOAD_SIZE: u64 = 10 * 1024 * 1024 * 1024; // 10 GB

pub const UPD_VALIDATION_ERROR: &str = "UPD_VALIDATION_ERROR";
pub const UPD_SLOT_ERROR: &str = "UPD_SLOT_ERROR";
pub const UPD_DIGEST_ERROR: &str = "UPD_DIGEST_ERROR";
pub const UPD_STATE_ERROR: &str = "UPD_STATE_ERROR";

/// Dual-boot update slot (UPD1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateSlot {
    SlotA,
    SlotB,
}

impl UpdateSlot {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateSlot::SlotA => "slot_a",
            UpdateSlot::SlotB => "slot_b",
        }
    }

    pub fn other(&self) -> Self {
        match self {
            UpdateSlot::SlotA => UpdateSlot::SlotB,
            UpdateSlot::SlotB => UpdateSlot::SlotA,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "slot_a" | "a" | "slota" | "slot0" => Some(UpdateSlot::SlotA),
            "slot_b" | "b" | "slotb" | "slot1" => Some(UpdateSlot::SlotB),
            _ => None,
        }
    }
}

/// Distribution release channels for system updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
    Development,
}

impl UpdateChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Beta => "beta",
            UpdateChannel::Nightly => "nightly",
            UpdateChannel::Development => "development",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "stable" | "prod" | "release" => Some(UpdateChannel::Stable),
            "beta" | "preview" | "staging" => Some(UpdateChannel::Beta),
            "nightly" | "daily" => Some(UpdateChannel::Nightly),
            "dev" | "development" => Some(UpdateChannel::Development),
            _ => None,
        }
    }
}

/// Lifecycle states of a system update execution (UPD4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    Idle,
    Checking,
    Downloading,
    Verifying,
    Applying,
    ReadyToReboot,
    Verified,
    RolledBack,
    Failed,
}

impl UpdateState {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateState::Idle => "idle",
            UpdateState::Checking => "checking",
            UpdateState::Downloading => "downloading",
            UpdateState::Verifying => "verifying",
            UpdateState::Applying => "applying",
            UpdateState::ReadyToReboot => "ready_to_reboot",
            UpdateState::Verified => "verified",
            UpdateState::RolledBack => "rolled_back",
            UpdateState::Failed => "failed",
        }
    }

    /// Evaluates valid state transitions according to UPD4 state machine.
    pub fn can_transition_to(&self, next: UpdateState) -> bool {
        match (self, next) {
            // Any active operation can transition to Failed
            (_, UpdateState::Failed) => true,
            // Idle can start checking or downloading
            (UpdateState::Idle, UpdateState::Checking) => true,
            (UpdateState::Idle, UpdateState::Downloading) => true,
            // Checking can find update (Downloading) or return to Idle
            (UpdateState::Checking, UpdateState::Downloading) => true,
            (UpdateState::Checking, UpdateState::Idle) => true,
            // Downloading transitions to Verifying
            (UpdateState::Downloading, UpdateState::Verifying) => true,
            // Verifying transitions to Applying
            (UpdateState::Verifying, UpdateState::Applying) => true,
            // Applying transitions to ReadyToReboot
            (UpdateState::Applying, UpdateState::ReadyToReboot) => true,
            // After reboot, system marks update as Verified or RolledBack
            (UpdateState::ReadyToReboot, UpdateState::Verified) => true,
            (UpdateState::ReadyToReboot, UpdateState::RolledBack) => true,
            // Terminal states can reset to Idle
            (UpdateState::Verified, UpdateState::Idle) => true,
            (UpdateState::RolledBack, UpdateState::Idle) => true,
            (UpdateState::Failed, UpdateState::Idle) => true,
            _ => false,
        }
    }
}

/// Partition or filesystem target for an individual update artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionTarget {
    Rootfs,
    Kernel,
    Initramfs,
    FullBundle,
}

/// A cryptographic payload artifact belonging to an update package (UPD3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateArtifact {
    pub target: PartitionTarget,
    pub file_name: String,
    pub sha256: String,
    pub size_bytes: u64,
}

impl UpdateArtifact {
    pub fn validate(&self) -> Result<(), String> {
        if self.file_name.trim().is_empty() {
            return Err(format!("{}: artifact file_name cannot be empty", UPD_VALIDATION_ERROR));
        }
        if self.file_name.len() > 256 || self.file_name.chars().any(|c| c.is_control()) {
            return Err(format!("{}: invalid artifact file_name '{}'", UPD_VALIDATION_ERROR, self.file_name));
        }
        if self.sha256.len() != 64 || !self.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("{}: invalid SHA-256 digest '{}'", UPD_DIGEST_ERROR, self.sha256));
        }
        if self.size_bytes == 0 || self.size_bytes > MAX_UPDATE_PAYLOAD_SIZE {
            return Err(format!("{}: invalid artifact size {} bytes", UPD_VALIDATION_ERROR, self.size_bytes));
        }
        Ok(())
    }
}

/// Complete signed update manifest containing release metadata and artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub update_id: String,
    pub version: String,
    pub channel: UpdateChannel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
    pub artifacts: Vec<UpdateArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub release_notes: String,
    pub published_at: String,
}

impl UpdateManifest {
    pub fn validate(&self) -> Result<(), String> {
        let clean_id = self.update_id.trim();
        if clean_id.is_empty() || clean_id.len() > MAX_UPDATE_ID_LEN {
            return Err(format!("{}: update_id must be between 1 and {} chars", UPD_VALIDATION_ERROR, MAX_UPDATE_ID_LEN));
        }
        let clean_ver = self.version.trim();
        if clean_ver.is_empty() || clean_ver.len() > MAX_UPDATE_VERSION_LEN {
            return Err(format!("{}: version must be between 1 and {} chars", UPD_VALIDATION_ERROR, MAX_UPDATE_VERSION_LEN));
        }
        if self.artifacts.is_empty() {
            return Err(format!("{}: manifest must contain at least one artifact", UPD_VALIDATION_ERROR));
        }
        for artifact in &self.artifacts {
            artifact.validate()?;
        }
        Ok(())
    }

    pub fn total_bytes(&self) -> u64 {
        self.artifacts.iter().map(|a| a.size_bytes).sum()
    }

    pub fn has_target(&self, target: PartitionTarget) -> bool {
        self.artifacts.iter().any(|a| a.target == target)
    }

    pub fn find_artifact(&self, target: PartitionTarget) -> Option<&UpdateArtifact> {
        self.artifacts.iter().find(|a| a.target == target)
    }
}

/// Status of physical A/B slots on the system (UPD1, UPD5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSlotStatus {
    pub current_slot: UpdateSlot,
    pub target_slot: UpdateSlot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_slot: Option<UpdateSlot>,
    pub slot_a_version: String,
    pub slot_b_version: String,
    pub slot_a_successful: bool,
    pub slot_b_successful: bool,
}

impl SystemSlotStatus {
    pub fn new(current: UpdateSlot, version: impl Into<String>) -> Self {
        let ver = version.into();
        let target = current.other();
        SystemSlotStatus {
            current_slot: current,
            target_slot: target,
            rollback_slot: Some(current),
            slot_a_version: if current == UpdateSlot::SlotA { ver.clone() } else { "none".into() },
            slot_b_version: if current == UpdateSlot::SlotB { ver } else { "none".into() },
            slot_a_successful: current == UpdateSlot::SlotA,
            slot_b_successful: current == UpdateSlot::SlotB,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.current_slot == self.target_slot {
            return Err(format!("{}: current_slot cannot be identical to target_slot", UPD_SLOT_ERROR));
        }
        Ok(())
    }

    pub fn switch_slot(&mut self) {
        self.rollback_slot = Some(self.current_slot);
        let next_slot = self.target_slot;
        self.target_slot = self.current_slot;
        self.current_slot = next_slot;
    }

    pub fn mark_slot_success(&mut self, slot: UpdateSlot, version: impl Into<String>) {
        let ver = version.into();
        match slot {
            UpdateSlot::SlotA => {
                self.slot_a_successful = true;
                self.slot_a_version = ver;
            }
            UpdateSlot::SlotB => {
                self.slot_b_successful = true;
                self.slot_b_version = ver;
            }
        }
    }
}

/// Real-time progress and state of the system update service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemUpdateStatus {
    pub state: UpdateState,
    pub current_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_version: Option<String>,
    pub active_slot: UpdateSlot,
    pub progress_percent: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub updated_at: String,
}

impl SystemUpdateStatus {
    pub fn new(current_version: impl Into<String>, active_slot: UpdateSlot, updated_at: impl Into<String>) -> Self {
        SystemUpdateStatus {
            state: UpdateState::Idle,
            current_version: current_version.into(),
            target_version: None,
            active_slot,
            progress_percent: 0,
            last_error: None,
            updated_at: updated_at.into(),
        }
    }

    pub fn transition(&mut self, next: UpdateState) -> Result<(), String> {
        if !self.state.can_transition_to(next) {
            return Err(format!(
                "{}: cannot transition from {:?} to {:?}",
                UPD_STATE_ERROR, self.state, next
            ));
        }
        self.state = next;
        if next == UpdateState::Idle {
            self.progress_percent = 0;
            self.last_error = None;
            self.target_version = None;
        }
        Ok(())
    }

    pub fn set_progress(&mut self, percent: u8) {
        self.progress_percent = percent.min(100);
    }

    pub fn set_error(&mut self, err: impl Into<String>) {
        self.state = UpdateState::Failed;
        self.last_error = Some(err.into());
    }
}
