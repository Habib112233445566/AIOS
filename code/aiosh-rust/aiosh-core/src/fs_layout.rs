//! Data model and validation logic for the Filesystem Layout subsystem.

use serde::{Deserialize, Serialize};

/// Target filesystem type classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum FsType {
    Ext4,
    Btrfs,
    Xfs,
    Vfat,
    Tmpfs,
    Devtmpfs,
    Procfs,
    Sysfs,
    Overlayfs,
    Squashfs,
    Swap,
    Custom(String),
}

impl std::fmt::Display for FsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ext4 => write!(f, "ext4"),
            Self::Btrfs => write!(f, "btrfs"),
            Self::Xfs => write!(f, "xfs"),
            Self::Vfat => write!(f, "vfat"),
            Self::Tmpfs => write!(f, "tmpfs"),
            Self::Devtmpfs => write!(f, "devtmpfs"),
            Self::Procfs => write!(f, "proc"),
            Self::Sysfs => write!(f, "sysfs"),
            Self::Overlayfs => write!(f, "overlay"),
            Self::Squashfs => write!(f, "squashfs"),
            Self::Swap => write!(f, "swap"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for FsType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.trim().to_lowercase();
        Ok(match norm.as_str() {
            "ext4" => Self::Ext4,
            "btrfs" => Self::Btrfs,
            "xfs" => Self::Xfs,
            "vfat" | "fat32" | "msdos" => Self::Vfat,
            "tmpfs" => Self::Tmpfs,
            "devtmpfs" => Self::Devtmpfs,
            "proc" | "procfs" => Self::Procfs,
            "sysfs" => Self::Sysfs,
            "overlay" | "overlayfs" => Self::Overlayfs,
            "squashfs" => Self::Squashfs,
            "swap" => Self::Swap,
            _ => Self::Custom(s.trim().to_string()),
        })
    }
}

/// Partition type classification for GPT partition tables.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PartitionType {
    /// EFI System Partition (ESP). GPT GUID: c12a7328-f81f-11d2-ba4b-00a0c93ec93b
    EfiSystem,
    /// Linux x86-64 Root filesystem. GPT GUID: 4f68bce3-e8cd-4db1-96e7-fbcaf984b709
    LinuxRoot,
    /// Linux /home filesystem. GPT GUID: 933ac7e1-2eb4-4f13-b844-0e14e2aef915
    LinuxHome,
    /// Linux Swap partition. GPT GUID: 0657fd6d-a4ab-43c4-84e5-0933c84b4f4f
    LinuxSwap,
    /// Linux /var partition. GPT GUID: 4d21b016-b534-45c2-a9fb-5c16e091fd2d
    LinuxVar,
    /// Generic Linux partition (0x8300 / 0FC63DAF-8483-4772-8E79-3D69D8477DE4)
    LinuxGeneric,
    /// Custom partition type GUID
    Custom(String),
}

impl PartitionType {
    /// Returns the canonical GPT partition type GUID string.
    pub fn gpt_type_guid(&self) -> &str {
        match self {
            Self::EfiSystem => "c12a7328-f81f-11d2-ba4b-00a0c93ec93b",
            Self::LinuxRoot => "4f68bce3-e8cd-4db1-96e7-fbcaf984b709",
            Self::LinuxHome => "933ac7e1-2eb4-4f13-b844-0e14e2aef915",
            Self::LinuxSwap => "0657fd6d-a4ab-43c4-84e5-0933c84b4f4f",
            Self::LinuxVar => "4d21b016-b534-45c2-a9fb-5c16e091fd2d",
            Self::LinuxGeneric => "0fc63daf-8483-4772-8e79-3d69d8477de4",
            Self::Custom(g) => g.as_str(),
        }
    }

    /// Maps a GPT partition type GUID string to PartitionType.
    pub fn from_gpt_guid(guid: &str) -> Self {
        let clean = guid.trim().to_lowercase();
        match clean.as_str() {
            "c12a7328-f81f-11d2-ba4b-00a0c93ec93b" => Self::EfiSystem,
            "4f68bce3-e8cd-4db1-96e7-fbcaf984b709" => Self::LinuxRoot,
            "933ac7e1-2eb4-4f13-b844-0e14e2aef915" => Self::LinuxHome,
            "0657fd6d-a4ab-43c4-84e5-0933c84b4f4f" => Self::LinuxSwap,
            "4d21b016-b534-45c2-a9fb-5c16e091fd2d" => Self::LinuxVar,
            "0fc63daf-8483-4772-8e79-3d69d8477de4" => Self::LinuxGeneric,
            _ => Self::Custom(clean),
        }
    }
}

