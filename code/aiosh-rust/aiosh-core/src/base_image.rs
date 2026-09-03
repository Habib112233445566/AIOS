//! Data model and validation logic for Linux Base Image Build subsystem.

use serde::{Deserialize, Serialize};

/// Target packaging format for the compiled Linux base image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Raw,
    Qcow2,
    Iso,
    Tarball,
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw => write!(f, "raw"),
            Self::Qcow2 => write!(f, "qcow2"),
            Self::Iso => write!(f, "iso"),
            Self::Tarball => write!(f, "tarball"),
        }
    }
}

/// Specifications for the assembled root filesystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootfsSpec {
    pub distro_id: String,
    pub architecture: String,
    pub filesystem_type: String,
    pub packages: Vec<String>,
    pub size_budget_bytes: u64,
    pub hostname: String,
}

/// Kernel, initramfs, and boot parameters for the target image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelSpec {
    pub version: String,
    pub cmdline: String,
    pub initramfs_generator: String,
}

/// Complete build specification and reproducible manifest for a Linux base image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseImageManifest {
    pub id: String,
    pub version: String,
    pub format: ImageFormat,
    pub rootfs: RootfsSpec,
    pub kernel: KernelSpec,
    pub created_at: String,
    pub artifact_path: Option<String>,
    pub artifact_sha256: Option<String>,
    pub artifact_size_bytes: Option<u64>,
}

impl BaseImageManifest {
    /// Validates internal consistency invariants I1..I6.
    pub fn validate(&self) -> Result<(), String> {
        validate_base_image_manifest(self)
    }

    /// Creates the reference canonical Debian 12 minimal base image manifest.
    pub fn debian_12_minimal(format: ImageFormat) -> Self {
        Self {
            id: format!("debian-12-minimal-{}", format),
            version: "1.0.0".into(),
            format,
            rootfs: RootfsSpec {
                distro_id: "debian-12-minimal-x86_64".into(),
                architecture: "x86_64".into(),
                filesystem_type: "ext4".into(),
                packages: vec![
                    "base-files".into(),
                    "base-passwd".into(),
                    "bash".into(),
                    "coreutils".into(),
                    "dash".into(),
                    "debian-archive-keyring".into(),
                    "diffutils".into(),
                    "dpkg".into(),
                    "findutils".into(),
                    "grep".into(),
                    "gzip".into(),
                    "init-system-helpers".into(),
                    "iproute2".into(),
                    "libc-bin".into(),
                    "login".into(),
                    "netbase".into(),
                    "sed".into(),
                    "systemd".into(),
                    "systemd-sysv".into(),
                    "tar".into(),
                    "udev".into(),
                    "util-linux".into(),
                ],
                size_budget_bytes: 1024 * 1024 * 1024, // 1 GiB ceiling
                hostname: "aios-node".into(),
            },
            kernel: KernelSpec {
                version: "6.1.0-28-amd64".into(),
                cmdline: "console=tty0 console=ttyS0,115200 quiet rw rootfstype=ext4".into(),
                initramfs_generator: "dracut".into(),
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            artifact_path: None,
            artifact_sha256: None,
            artifact_size_bytes: None,
        }
    }

    /// Creates the reference container-optimized Alpine 3.19 base image manifest.
    pub fn alpine_319_container(format: ImageFormat) -> Self {
        Self {
            id: format!("alpine-319-container-{}", format),
            version: "1.0.0".into(),
            format,
            rootfs: RootfsSpec {
                distro_id: "alpine-319-container-x86_64".into(),
                architecture: "x86_64".into(),
                filesystem_type: "squashfs".into(),
                packages: vec![
                    "alpine-baselayout".into(),
                    "alpine-keys".into(),
                    "apk-tools".into(),
                    "busybox".into(),
                    "musl".into(),
                ],
                size_budget_bytes: 256 * 1024 * 1024, // 256 MiB ceiling
                hostname: "aios-container".into(),
            },
            kernel: KernelSpec {
                version: "6.1.66-0-lts".into(),
                cmdline: "console=tty0 console=ttyS0,115200 quiet".into(),
                initramfs_generator: "mkinitfs".into(),
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            artifact_path: None,
            artifact_sha256: None,
            artifact_size_bytes: None,
        }
    }
}

/// Validates internal consistency invariants I1..I6 of a BaseImageManifest.
pub fn validate_base_image_manifest(manifest: &BaseImageManifest) -> Result<(), String> {
    if manifest.id.is_empty() {
        return Err("manifest id cannot be empty".into());
    }
    if !manifest.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(format!("manifest id '{}' must contain only lowercase alphanumeric characters and hyphens", manifest.id));
    }

    let parts: Vec<&str> = manifest.version.split('.').collect();
    if parts.len() != 3 || !parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty()) {
        return Err(format!("manifest version '{}' must be valid SemVer (e.g. 1.0.0)", manifest.version));
    }

    if manifest.rootfs.packages.is_empty() {
        return Err("rootfs packages list cannot be empty".into());
    }

