//! Core service and registry for Linux Base Image Build subsystem.

use std::collections::BTreeMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::base_image::{BaseImageManifest, ImageFormat};

/// A discrete step in an image build execution plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildStage {
    pub name: String,
    pub description: String,
    pub command_template: String,
    pub estimated_duration_secs: u32,
}

/// Synthesized execution plan for building a bootable or container base image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildPlan {
    pub image_id: String,
    pub target_format: ImageFormat,
    pub stages: Vec<BuildStage>,
    pub estimated_artifact_size_bytes: u64,
    pub estimated_total_duration_secs: u32,
    pub generated_at: String,
}

impl BuildPlan {
    /// Validates internal consistency invariants P1..P3.
    pub fn validate(&self) -> Result<(), String> {
        if self.stages.len() < 4 {
            return Err("build plan must contain at least 4 discrete stages".into());
        }
        let total_dur: u32 = self.stages.iter().map(|s| s.estimated_duration_secs).sum();
        if self.estimated_total_duration_secs != total_dur {
            return Err("invariant P2 violated: total duration does not equal sum of stage durations".into());
        }
        if self.estimated_artifact_size_bytes == 0 {
            return Err("estimated artifact size must be greater than zero".into());
        }
        Ok(())
    }
}

/// Registry and build orchestrator for base image manifests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStore {
    images: BTreeMap<String, BaseImageManifest>,
}

impl Default for ImageStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageStore {
    /// Initializes a store seeded with reference Debian 12 minimal and Alpine 3.19 images.
    pub fn new() -> Self {
        let mut store = Self::empty();
        let debian_raw = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        let debian_qcow2 = BaseImageManifest::debian_12_minimal(ImageFormat::Qcow2);
        let debian_iso = BaseImageManifest::debian_12_minimal(ImageFormat::Iso);
        let alpine_tar = BaseImageManifest::alpine_319_container(ImageFormat::Tarball);

        let _ = store.register_image(debian_raw);
        let _ = store.register_image(debian_qcow2);
        let _ = store.register_image(debian_iso);
        let _ = store.register_image(alpine_tar);
        store
    }

    /// Initializes an empty image store.
    pub fn empty() -> Self {
        Self {
            images: BTreeMap::new(),
        }
    }