/// Mount point specification adhering to standard 6-field `fstab(5)` semantics.
///
/// T-01542 D1: unknown JSON fields are rejected (no silent tolerance) — a typo'd
/// field name must fail loudly instead of validating a document that means
/// something other than what was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MountPointSpec {
    /// Mount point directory (e.g. "/", "/boot/efi", "/tmp").
    pub path: String,
    /// Device identifier: UUID="...", LABEL="...", /dev/..., or pseudo-name ("tmpfs", "proc").
    pub device: String,
    /// Filesystem driver type.
    pub fs_type: FsType,
    /// Mount options (e.g. ["defaults", "rw", "nodev", "nosuid"]).
    pub options: Vec<String>,
    /// Dump frequency (0 = do not dump, 1 = backup).
    pub dump: u32,
    /// fsck pass order (0 = do not check, 1 = root fs, 2 = all other fs).
    pub pass: u32,
    /// Whether this mount is strictly required for operational readiness.
    pub required: bool,
}

impl MountPointSpec {
    /// Formats this specification into a standard 6-field `/etc/fstab` entry.
    pub fn to_fstab_line(&self) -> String {
        let opts = if self.options.is_empty() {
            "defaults".to_string()
        } else {
            self.options.join(",")
        };
        format!(
            "{:<35} {:<15} {:<10} {:<30} {:<2} {:<2}",
            self.device, self.path, self.fs_type, opts, self.dump, self.pass
        )
    }

    /// Parses a standard 6-field `fstab(5)` entry.
    pub fn parse_fstab_line(line: &str) -> Result<Option<Self>, String> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(None);
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 4 || parts.len() > 6 {
            return Err(format!(
                "invalid fstab line: expected between 4 and 6 whitespace-delimited fields, found {}",
                parts.len()
            ));
        }
        let device = parts[0].to_string();
        let path = parts[1].to_string();
        let fs_type: FsType = parts[2].parse().unwrap_or(FsType::Custom(parts[2].to_string()));
        let options: Vec<String> = parts[3]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let dump: u32 = if parts.len() >= 5 {
            parts[4].parse().map_err(|e| format!("invalid dump field: {}", e))?
        } else {
            0
        };
        let pass: u32 = if parts.len() >= 6 {
            parts[5].parse().map_err(|e| format!("invalid pass field: {}", e))?
        } else {
            0
        };
        let required = path == "/";

        let spec = Self {
            path,
            device,
            fs_type,
            options,
            dump,
            pass,
            required,
        };
        validate_mount_point(&spec)?;
        Ok(Some(spec))
    }
}

/// Partition specification on a target block device.
///
/// T-01542 D1: unknown fields rejected — see [`MountPointSpec`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartitionSpec {
    /// Partition table slot index (1-based, e.g. 1..128).
    pub index: u32,
    /// Human-readable partition label (e.g. "EFI", "AIOS_ROOT").
    pub label: String,
    /// Partition type classification.
    pub partition_type: PartitionType,
    /// Partition capacity in Megabytes (MiB).
    pub size_mib: u64,
    /// UUID of the partition if pre-assigned.
    pub uuid: Option<String>,
    /// Whether the bootable flag is set.
    pub bootable: bool,
    /// Optional default filesystem to format onto the partition.
    pub format_as: Option<FsType>,
}

/// Directory specification in the filesystem hierarchy.
///
/// T-01542 D1: unknown fields rejected — see [`MountPointSpec`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectorySpec {
    /// Normalized absolute directory path (e.g. "/var/lib/aios", "/tmp").
    pub path: String,
    /// POSIX permissions mode in octal (e.g. 0o755, 0o700, 0o1777).
    pub mode: u32,
    /// User ownership name (e.g. "root", "aios").
    pub owner: String,
    /// Group ownership name (e.g. "root", "aios").
    pub group: String,
    /// Functional purpose / description.
    pub description: String,
    /// If this path is a compatibility symlink (UsrMerge), target relative path (e.g. "usr/bin").
    pub symlink_target: Option<String>,
}

