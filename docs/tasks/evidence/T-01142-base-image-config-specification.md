# T-01142 — Base Image Build / Configuration: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Configuration

## 1. Specification Contract: `ImageBuildConfig`
```rust
pub struct ImageBuildConfig {
    pub build_dir: PathBuf,
    pub output_dir: PathBuf,
    pub default_target: String,
    pub max_build_duration_secs: u64,
    pub max_artifact_size_bytes: u64,
    pub compression_level: u32,
}
```

## 2. Invariant Specifications (CF1..CF6)
- **CF1 (Build Dir)**: Non-empty path.
- **CF2 (Output Dir)**: Non-empty path.
- **CF3 (Default Target)**: 1..128 printable graphic ASCII characters.
- **CF4 (Timeout Bounds)**: `10 <= max_build_duration_secs <= 86400`.
- **CF5 (Size Bounds)**: `1 MiB <= max_artifact_size_bytes <= 100 GiB`.
- **CF6 (Compression Bounds)**: `1 <= compression_level <= 22`.