    /// Registers a new image manifest, enforcing validation and identifier uniqueness.
    pub fn register_image(&mut self, manifest: BaseImageManifest) -> Result<(), String> {
        manifest.validate()?;
        if self.images.contains_key(&manifest.id) {
            return Err(format!("image with id '{}' is already registered", manifest.id));
        }
        self.images.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    /// Returns all registered image manifests sorted by identifier.
    pub fn list_images(&self) -> Vec<BaseImageManifest> {
        self.images.values().cloned().collect()
    }

    /// Retrieves an image manifest by identifier.
    pub fn get_image(&self, id: &str) -> Option<&BaseImageManifest> {
        self.images.get(id)
    }

    /// Filters images matching a specific packaging format.
    pub fn filter_by_format(&self, format: ImageFormat) -> Vec<BaseImageManifest> {
        self.images
            .values()
            .filter(|m| m.format == format)
            .cloned()
            .collect()
    }

    /// Filters images matching a specific target distribution.
    pub fn filter_by_distro(&self, distro_id: &str) -> Vec<BaseImageManifest> {
        self.images
            .values()
            .filter(|m| m.rootfs.distro_id == distro_id)
            .cloned()
            .collect()
    }

    /// Synthesizes a discrete, deterministic 4-stage build execution plan for an image.
    pub fn generate_build_plan(&self, id: &str) -> Result<BuildPlan, String> {
        let manifest = self.get_image(id).ok_or_else(|| format!("image '{}' not found", id))?;

        let bootstrap_cmd = match manifest.rootfs.distro_id.as_str() {
            d if d.starts_with("debian") => {
                format!(
                    "debootstrap --arch={} --variant=minbase --include={} bookworm /target http://deb.debian.org/debian",
                    manifest.rootfs.architecture,
                    manifest.rootfs.packages.join(",")
                )
            }
            d if d.starts_with("alpine") => {
                format!(
                    "apk add --root /target --initdb --arch {} {}",
                    manifest.rootfs.architecture,
                    manifest.rootfs.packages.join(" ")
                )
            }
            _ => format!("bootstrap-rootfs --distro {} --arch {}", manifest.rootfs.distro_id, manifest.rootfs.architecture),
        };

        let kernel_cmd = format!(
            "chroot /target {} --kver {} --cmdline \"{}\"",
            manifest.kernel.initramfs_generator,
            manifest.kernel.version,
            manifest.kernel.cmdline
        );

        let config_cmd = format!(
            "echo {} > /target/etc/hostname && systemd-machine-id-setup --root=/target",
            manifest.rootfs.hostname
        );

        let package_cmd = match manifest.format {
            ImageFormat::Raw => format!("mkfs.{} -d /target /out/{}.img {}B", manifest.rootfs.filesystem_type, manifest.id, manifest.rootfs.size_budget_bytes),
            ImageFormat::Qcow2 => format!("qemu-img convert -f raw -O qcow2 /out/{}.img /out/{}.qcow2", manifest.id, manifest.id),
            ImageFormat::Iso => format!("xorriso -as mkisofs -iso-level 3 -o /out/{}.iso /target", manifest.id),
            ImageFormat::Tarball => format!("tar --zstd -cf /out/{}.tar.zst -C /target .", manifest.id),
        };

        let stages = vec![
            BuildStage {
                name: "bootstrap".into(),
                description: "Bootstrap minimal root filesystem with specified package set".into(),
                command_template: bootstrap_cmd,
                estimated_duration_secs: 45,
            },
            BuildStage {
                name: "kernel_and_boot".into(),
                description: "Install Linux LTS kernel and generate initramfs".into(),
                command_template: kernel_cmd,
                estimated_duration_secs: 30,
            },
            BuildStage {
                name: "system_config".into(),
                description: "Configure system hostname, network baseline, and machine-id".into(),
                command_template: config_cmd,
                estimated_duration_secs: 5,
            },
            BuildStage {
                name: "artifact_packaging".into(),
                description: format!("Pack assembled filesystem into target format {}", manifest.format),
                command_template: package_cmd,
                estimated_duration_secs: 25,
            },
        ];

        let total_dur: u32 = stages.iter().map(|s| s.estimated_duration_secs).sum();
        let estimated_size = match manifest.format {
            ImageFormat::Tarball => (manifest.rootfs.size_budget_bytes / 4).max(64 * 1024 * 1024),
            ImageFormat::Iso | ImageFormat::Qcow2 => (manifest.rootfs.size_budget_bytes / 2).max(128 * 1024 * 1024),
            ImageFormat::Raw => manifest.rootfs.size_budget_bytes,
        };

        let plan = BuildPlan {
            image_id: manifest.id.clone(),
            target_format: manifest.format,
            stages,
            estimated_artifact_size_bytes: estimated_size,
            estimated_total_duration_secs: total_dur,
            generated_at: chrono::Utc::now().to_rfc3339(),
        };

        plan.validate()?;
        Ok(plan)
    }

    /// Persists the image registry to disk as formatted JSON.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.images)
            .map_err(|e| format!("failed to serialize image store: {}", e))?;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(path, content.as_bytes())
            .map_err(|e| format!("failed to write image store to '{}': {}", path.display(), e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644));
        }
        Ok(())
    }

    /// Loads an image registry from disk, bounded to 10 MiB maximum file size.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)
            .map_err(|e| format!("failed to open image store at '{}': {}", path.display(), e))?;

        let meta = file.metadata()
            .map_err(|e| format!("failed to read metadata for '{}': {}", path.display(), e))?;
        if meta.len() > 10 * 1024 * 1024 {
            return Err(format!("image store at '{}' exceeds maximum allowed size of 10 MiB (was {} bytes)", path.display(), meta.len()));
        }

        let mut bytes = Vec::new();
        file.by_ref()
            .take(10 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("failed to read image store from '{}': {}", path.display(), e))?;

        if bytes.len() > 10 * 1024 * 1024 {
            return Err("image store content exceeded 10 MiB during stream read".into());
        }

        let images: BTreeMap<String, BaseImageManifest> = serde_json::from_slice(&bytes)
            .map_err(|e| format!("failed to parse image store at '{}': {}", path.display(), e))?;

        for manifest in images.values() {
            manifest.validate()?;
        }
        Ok(Self { images })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_image_store_canonical_initialization() {
        let store = ImageStore::new();
        let images = store.list_images();
        assert_eq!(images.len(), 4);
        assert!(store.get_image("debian-12-minimal-raw").is_some());
        assert!(store.get_image("debian-12-minimal-qcow2").is_some());
        assert!(store.get_image("debian-12-minimal-iso").is_some());
        assert!(store.get_image("alpine-319-container-tarball").is_some());
    }

    #[test]
    fn test_image_store_filters() {
        let store = ImageStore::new();
        let debian_images = store.filter_by_distro("debian-12-minimal-x86_64");
        assert_eq!(debian_images.len(), 3);

        let iso_images = store.filter_by_format(ImageFormat::Iso);
        assert_eq!(iso_images.len(), 1);
        assert_eq!(iso_images[0].id, "debian-12-minimal-iso");
    }

    #[test]
    fn test_generate_build_plan_valid() {
        let store = ImageStore::new();
        let plan = store.generate_build_plan("debian-12-minimal-raw").unwrap();
        assert_eq!(plan.stages.len(), 4);
        assert_eq!(plan.target_format, ImageFormat::Raw);
        assert_eq!(plan.estimated_total_duration_secs, 105);
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_image_store_persistence_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("image_store.json");

        let store = ImageStore::new();
        assert!(store.save_to_path(&path).is_ok());

        let loaded = ImageStore::load_from_path(&path).unwrap();
        assert_eq!(loaded.list_images().len(), 4);
    }

    #[test]
    fn test_duplicate_image_registration_rejected() {
        let mut store = ImageStore::new();
        let dupe = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
        assert!(store.register_image(dupe).is_err());
    }

    #[test]
    fn test_generate_build_plan_nonexistent_image() {
        let store = ImageStore::new();
        assert!(store.generate_build_plan("nonexistent-image").is_err());
    }

    #[test]
    fn test_build_plan_validation_failures() {
        let mut plan = BuildPlan {
            image_id: "test-img".into(),
            target_format: ImageFormat::Raw,
            stages: vec![],
            estimated_artifact_size_bytes: 100,
            estimated_total_duration_secs: 10,
            generated_at: "2026-09-03T12:00:00Z".into(),
        };
        assert!(plan.validate().is_err()); // less than 4 stages

        plan.stages = vec![
            BuildStage { name: "s1".into(), description: "d1".into(), command_template: "c1".into(), estimated_duration_secs: 5 },
            BuildStage { name: "s2".into(), description: "d2".into(), command_template: "c2".into(), estimated_duration_secs: 5 },
            BuildStage { name: "s3".into(), description: "d3".into(), command_template: "c3".into(), estimated_duration_secs: 5 },
            BuildStage { name: "s4".into(), description: "d4".into(), command_template: "c4".into(), estimated_duration_secs: 5 },
        ];
        plan.estimated_total_duration_secs = 999; // mismatch
        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_build_plan_alpine_tarball() {
        let store = ImageStore::new();
        let plan = store.generate_build_plan("alpine-319-container-tarball").unwrap();
        assert_eq!(plan.target_format, ImageFormat::Tarball);
        assert!(plan.stages[0].command_template.contains("apk add"));
        assert!(plan.stages[3].command_template.contains("tar --zstd"));
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_oversized_store_file_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("oversized.json");
        // Create an oversized file > 10 MiB
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(10 * 1024 * 1024 + 1024).unwrap();

        let res = ImageStore::load_from_path(&path);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("exceeds maximum allowed size"));
    }
}