/// Complete Filesystem Layout specification for target system configuration and validation.
///
/// T-01542 D1: unknown fields rejected — see [`MountPointSpec`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemLayoutSpec {
    /// Unique identifier for the layout (e.g. "aios-uefi-standard-v1").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Detailed description of the target system architecture.
    pub description: String,
    /// Minimum block device capacity required to house this layout in bytes.
    pub target_disk_min_bytes: u64,
    /// Ordered list of partitions to create on the storage disk.
    pub partitions: Vec<PartitionSpec>,
    /// Mount specifications to write to /etc/fstab and verify during boot.
    pub mounts: Vec<MountPointSpec>,
    /// Essential directories to provision and verify for FHS 3.0 and AIOS.
    pub directories: Vec<DirectorySpec>,
    /// UTC timestamp of layout creation.
    pub created_at: String,
}

impl FilesystemLayoutSpec {
    /// Validates internal consistency invariants FL1..FL6.
    pub fn validate(&self) -> Result<(), String> {
        validate_filesystem_layout(self)
    }

    /// Serializes this layout to a formatted JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("failed to serialize layout: {}", e))
    }

    /// Deserializes and validates a layout from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let layout: Self = serde_json::from_str(json_str)
            .map_err(|e| format!("failed to deserialize layout JSON: {}", e))?;
        layout.validate()?;
        Ok(layout)
    }

    /// Generates full `/etc/fstab` file content from the mounts specification.
    pub fn generate_fstab(&self) -> String {
        let mut lines = Vec::new();
        lines.push("# /etc/fstab: static file system information generated by AIOS fs_layout.".to_string());
        lines.push(format!("# Layout ID: {}", self.id));
        lines.push(format!("# Generated at: {}", self.created_at));
        lines.push(format!(
            "{:<35} {:<15} {:<10} {:<30} {:<2} {:<2}",
            "# <file system>", "<mount point>", "<type>", "<options>", "<dump>", "<pass>"
        ));
        for m in &self.mounts {
            lines.push(m.to_fstab_line());
        }
        lines.push(String::new());
        lines.join("\n")
    }

    /// Canonical reference layout for UEFI x86_64 installation on a 64 GiB target.
    pub fn standard_uefi() -> Self {
        Self {
            id: "aios-uefi-standard-v1".into(),
            name: "AIOS Standard UEFI x86_64 Layout".into(),
            description: "Standard partition and mount layout for UEFI target host with ESP, Root, Swap, and CIS hardened temporary mounts".into(),
            target_disk_min_bytes: 64 * 1024 * 1024 * 1024,
            partitions: vec![
                PartitionSpec {
                    index: 1,
                    label: "EFI".into(),
                    partition_type: PartitionType::EfiSystem,
                    size_mib: 512,
                    uuid: None,
                    bootable: true,
                    format_as: Some(FsType::Vfat),
                },
                PartitionSpec {
                    index: 2,
                    label: "AIOS_ROOT".into(),
                    partition_type: PartitionType::LinuxRoot,
                    size_mib: 50 * 1024,
                    uuid: None,
                    bootable: false,
                    format_as: Some(FsType::Ext4),
                },
                PartitionSpec {
                    index: 3,
                    label: "AIOS_SWAP".into(),
                    partition_type: PartitionType::LinuxSwap,
                    size_mib: 4 * 1024,
                    uuid: None,
                    bootable: false,
                    format_as: Some(FsType::Swap),
                },
            ],
            mounts: vec![
                MountPointSpec {
                    path: "/".into(),
                    device: "LABEL=AIOS_ROOT".into(),
                    fs_type: FsType::Ext4,
                    options: vec!["rw".into(), "relatime".into(), "errors=remount-ro".into()],
                    dump: 0,
                    pass: 1,
                    required: true,
                },
                MountPointSpec {
                    path: "/boot/efi".into(),
                    device: "LABEL=EFI".into(),
                    fs_type: FsType::Vfat,
                    options: vec!["rw".into(), "relatime".into(), "fmask=0077".into(), "dmask=0077".into(), "nodev".into(), "nosuid".into()],
                    dump: 0,
                    pass: 2,
                    required: true,
                },
                MountPointSpec {
                    path: "/tmp".into(),
                    device: "tmpfs".into(),
                    fs_type: FsType::Tmpfs,
                    options: vec!["rw".into(), "nosuid".into(), "nodev".into(), "noexec".into(), "size=4G".into()],
                    dump: 0,
                    pass: 0,
                    required: true,
                },
                MountPointSpec {
                    path: "/dev/shm".into(),
                    device: "tmpfs".into(),
                    fs_type: FsType::Tmpfs,
                    options: vec!["rw".into(), "nosuid".into(), "nodev".into(), "noexec".into()],
                    dump: 0,
                    pass: 0,
                    required: true,
                },
                MountPointSpec {
                    path: "/proc".into(),
                    device: "proc".into(),
                    fs_type: FsType::Procfs,
                    options: vec!["rw".into(), "nosuid".into(), "nodev".into(), "noexec".into()],
                    dump: 0,
                    pass: 0,
                    required: true,
                },
                MountPointSpec {
                    path: "/sys".into(),
                    device: "sysfs".into(),
                    fs_type: FsType::Sysfs,
                    options: vec!["rw".into(), "nosuid".into(), "nodev".into(), "noexec".into()],
                    dump: 0,
                    pass: 0,
                    required: true,
                },
            ],
            directories: vec![
                DirectorySpec {
                    path: "/var/lib/aios".into(),
                    mode: 0o750,
                    owner: "root".into(),
                    group: "aios".into(),
                    description: "AIOS daemon persistent state and data".into(),
                    symlink_target: None,
                },
                DirectorySpec {
                    path: "/run/aios".into(),
                    mode: 0o750,
                    owner: "root".into(),
                    group: "aios".into(),
                    description: "AIOS volatile runtime IPC sockets".into(),
                    symlink_target: None,
                },
                DirectorySpec {
                    path: "/etc/aios".into(),
                    mode: 0o755,
                    owner: "root".into(),
                    group: "root".into(),
                    description: "AIOS system daemon configurations".into(),
                    symlink_target: None,
                },
                DirectorySpec {
                    path: "/var/log/aios".into(),
                    mode: 0o750,
                    owner: "root".into(),
                    group: "aios".into(),
                    description: "AIOS tamper-evident audit logs".into(),
                    symlink_target: None,
                },
                DirectorySpec {
                    path: "/tmp".into(),
                    mode: 0o1777,
                    owner: "root".into(),
                    group: "root".into(),
                    description: "Sticky bit world-writable temporary directory".into(),
                    symlink_target: None,
                },
                DirectorySpec {
                    path: "/bin".into(),
                    mode: 0o777,
                    owner: "root".into(),
                    group: "root".into(),
                    description: "UsrMerge /bin compatibility symlink".into(),
                    symlink_target: Some("usr/bin".into()),
                },
                DirectorySpec {
                    path: "/sbin".into(),
                    mode: 0o777,
                    owner: "root".into(),
                    group: "root".into(),
                    description: "UsrMerge /sbin compatibility symlink".into(),
                    symlink_target: Some("usr/sbin".into()),
                },
                DirectorySpec {
                    path: "/lib".into(),
                    mode: 0o777,
                    owner: "root".into(),
                    group: "root".into(),
                    description: "UsrMerge /lib compatibility symlink".into(),
                    symlink_target: Some("usr/lib".into()),
                },
            ],
            created_at: "2026-09-16T00:00:00Z".into(),
        }
    }

    /// Minimal layout for container/test environments with virtual filesystems.
    pub fn minimal_container() -> Self {
        Self {
            id: "aios-container-minimal-v1".into(),
            name: "AIOS Minimal Container Layout".into(),
            description: "Virtual and pseudo mount layout for container and chroot environments".into(),
            target_disk_min_bytes: 1024 * 1024 * 1024,
            partitions: vec![],
            mounts: vec![
                MountPointSpec {
                    path: "/".into(),
                    device: "overlay".into(),
                    fs_type: FsType::Overlayfs,
                    options: vec!["rw".into(), "relatime".into()],
                    dump: 0,
                    pass: 1,
                    required: true,
                },
                MountPointSpec {
                    path: "/proc".into(),
                    device: "proc".into(),
                    fs_type: FsType::Procfs,
                    options: vec!["rw".into(), "nosuid".into(), "nodev".into(), "noexec".into()],
                    dump: 0,
                    pass: 0,
                    required: true,
                },
                MountPointSpec {
                    path: "/sys".into(),
                    device: "sysfs".into(),
                    fs_type: FsType::Sysfs,
                    options: vec!["rw".into(), "nosuid".into(), "nodev".into(), "noexec".into()],
                    dump: 0,
                    pass: 0,
                    required: true,
                },
                MountPointSpec {
                    path: "/tmp".into(),
                    device: "tmpfs".into(),
                    fs_type: FsType::Tmpfs,
                    options: vec!["rw".into(), "nosuid".into(), "nodev".into(), "noexec".into()],
                    dump: 0,
                    pass: 0,
                    required: true,
                },
            ],
            directories: vec![
                DirectorySpec {
                    path: "/var/lib/aios".into(),
                    mode: 0o750,
                    owner: "root".into(),
                    group: "aios".into(),
                    description: "AIOS daemon state directory".into(),
                    symlink_target: None,
                },
                DirectorySpec {
                    path: "/run/aios".into(),
                    mode: 0o750,
                    owner: "root".into(),
                    group: "aios".into(),
                    description: "AIOS runtime socket directory".into(),
                    symlink_target: None,
                },
            ],
            created_at: "2026-09-16T00:00:00Z".into(),
        }
    }
}