    // Rootfs package name security checks
    for pkg in &manifest.rootfs.packages {
        if pkg.is_empty() || pkg.len() > 128 {
            return Err(format!("package name '{}' must be between 1 and 128 characters", pkg));
        }
        let first_char = pkg.chars().next().unwrap();
        if !first_char.is_ascii_lowercase() && !first_char.is_ascii_digit() {
            return Err(format!("package name '{}' must start with a lowercase alphanumeric character", pkg));
        }
        if !pkg.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '+' || c == '.' || c == '-') {
            return Err(format!("package name '{}' contains invalid characters", pkg));
        }
    }

    // Hostname RFC 1123 security checks
    let hostname = &manifest.rootfs.hostname;
    if hostname.is_empty() || hostname.len() > 63 {
        return Err("rootfs hostname must be between 1 and 63 characters".into());
    }
    if hostname.starts_with('-') || hostname.ends_with('-') {
        return Err("rootfs hostname cannot start or end with a hyphen".into());
    }
    if !hostname.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err("rootfs hostname must contain only lowercase alphanumeric characters and hyphens".into());
    }

    // Kernel cmdline security checks
    if manifest.kernel.cmdline.len() > 4096 {
        return Err("kernel cmdline cannot exceed 4096 characters".into());
    }
    if manifest.kernel.cmdline.chars().any(|c| c == '\0' || c == '\r' || c == '\n') {
        return Err("kernel cmdline cannot contain null or newline control characters".into());
    }

    let valid_fs = ["ext4", "squashfs", "btrfs", "erofs"];
    if !valid_fs.contains(&manifest.rootfs.filesystem_type.as_str()) {
        return Err(format!("unsupported filesystem_type '{}', expected one of {:?}", manifest.rootfs.filesystem_type, valid_fs));
    }

    if manifest.rootfs.size_budget_bytes == 0 || manifest.rootfs.size_budget_bytes > 10 * 1024 * 1024 * 1024 {
        return Err("rootfs size_budget_bytes must be between 1 byte and 10 GiB".into());
    }

    if let Some(ref hash) = manifest.artifact_sha256 {
        if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) {
            return Err("artifact_sha256 must be a 64-character lowercase hex SHA-256 string".into());
        }
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_canonical_manifest_debian_valid() {
        let manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.rootfs.architecture, "x86_64");
        assert_eq!(manifest.rootfs.filesystem_type, "ext4");
        assert!(manifest.rootfs.packages.len() >= 10);
    }

    #[test]
    fn test_canonical_manifest_alpine_valid() {
        let manifest = BaseImageManifest::alpine_319_container(ImageFormat::Tarball);
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.rootfs.filesystem_type, "squashfs");
    }

    #[test]
    fn test_invalid_manifest_id() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.id = "Debian_12_UPPERCASE".into();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_invalid_semver() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.version = "1.0".into();
        assert!(manifest.validate().is_err());

        manifest.version = "1.0.beta".into();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_empty_packages() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.rootfs.packages.clear();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_invalid_filesystem() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.rootfs.filesystem_type = "ntfs".into();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_size_budget_limits() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.rootfs.size_budget_bytes = 0;
        assert!(manifest.validate().is_err());

        manifest.rootfs.size_budget_bytes = 20 * 1024 * 1024 * 1024; // 20 GiB > 10 GiB limit
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_artifact_sha256_validation() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.artifact_sha256 = Some("invalid_short_hash".into());
        assert!(manifest.validate().is_err());

        manifest.artifact_sha256 = Some("E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855".into()); // uppercase
        assert!(manifest.validate().is_err());

        manifest.artifact_sha256 = Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into());
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_json_roundtrip() {
        let manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Qcow2);
        let serialized = serde_json::to_string(&manifest).unwrap();
        let deserialized: BaseImageManifest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(manifest, deserialized);
    }

    #[test]
    fn test_image_format_display() {
        assert_eq!(ImageFormat::Raw.to_string(), "raw");
        assert_eq!(ImageFormat::Qcow2.to_string(), "qcow2");
        assert_eq!(ImageFormat::Iso.to_string(), "iso");
        assert_eq!(ImageFormat::Tarball.to_string(), "tarball");
    }

    #[test]
    fn test_package_name_hardening() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.rootfs.packages.push("systemd; rm -rf /".into());
        assert!(manifest.validate().is_err());

        manifest.rootfs.packages.pop();
        manifest.rootfs.packages.push("-leading-dash".into());
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_hostname_hardening() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.rootfs.hostname = "-bad-hostname".into();
        assert!(manifest.validate().is_err());

        manifest.rootfs.hostname = "bad_hostname_with_underscores".into();
        assert!(manifest.validate().is_err());

        manifest.rootfs.hostname = "valid-node-1".into();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_cmdline_hardening() {
        let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        manifest.kernel.cmdline.push('\n');
        assert!(manifest.validate().is_err());

        manifest.kernel.cmdline = "console=tty0\0malicious".into();
        assert!(manifest.validate().is_err());
    }
}
