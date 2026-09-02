# T-01002 — Distro Selection & Justification / Data Model: Specification

## 1. Data Model Specification

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistroFamily {
    Debian,
    Alpine,
    Arch,
    CustomMinimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InitSystem {
    Systemd,
    OpenRC,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchTarget {
    X86_64,
    Aarch64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CLibrary {
    Glibc,
    Musl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistroProfile {
    pub id: String,                 // e.g., "debian-12-minimal"
    pub name: String,               // e.g., "Debian GNU/Linux 12 (Bookworm)"
    pub family: DistroFamily,
    pub release_version: String,    // e.g., "12.5"
    pub init_system: InitSystem,
    pub arch: ArchTarget,
    pub c_lib: CLibrary,
    pub min_kernel_version: String, // e.g., "6.1.0"
    pub default_packages: Vec<String>,
    pub recommended: bool,
    pub justification: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistroEvaluation {
    pub profile_id: String,
    pub binary_compatibility_score: f32, // 0.0 .. 1.0
    pub footprint_score: f32,            // 0.0 .. 1.0
    pub security_score: f32,             // 0.0 .. 1.0
    pub overall_score: f32,              // 0.0 .. 1.0
    pub is_production_ready: bool,
    pub evaluated_at_utc: String,
}
```

## 2. Validation & Invariants
- `id` must not be empty and must match `^[a-z0-9\-_]+$`.
- `name` and `release_version` must be non-empty.
- `min_kernel_version` must be a valid semver triple (`X.Y.Z`).
- `overall_score` is computed as: `0.4 * binary_compatibility + 0.3 * security + 0.3 * footprint`.
- A profile is `production_ready` if `overall_score >= 0.75` and `binary_compatibility >= 0.8`.