/// Validates path hygiene according to invariant FL2.
pub fn validate_path_hygiene(path: &str, field_name: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err(format!("FL2 violation: {} cannot be empty", field_name));
    }
    if !path.starts_with('/') {
        return Err(format!(
            "FL2 violation: {} '{}' must be an absolute path starting with '/'",
            field_name, path
        ));
    }
    if path.len() > 1024 {
        return Err(format!(
            "FL2 violation: {} '{}' exceeds maximum allowed length of 1024 characters",
            field_name, path
        ));
    }
    if path.contains('\0') || path.chars().any(|c| (c as u32) < 32 || (c as u32) == 127) {
        return Err(format!(
            "FL2 violation: {} '{}' contains control characters",
            field_name, path
        ));
    }
    if path.contains("//") {
        return Err(format!(
            "FL2 violation: {} '{}' contains empty path components ('//')",
            field_name, path
        ));
    }
    if path.len() > 1 && path.ends_with('/') {
        return Err(format!(
            "FL2 violation: {} '{}' has forbidden trailing slash",
            field_name, path
        ));
    }
    // Check segments for . or ..
    for seg in path.split('/') {
        if seg == "." || seg == ".." {
            return Err(format!(
                "FL2 violation: {} '{}' contains relative segment '{}'",
                field_name, path, seg
            ));
        }
    }
    Ok(())
}

