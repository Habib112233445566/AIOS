//! Security policy enforcement for Linux Base Image Build subsystem.

use serde::{Deserialize, Serialize};
use crate::base_image::BaseImageManifest;
use crate::base_image_service::ImageStore;

/// Enforcement mode for base image security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaseImagePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

impl Default for BaseImagePolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Security policy defining mandatory security criteria for base image manifests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseImageSecurityPolicy {
    pub mode: BaseImagePolicyMode,
    pub prohibited_kernel_params: Vec<String>,
    pub prohibited_packages: Vec<String>,
    pub allowed_architectures: Vec<String>,
    pub allowed_filesystems: Vec<String>,
    pub require_core_packages: bool,
}

impl Default for BaseImageSecurityPolicy {
    fn default() -> Self {
        Self {
            mode: BaseImagePolicyMode::Enforcing,
            prohibited_kernel_params: vec![
                "nokaslr".into(),
                "mitigations=off".into(),
                "pti=off".into(),
                "selinux=0".into(),
                "apparmor=0".into(),
                "enforcing=0".into(),
                "init=/bin/sh".into(),
                "init=/bin/bash".into(),
                "init=/bin/dash".into(),
                "single".into(),
                "emergency".into(),
            ],
            prohibited_packages: vec![
                "telnet".into(),
                "rsh-client".into(),
                "rsh-server".into(),
                "rlogin".into(),
                "rexec".into(),
                "nis".into(),
                "yp-tools".into(),
            ],
            allowed_architectures: vec![
                "x86_64".into(),
                "aarch64".into(),
                "riscv64".into(),
            ],
            allowed_filesystems: vec![
                "ext4".into(),
                "squashfs".into(),
                "btrfs".into(),
                "erofs".into(),
                "xfs".into(),
            ],
            require_core_packages: true,
        }
    }
}

/// A specific security violation found during policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseImagePolicyViolation {
    pub rule_id: String,
    pub description: String,
    pub fatal: bool,
}

/// Complete report of policy evaluation against a base image manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseImagePolicyVerdict {
    pub manifest_id: String,
    pub allowed: bool,
    pub mode: BaseImagePolicyMode,
    pub violations: Vec<BaseImagePolicyViolation>,
    pub evaluated_at: String,
}

impl BaseImageSecurityPolicy {
    /// Validates policy configuration invariants and enforces resource limits.
    pub fn validate(&self) -> Result<(), String> {
        if self.allowed_architectures.is_empty() {
            return Err("allowed_architectures cannot be empty".into());
        }
        if self.allowed_architectures.len() > 64 {
            return Err("allowed_architectures exceeds maximum of 64 entries".into());
        }
        for arch in &self.allowed_architectures {
            if arch.is_empty() || arch.len() > 64 || arch.chars().any(|c| c.is_control()) {
                return Err(format!("invalid architecture entry: '{}'", arch));
            }
        }

        if self.allowed_filesystems.is_empty() {
            return Err("allowed_filesystems cannot be empty".into());
        }
        if self.allowed_filesystems.len() > 64 {
            return Err("allowed_filesystems exceeds maximum of 64 entries".into());
        }
        for fs in &self.allowed_filesystems {
            if fs.is_empty() || fs.len() > 64 || fs.chars().any(|c| c.is_control()) {
                return Err(format!("invalid filesystem entry: '{}'", fs));
            }
        }

        if self.prohibited_packages.len() > 1024 {
            return Err("prohibited_packages exceeds maximum of 1024 entries".into());
        }
        for pkg in &self.prohibited_packages {
            if pkg.is_empty() || pkg.len() > 128 || pkg.chars().any(|c| c.is_control()) {
                return Err(format!("invalid prohibited package entry: '{}'", pkg));
            }
        }

        if self.prohibited_kernel_params.len() > 1024 {
            return Err("prohibited_kernel_params exceeds maximum of 1024 entries".into());
        }
        for param in &self.prohibited_kernel_params {
            if param.is_empty() || param.len() > 256 || param.chars().any(|c| c.is_control()) {
                return Err(format!("invalid prohibited kernel parameter entry: '{}'", param));
            }
        }

        Ok(())
    }

    /// Loads security policy with environment variable overrides.
    pub fn from_env() -> Result<Self, String> {
        Self::from_source(|k| std::env::var(k).ok())
    }

