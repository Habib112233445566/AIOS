//! AIOS Linux Distribution Selection and Target Architecture Model.
//!
//! Models supported Linux distributions, target architectures, C libraries,
//! and evaluation metrics for AIOS base systems.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Supported Linux distribution families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistroFamily {
    Debian,
    Alpine,
    Arch,
    CustomMinimal,
}

/// Supported system init frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InitSystem {
    Systemd,
    OpenRC,
    None,
}

/// Target CPU architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchTarget {
    X86_64,
    Aarch64,
}

/// Target standard C library implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CLibrary {
    Glibc,
    Musl,
}

/// Linux distribution profile specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistroProfile {
    pub id: String,
    pub name: String,
    pub family: DistroFamily,
    pub release_version: String,
    pub init_system: InitSystem,
    pub arch: ArchTarget,
    pub c_lib: CLibrary,
    pub min_kernel_version: String,
    pub default_packages: Vec<String>,
    pub recommended: bool,
    pub justification: String,
}

impl DistroProfile {
    /// Creates a standard Debian 12 (Bookworm) minimal base profile.
    pub fn debian_12_bookworm_x86_64() -> Self {
        Self {
            id: "debian-12-minimal-x86_64".into(),
            name: "Debian GNU/Linux 12 (Bookworm) Minimal".into(),
            family: DistroFamily::Debian,
            release_version: "12.5".into(),
            init_system: InitSystem::Systemd,
            arch: ArchTarget::X86_64,
            c_lib: CLibrary::Glibc,
            min_kernel_version: "6.1.0".into(),
            default_packages: vec![
                "systemd".into(),
                "systemd-sysv".into(),
                "udev".into(),
                "dbus".into(),
                "ca-certificates".into(),
                "curl".into(),
                "sqlite3".into(),
                "python3".into(),
                "python3-venv".into(),
            ],
            recommended: true,
            justification: "Gold standard binary compatibility with Python data/AI packages (glibc), systemd cgroup v2 resource limits, security update track record.".into(),
        }
    }

    /// Creates a standard Alpine 3.19+ container/sandbox profile.
    pub fn alpine_319_x86_64() -> Self {
        Self {
            id: "alpine-319-container-x86_64".into(),
            name: "Alpine Linux 3.19 Container Base".into(),
            family: DistroFamily::Alpine,
            release_version: "3.19.1".into(),
            init_system: InitSystem::None,
            arch: ArchTarget::X86_64,
            c_lib: CLibrary::Musl,
            min_kernel_version: "6.1.0".into(),
            default_packages: vec![
                "alpine-base".into(),
                "ca-certificates".into(),
                "curl".into(),
                "sqlite".into(),
            ],
            recommended: false,
            justification: "Ultra-compact image (<10MB) for lightweight ephemeral agent worker sandboxes.".into(),
        }
    }
}

/// Evaluation score and production readiness assessment for a distro profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistroEvaluation {
    pub profile_id: String,
    pub binary_compatibility_score: f32,
    pub footprint_score: f32,
    pub security_score: f32,
    pub overall_score: f32,
    pub is_production_ready: bool,
    pub evaluated_at_utc: String,
}

impl DistroEvaluation {
    /// Evaluates a distro profile against AIOS system criteria.
    pub fn evaluate(profile: &DistroProfile) -> Self {
        let binary_compatibility_score = match profile.c_lib {
            CLibrary::Glibc => 1.0,
            CLibrary::Musl => 0.65,
        };

        let footprint_score = match profile.family {
            DistroFamily::Alpine => 1.0,
            DistroFamily::CustomMinimal => 0.9,
            DistroFamily::Debian => 0.75,
            DistroFamily::Arch => 0.6,
        };

        let security_score = match profile.family {
            DistroFamily::Debian => 0.95,
            DistroFamily::Alpine => 0.85,
            DistroFamily::Arch => 0.7,
            DistroFamily::CustomMinimal => 0.6,
        };

        let overall_score = (binary_compatibility_score * 0.4)
            + (security_score * 0.3)
            + (footprint_score * 0.3);

        let is_production_ready = overall_score >= 0.75 && binary_compatibility_score >= 0.8;

        Self {
            profile_id: profile.id.clone(),
            binary_compatibility_score,
            footprint_score,
            security_score,
            overall_score,
            is_production_ready,
            evaluated_at_utc: Utc::now().to_rfc3339(),
        }
    }
}

/// Validate structural and semver invariants for a DistroProfile.
pub fn validate_distro_profile(profile: &DistroProfile) -> Result<(), String> {
    if profile.id.trim().is_empty() {
        return Err("DistroProfile id cannot be empty".into());
    }
    if !profile.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(format!("DistroProfile id '{}' contains invalid characters", profile.id));
    }
    if profile.name.trim().is_empty() {
        return Err("DistroProfile name cannot be empty".into());
    }
    if profile.release_version.trim().is_empty() {
        return Err("DistroProfile release_version cannot be empty".into());
    }
    if profile.min_kernel_version.trim().is_empty() {
        return Err("DistroProfile min_kernel_version cannot be empty".into());
    }

    let parts: Vec<&str> = profile.min_kernel_version.split('.').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return Err(format!(
            "DistroProfile min_kernel_version '{}' is not a valid semver version",
            profile.min_kernel_version
        ));
    }
    for part in parts {
        if part.parse::<u32>().is_err() {
            return Err(format!(
                "DistroProfile min_kernel_version '{}' contains non-numeric segments",
                profile.min_kernel_version
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distro_profile_validation_and_defaults() {
        let debian = DistroProfile::debian_12_bookworm_x86_64();
        assert!(validate_distro_profile(&debian).is_ok());

        let alpine = DistroProfile::alpine_319_x86_64();
        assert!(validate_distro_profile(&alpine).is_ok());

        let mut invalid = debian.clone();
        invalid.min_kernel_version = "invalid_kernel".into();
        assert!(validate_distro_profile(&invalid).is_err());
    }

    #[test]
    fn test_distro_evaluation_scoring() {
        let debian = DistroProfile::debian_12_bookworm_x86_64();
        let eval = DistroEvaluation::evaluate(&debian);
        assert!(eval.is_production_ready);
        assert!(eval.overall_score >= 0.75);

        let alpine = DistroProfile::alpine_319_x86_64();
        let eval_alpine = DistroEvaluation::evaluate(&alpine);
        assert!(!eval_alpine.is_production_ready); // Binary compat 0.65 < 0.8
    }
}