/// Validates an individual mount point specification.
pub fn validate_mount_point(spec: &MountPointSpec) -> Result<(), String> {
    validate_path_hygiene(&spec.path, "mount point path")?;
    if spec.device.trim().is_empty() {
        return Err("mount point device cannot be empty".into());
    }
    if spec.device.len() > 256 {
        return Err(format!("mount point device '{}' exceeds maximum length of 256 characters", spec.device));
    }
    if spec.device.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(format!("mount point device '{}' cannot contain control characters or whitespace", spec.device));
    }
    // T-01542 D7/E-6: `dump` is bounded to fstab(5) field-5 semantics.
    if spec.dump > 1 {
        return Err(format!(
            "mount '{}' dump must be 0 or 1, found {}",
            spec.path, spec.dump
        ));
    }
    for opt in &spec.options {
        if opt.trim().is_empty() {
            return Err("mount option cannot be empty".into());
        }
        if opt.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(format!("mount option '{}' cannot contain control characters or whitespace", opt));
        }
    }
    // Check pass numbers
    if spec.path == "/" {
        if spec.pass != 1 {
            return Err(format!(
                "FL1 violation: root mount pass number must be 1, found {}",
                spec.pass
            ));
        }
    } else if spec.pass == 1 {
        return Err(format!(
            "FL1 violation: non-root mount '{}' cannot have pass number 1",
            spec.path
        ));
    }
    // Check CIS security options on /tmp and /dev/shm
    if spec.path == "/tmp" || spec.path == "/dev/shm" {
        if !spec.options.iter().any(|o| o == "nodev") {
            return Err(format!(
                "FL4 violation: mount '{}' missing mandatory security option 'nodev'",
                spec.path
            ));
        }
        if !spec.options.iter().any(|o| o == "nosuid") {
            return Err(format!(
                "FL4 violation: mount '{}' missing mandatory security option 'nosuid'",
                spec.path
            ));
        }
        // T-01542 D3/E-3: noexec joins nodev+nosuid (CIS recommends all three;
        // both built-in layouts already carry it, so this is additive-only).
        if !spec.options.iter().any(|o| o == "noexec") {
            return Err(format!(
                "FL4 violation: mount '{}' missing mandatory security option 'noexec'",
                spec.path
            ));
        }
    }
    Ok(())
}

