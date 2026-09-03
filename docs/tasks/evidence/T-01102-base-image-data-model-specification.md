# T-01102 — Base Image Build / Data Model: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Data Model

## 1. Rust Types & Data Contract Specification
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Raw,
    Qcow2,
    Iso,
    Tarball,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootfsSpec {
    pub distro_id: String,
    pub architecture: String,
    pub filesystem_type: String,
    pub packages: Vec<String>,
    pub size_budget_bytes: u64,
    pub hostname: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelSpec {
    pub version: String,
    pub cmdline: String,
    pub initramfs_generator: String,
}

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
```

## 2. Invariants
- `I1 (Format & ID)`: `id` must be non-empty, matching `^[a-z0-9-]+$`.
- `I2 (SemVer 2.0)`: `version` must parse into `major.minor.patch`.
- `I3 (Package List)`: `rootfs.packages` must not be empty.
- `I4 (Filesystem)`: `filesystem_type` must be one of `"ext4"`, `"squashfs"`, `"btrfs"`, `"erofs"`.
- `I5 (Size Budget)`: `size_budget_bytes` must be $> 0$ and $\le 10 \text{ GiB}$.
- `I6 (Checksum)`: If present, `artifact_sha256` must be exactly 64 lowercase hex characters.