    /// Loads security policy from a provider closure.
    pub fn from_source<F>(get: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let mut policy = Self::default();

        if let Some(mode_str) = get("AIOSH_BASE_IMAGE_POLICY_MODE") {
            match mode_str.trim().to_lowercase().as_str() {
                "enforcing" => policy.mode = BaseImagePolicyMode::Enforcing,
                "audit" => policy.mode = BaseImagePolicyMode::Audit,
                "permissive" => policy.mode = BaseImagePolicyMode::Permissive,
                other => {
                    return Err(format!(
                        "invalid AIOSH_BASE_IMAGE_POLICY_MODE: '{}' must be 'enforcing', 'audit', or 'permissive'",
                        other
                    ));
                }
            }
        }

        if let Some(pkgs_str) = get("AIOSH_BASE_IMAGE_PROHIBITED_PACKAGES") {
            policy.prohibited_packages = pkgs_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        if let Some(params_str) = get("AIOSH_BASE_IMAGE_PROHIBITED_KERNEL_PARAMS") {
            policy.prohibited_kernel_params = params_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        if let Some(archs_str) = get("AIOSH_BASE_IMAGE_ALLOWED_ARCH") {
            policy.allowed_architectures = archs_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        if let Some(fs_str) = get("AIOSH_BASE_IMAGE_ALLOWED_FS") {
            policy.allowed_filesystems = fs_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        policy.validate()?;
        Ok(policy)
    }

    /// Evaluates a base image manifest against the security policy.
    pub fn evaluate(&self, manifest: &BaseImageManifest) -> BaseImagePolicyVerdict {
        let mut violations = Vec::new();

        if self.mode == BaseImagePolicyMode::Permissive {
            return BaseImagePolicyVerdict {
                manifest_id: manifest.id.clone(),
                allowed: true,
                mode: self.mode,
                violations,
                evaluated_at: "2026-09-04T00:00:00Z".into(),
            };
        }

        // P0: Poisoning and control character check
        if manifest.kernel.cmdline.chars().any(|c| c.is_control()) {
            violations.push(BaseImagePolicyViolation {
                rule_id: "P0_MALFORMED_INPUT".into(),
                description: "Kernel command line contains forbidden control characters".into(),
                fatal: true,
            });
        }
        for pkg in &manifest.rootfs.packages {
            if pkg.chars().any(|c| c.is_control()) {
                violations.push(BaseImagePolicyViolation {
                    rule_id: "P0_MALFORMED_INPUT".into(),
                    description: format!("Package name contains forbidden control characters: '{}'", pkg),
                    fatal: true,
                });
            }
        }

        // P1 & P2 & P3: Kernel Cmdline checks
        let cmdline_parts: Vec<&str> = manifest.kernel.cmdline.split_whitespace().collect();
        for param in &cmdline_parts {
            for prohibited in &self.prohibited_kernel_params {
                if param == prohibited || param.starts_with(&format!("{}=", prohibited)) {
                    violations.push(BaseImagePolicyViolation {
                        rule_id: "P1_P3_KERNEL_PARAM".into(),
                        description: format!("Prohibited kernel parameter detected: '{}'", param),
                        fatal: self.mode == BaseImagePolicyMode::Enforcing,
                    });
                }
            }
        }

        // P4: Prohibited packages check
        for pkg in &manifest.rootfs.packages {
            if self.prohibited_packages.iter().any(|p| p.eq_ignore_ascii_case(pkg)) {
                violations.push(BaseImagePolicyViolation {
                    rule_id: "P4_PROHIBITED_PACKAGE".into(),
                    description: format!("Prohibited package detected: '{}'", pkg),
                    fatal: self.mode == BaseImagePolicyMode::Enforcing,
                });
            }
        }

        // P5: Architecture whitelist check
        if !self.allowed_architectures.iter().any(|a| a.eq_ignore_ascii_case(&manifest.rootfs.architecture)) {
            violations.push(BaseImagePolicyViolation {
                rule_id: "P5_ARCHITECTURE_WHITELIST".into(),
                description: format!("Architecture '{}' not in approved whitelist", manifest.rootfs.architecture),
                fatal: self.mode == BaseImagePolicyMode::Enforcing,
            });
        }

        // P6: Filesystem whitelist check
        if !self.allowed_filesystems.iter().any(|fs| fs.eq_ignore_ascii_case(&manifest.rootfs.filesystem_type)) {
            violations.push(BaseImagePolicyViolation {
                rule_id: "P6_FILESYSTEM_WHITELIST".into(),
                description: format!("Filesystem type '{}' not in approved whitelist", manifest.rootfs.filesystem_type),
                fatal: self.mode == BaseImagePolicyMode::Enforcing,
            });
        }

        // Mandatory core packages check (for distributions with packages list)
        if self.require_core_packages && !manifest.rootfs.packages.is_empty() {
            let has_base = manifest.rootfs.packages.iter().any(|p| p == "base-files" || p == "alpine-baselayout");
            if !has_base {
                violations.push(BaseImagePolicyViolation {
                    rule_id: "P7_MANDATORY_PACKAGE".into(),
                    description: "Missing mandatory system base package ('base-files' or 'alpine-baselayout')".into(),
                    fatal: self.mode == BaseImagePolicyMode::Enforcing,
                });
            }
        }

        let allowed = if self.mode == BaseImagePolicyMode::Enforcing {
            violations.is_empty()
        } else {
            true
        };

        BaseImagePolicyVerdict {
            manifest_id: manifest.id.clone(),
            allowed,
            mode: self.mode,
            violations,
            evaluated_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    /// Evaluates all images in an ImageStore against this security policy.
    pub fn check_all(&self, store: &ImageStore) -> Vec<BaseImagePolicyVerdict> {
        store
            .list_images()
            .iter()
            .map(|manifest| self.evaluate(manifest))
            .collect()
    }

    /// Returns only image manifests that pass this security policy.
    pub fn filter_compliant_manifests(&self, store: &ImageStore) -> Vec<BaseImageManifest> {
        store
            .list_images()
            .into_iter()
            .filter(|manifest| self.evaluate(manifest).allowed)
            .collect()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::base_image::ImageFormat;

    #[test]
    fn test_base_image_policy_default_validation() {
        let policy = BaseImageSecurityPolicy::default();
        assert!(policy.validate().is_ok());

        let mut invalid_policy = BaseImageSecurityPolicy::default();
        invalid_policy.allowed_architectures.clear();
        assert!(invalid_policy.validate().is_err());
    }

    #[test]
    fn test_base_image_policy_passes_reference_debian() {
        let policy = BaseImageSecurityPolicy::default();
        let debian = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        let verdict = policy.evaluate(&debian);
        assert!(verdict.allowed);
        assert!(verdict.violations.is_empty());
    }

    #[test]
    fn test_base_image_policy_prohibited_kernel_param() {
        let policy = BaseImageSecurityPolicy::default();
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.kernel.cmdline.push_str(" nokaslr mitigations=off");

        let verdict = policy.evaluate(&manifest);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "P1_P3_KERNEL_PARAM"));
    }

    #[test]
    fn test_base_image_policy_prohibited_package() {
        let policy = BaseImageSecurityPolicy::default();
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.rootfs.packages.push("telnet".into());

        let verdict = policy.evaluate(&manifest);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "P4_PROHIBITED_PACKAGE"));
    }

    #[test]
    fn test_base_image_policy_audit_mode() {
        let mut policy = BaseImageSecurityPolicy::default();
        policy.mode = BaseImagePolicyMode::Audit;

        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.rootfs.packages.push("telnet".into());

        let verdict = policy.evaluate(&manifest);
        assert!(verdict.allowed); // Allowed in audit mode
        assert!(!verdict.violations.is_empty());
        assert!(!verdict.violations[0].fatal);
    }

    #[test]
    fn test_base_image_policy_store_check_all() {
        let policy = BaseImageSecurityPolicy::default();
        let store = ImageStore::new();
        let verdicts = policy.check_all(&store);
        assert!(!verdicts.is_empty());
        assert!(verdicts.iter().all(|v| v.allowed));

        let compliant = policy.filter_compliant_manifests(&store);
        assert_eq!(compliant.len(), store.list_images().len());
    }

    #[test]
    fn test_base_image_policy_hardening_bounds_and_poisoning() {
        let policy = BaseImageSecurityPolicy::default();

        // Control char in cmdline
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.kernel.cmdline.push('\x07');
        let v1 = policy.evaluate(&manifest);
        assert!(!v1.allowed);
        assert!(v1.violations.iter().any(|v| v.rule_id == "P0_MALFORMED_INPUT"));

        // Control char in package name
        let mut manifest2 = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest2.rootfs.packages.push("bad\x00pkg".into());
        let v2 = policy.evaluate(&manifest2);
        assert!(!v2.allowed);
        assert!(v2.violations.iter().any(|v| v.rule_id == "P0_MALFORMED_INPUT"));

        // Oversized policy limits
        let mut bad_policy = BaseImageSecurityPolicy::default();
        bad_policy.allowed_architectures = (0..65).map(|i| format!("arch_{}", i)).collect();
        assert!(bad_policy.validate().is_err());

        let mut bad_policy2 = BaseImageSecurityPolicy::default();
        bad_policy2.prohibited_packages = (0..1025).map(|i| format!("pkg_{}", i)).collect();
        assert!(bad_policy2.validate().is_err());
    }
}