/// Validates an individual partition specification.
pub fn validate_partition_spec(spec: &PartitionSpec) -> Result<(), String> {
    if spec.index == 0 || spec.index > 128 {
        return Err(format!(
            "FL5 violation: partition index must be in range 1..=128, found {}",
            spec.index
        ));
    }
    if spec.label.trim().is_empty() {
        return Err("partition label cannot be empty".into());
    }
    if spec.size_mib == 0 {
        return Err("partition size must be greater than zero".into());
    }
    if spec.partition_type == PartitionType::EfiSystem {
        if spec.size_mib < 100 {
            return Err(format!(
                "FL5 violation: EFI System Partition size must be at least 100 MiB, found {} MiB",
                spec.size_mib
            ));
        }
        if let Some(ref fs) = spec.format_as {
            if *fs != FsType::Vfat {
                return Err(format!(
                    "FL5 violation: EFI System Partition must be formatted as vfat, found {}",
                    fs
                ));
            }
        }
    }
    Ok(())
}

/// Validates an individual directory specification.
pub fn validate_directory_spec(spec: &DirectorySpec) -> Result<(), String> {
    validate_path_hygiene(&spec.path, "directory path")?;
    if spec.owner.trim().is_empty() {
        return Err("directory owner cannot be empty".into());
    }
    if spec.owner.len() > 64 || spec.owner.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(format!("directory owner '{}' invalid: must be <= 64 chars and contain no control chars or whitespace", spec.owner));
    }
    if spec.group.trim().is_empty() {
        return Err("directory group cannot be empty".into());
    }
    if spec.group.len() > 64 || spec.group.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(format!("directory group '{}' invalid: must be <= 64 chars and contain no control chars or whitespace", spec.group));
    }
    // T-01542 D2/E-2: mode must be a legal POSIX permission mask (1..=0o7777).
    // 0 would create an unusable directory; beyond 0o7777 is a unit error
    // (bytes-as-mode). Setuid/setgid/sticky combinations inside the range are
    // legitimate operator choices and are not policed.
    if spec.mode == 0 || spec.mode > 0o7777 {
        return Err(format!(
            "directory '{}' mode must be in 1..=0o7777 (octal), found {}",
            spec.path, spec.mode
        ));
    }
    if let Some(ref target) = spec.symlink_target {
        if target.trim().is_empty() {
            return Err("symlink target cannot be empty if specified".into());
        }
        if target.len() > 1024 || target.chars().any(|c| c.is_control()) {
            return Err("symlink target cannot exceed 1024 characters or contain control characters".into());
        }
        // T-01542 D4/E-4: symlink entries are UsrMerge entries — relative,
        // dot-segment-free, rooted at 'usr'. Absolute or escaping targets would
        // defeat the merge; plain directories (symlink_target: None) are exempt.
        if target.starts_with('/')
            || target.split('/').any(|seg| seg == "." || seg == "..")
            || target.split('/').next() != Some("usr")
        {
            return Err(format!(
                "directory '{}' symlink_target '{}' must be a relative path under 'usr' (UsrMerge)",
                spec.path, target
            ));
        }
    }
    Ok(())
}

/// Validates internal consistency invariants FL1..FL6 across the whole filesystem layout.
pub fn validate_filesystem_layout(spec: &FilesystemLayoutSpec) -> Result<(), String> {
    if spec.id.trim().is_empty() {
        return Err("layout id cannot be empty".into());
    }
    if spec.id.len() > 64 || !spec.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(format!("layout id '{}' invalid: must be <= 64 ASCII alphanumeric, hyphen, underscore, or dot characters", spec.id));
    }
    if spec.name.trim().is_empty() {
        return Err("layout name cannot be empty".into());
    }
    if spec.name.len() > 128 {
        return Err(format!("layout name exceeds maximum length of 128 characters, found {}", spec.name.len()));
    }
    if spec.description.len() > 1024 {
        return Err(format!("layout description exceeds maximum length of 1024 characters, found {}", spec.description.len()));
    }
    if spec.partitions.len() > 128 {
        return Err(format!("layout partition count ({}) exceeds maximum limit of 128 partitions", spec.partitions.len()));
    }
    if spec.mounts.len() > 128 {
        return Err(format!("layout mount point count ({}) exceeds maximum limit of 128 mounts", spec.mounts.len()));
    }
    if spec.directories.len() > 1024 {
        return Err(format!("layout directory count ({}) exceeds maximum limit of 1024 directories", spec.directories.len()));
    }
    // T-01542 D7/E-5: creation timestamp must be an RFC 3339 **UTC** instant —
    // the Z designator is required, so a valid-but-offset form (+02:00) is
    // refused too: two spellings of the same instant would otherwise be
    // unequal keys in every byte-level comparison downstream.
    let created_ok = chrono::DateTime::parse_from_rfc3339(&spec.created_at)
        .ok()
        .filter(|_| spec.created_at.ends_with('Z'))
        .is_some();
    if !created_ok {
        return Err(format!(
            "layout 'created_at' must be an RFC 3339 UTC timestamp, found '{}'",
            spec.created_at
        ));
    }

    // FL1: Exactly one root mount point
    let root_count = spec.mounts.iter().filter(|m| m.path == "/").count();
    if root_count != 1 {
        return Err(format!(
            "FL1 violation: layout must have exactly one root ('/') mount, found {}",
            root_count
        ));
    }

    // FL6 (T-01542 D8/E-7): a layout with no required mount cannot be
    // "operationally ready" — the minimal honest semantics for the `required`
    // field, which was previously carried but never read.
    if !spec.mounts.iter().any(|m| m.required) {
        return Err("FL6 violation: at least one mount must be marked required".into());
    }

    // FL3: Unique mount points and parent-before-child ordering
    let mut seen_paths = std::collections::HashSet::new();
    for (idx, m) in spec.mounts.iter().enumerate() {
        validate_mount_point(m)?;
        if !seen_paths.insert(&m.path) {
            return Err(format!("FL3 violation: duplicate mount path '{}'", m.path));
        }
        // Ensure that if any preceding mount is an ancestor, it was mounted before.
        // And ensure that an ancestor cannot appear AFTER its descendant.
        for prior in &spec.mounts[..idx] {
            if is_descendant_path(&prior.path, &m.path) {
                return Err(format!(
                    "FL3 violation: child mount '{}' appears before parent '{}'",
                    prior.path, m.path
                ));
            }
        }
    }

    // FL5: Partition constraints
    let mut seen_indices = std::collections::HashSet::new();
    let mut total_part_mib: u64 = 0;
    let mut has_esp = false;

    for part in &spec.partitions {
        validate_partition_spec(part)?;
        if !seen_indices.insert(part.index) {
            return Err(format!(
                "FL5 violation: duplicate partition index {}",
                part.index
            ));
        }
        total_part_mib = total_part_mib.saturating_add(part.size_mib);
        if part.partition_type == PartitionType::EfiSystem {
            has_esp = true;
        }
    }

    let disk_budget_mib = spec.target_disk_min_bytes / (1024 * 1024);
    if total_part_mib > disk_budget_mib {
        return Err(format!(
            "FL5 violation: total partition size ({} MiB) exceeds target disk budget ({} MiB)",
            total_part_mib, disk_budget_mib
        ));
    }

    if has_esp {
        // If an ESP partition is present, verify that /boot/efi is among the mount points
        let esp_mount = spec.mounts.iter().find(|m| m.path == "/boot/efi");
        if let Some(m) = esp_mount {
            if m.fs_type != FsType::Vfat {
                return Err(format!(
                    "FL5 violation: mount point /boot/efi must use vfat filesystem, found {}",
                    m.fs_type
                ));
            }
        }
    }

    // Validate directory specifications
    for dir in &spec.directories {
        validate_directory_spec(dir)?;
    }

    Ok(())
}

/// Helper to determine if `child` is a sub-path under `parent`.
fn is_descendant_path(child: &str, parent: &str) -> bool {
    if child == parent {
        return false;
    }
    if parent == "/" {
        return true;
    }
    if child.starts_with(parent) {
        let remainder = &child[parent.len()..];
        return remainder.starts_with('/');
    }
    false
}
