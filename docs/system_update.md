# AIOS System Update Mechanism Subsystem

## 1. Architectural Overview

The **System Update Mechanism** is the operating system lifecycle and atomic deployment subsystem of AIOS (`Phase 1 — Linux Base System & Bootable Target / System Update Mechanism`, tasks `T-01901` through `T-02000`).

The subsystem provides:
- **Dual-Slot A/B Partition Deployment**: Active / Inactive partition isolation. All updates are staged and verified on the inactive slot without affecting the live running system.
- **Atomic Slot Switching & Rollback Safeguards**: Bootloader variables (e.g. `systemd-boot`, GRUB, or U-Boot) point to the newly prepared slot only after full cryptographic verification. If boot verification fails, the system automatically rolls back to the previous known-good slot.
- **Cryptographic Payload Integrity**: Multi-artifact manifests where every constituent image (kernel, rootfs, initramfs) requires strict SHA-256 digest verification and size bounds checking.
- **Deterministic State Machine**: A strictly linear lifecycle machine (`Idle -> Checking -> Downloading -> Verifying -> Applying -> ReadyToReboot -> Verified / RolledBack -> Idle`) that prevents premature execution or verification bypass.
- **Cross-Substrate Parity**: Byte-level schema compatibility between Rust (`aiosh-core`), CLI (`aiosh-cli`), and MCP tooling.

---

## 2. Domain Data Model (Sub-Epic 1: T-01901..T-01910)

The core data structures are defined in `code/aiosh-rust/aiosh-core/src/system_update.rs`.

### 2.1 Update Slot (`UpdateSlot`)
Represents the dual boot partitions:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateSlot {
    SlotA,
    SlotB,
}

impl UpdateSlot {
    pub fn other(&self) -> Self;
    pub fn as_str(&self) -> &'static str;
    pub fn from_str_loose(s: &str) -> Option<Self>;
}
```

### 2.2 Distribution Channels (`UpdateChannel`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
    Development,
}
```

### 2.3 Partition Targets (`PartitionTarget`)
Designates which target block device or partition an artifact updates:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionTarget {
    Rootfs,
    Kernel,
    Initramfs,
    FullBundle,
}
```

### 2.4 Payload Artifact (`UpdateArtifact`)
A cryptographic payload component:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateArtifact {
    pub target: PartitionTarget,
    pub file_name: String,
    pub sha256: String,
    pub size_bytes: u64,
}
```
**Validation Rules:**
- `file_name`: 1..128 characters, no path traversal (`..`), no path separators (`/`, `\`), no leading dots, no whitespace or control characters.
- `sha256`: Exactly 64 ASCII hexadecimal characters.
- `size_bytes`: $1 \le \text{size} \le 10\text{ GB}$ (`MAX_UPDATE_PAYLOAD_SIZE`).

### 2.5 Update Manifest (`UpdateManifest`)
Complete signed release metadata document:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub update_id: String,
    pub version: String,
    pub channel: UpdateChannel,
    pub min_version: Option<String>,
    pub artifacts: Vec<UpdateArtifact>,
    pub signature: Option<String>,
    pub release_notes: String,
    pub published_at: String,
}
```
**Validation Rules:**
- `update_id`: 1..128 characters.
- `version`: 1..64 characters.
- `artifacts`: 1..32 artifacts (`MAX_ARTIFACTS_PER_MANIFEST`).
- All artifact file names and partition targets must be unique within the manifest.
- Total byte footprint calculated with overflow-safe saturating arithmetic (`saturating_add`).

### 2.6 Slot Status (`SystemSlotStatus`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSlotStatus {
    pub current_slot: UpdateSlot,
    pub target_slot: UpdateSlot,
    pub rollback_slot: Option<UpdateSlot>,
    pub slot_a_version: String,
    pub slot_b_version: String,
    pub slot_a_successful: bool,
    pub slot_b_successful: bool,
}
```
- Invariant: `current_slot != target_slot`.
- Supports `switch_slot()` and `mark_slot_success(slot, version)`.

### 2.7 Update Execution Lifecycle (`UpdateState` & `SystemUpdateStatus`)
```rust
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
```

---

## 3. Subsystem Invariants (UPD1..UPD6)

- **`UPD1` (Active Slot Exclusivity & Toggling)**: Exactly one slot is designated `current_slot`. All updates stage to `current_slot.other()`. A status where `current_slot == target_slot` triggers `UPD_SLOT_ERROR`.
- **`UPD2` (Version Boundaries & Channels)**: Version strings are bounded $\le 64$ characters. Channels must match `stable`, `beta`, `nightly`, or `development`.
- **`UPD3` (Cryptographic Digest Integrity)**: Every artifact must have an exact 64-char ASCII hex SHA-256 digest (`UPD_DIGEST_ERROR`). Filenames are strictly sanitized against directory traversal.
- **`UPD4` (Strict Linear State Machine)**: Only valid transitions are permitted:
  - `Idle -> Checking | Downloading`
  - `Checking -> Downloading | Idle`
  - `Downloading -> Verifying`
  - `Verifying -> Applying`
  - `Applying -> ReadyToReboot`
  - `ReadyToReboot -> Verified | RolledBack`
  - Terminal states (`Verified`, `RolledBack`, `Failed`) reset to `Idle`.
  - Transitions to `Failed` are allowed from any active state.
- **`UPD5` (Rollback Safeguard)**: The previous known-good partition is captured in `rollback_slot`. Slot toggling never discards recovery metadata.
- **`UPD6` (Path Hygiene & JSON Parity)**: All artifact paths, staging files, and manifest structures serialize to canonical snake_case JSON compatible across Rust and Python surfaces.

---

## 4. Threat Model & Hardening Mitigations

| Threat ID | Threat Category | Implemented Mitigation |
|---|---|---|
| `THREAT-UPD-01` | Directory Traversal | Filenames restricted to 128 chars; forbidden `..`, `/`, `\`, leading dots, control characters, and whitespace. |
| `THREAT-UPD-02` | Digest Evasion | Exact 64-character ASCII hexadecimal matching; rejected truncated or non-hex digests. |
| `THREAT-UPD-03` | Downgrade / Replay | Release manifest includes `min_version` validation against running version. |
| `THREAT-UPD-04` | Active Slot Mutation | Rejection of updates where target slot matches current active slot. |
| `THREAT-UPD-05` | Denial-of-Service / Overflow | Manifest capped at 32 artifacts; `total_bytes()` uses `saturating_add`. |
| `THREAT-UPD-06` | State Machine Desynchronization | State machine rejects illegal leaps (e.g. `Idle -> ReadyToReboot`) with `UPD_STATE_ERROR`. |

---

## 5. Core Service Subsystem (Sub-Epic 2: T-01911..T-01920)

The core update orchestrator is implemented in `code/aiosh-rust/aiosh-core/src/system_update_service.rs`.

### 5.1 Service Configuration (`SystemUpdateServiceConfig`)
```rust
pub struct SystemUpdateServiceConfig {
    pub state_dir: PathBuf,              // Default: /var/lib/aiosh/updates
    pub staging_dir: PathBuf,            // Default: /var/lib/aiosh/updates/staging
    pub max_payload_bytes: u64,          // Default: 10 GB
    pub auto_rollback_on_failure: bool,  // Default: true
}
```

### 5.2 Service Lifecycle Flow
1. **Intake (`check_manifest`)**: Evaluates `UpdateManifest` schema, validates that the system is currently `Idle`, creates `staging_dir`, and transitions to `Downloading`.
2. **Staging (`stage_artifact`)**: 
   - Checks cumulative payload bytes against `max_payload_bytes`.
   - Protects against symlink hijacking on `staging_dir/{file_name}`.
   - Computes SHA-256 digest on incoming byte payload; mismatches immediately halt in `Failed` state with `UPD_DIGEST_ERROR`.
   - Writes payload to sandboxed staging file and increments progress percentage.
3. **Verification Gate (`verify_staged`)**:
   - Asserts all artifacts declared in the manifest exist in staging.
   - Transitions state to `Verifying`.
4. **Boot Slot Application (`apply_update`)**:
   - Asserts slot invariants (`current_slot != target_slot`).
   - Toggles active slot indicator (`slot_status.switch_slot()`).
   - Transitions state to `ReadyToReboot`.
5. **Boot Confirmation (`confirm_boot`)**:
   - Invoked after boot; marks target slot as booted successfully with the new version.
   - Resets state to `Idle`.
6. **Rollback (`rollback`)**:
   - Invoked if boot health checks fail.
   - Restores partition pointer to `rollback_slot`.
   - Resets state to `Idle`.

### 5.3 Core Service Invariants (USVC1..USVC6)
- **`USVC1` (Isolated Staging Directory)**: Artifacts are isolated in `config.staging_dir`. Symlinks are rejected.
- **`USVC2` (Cryptographic Digest Gate)**: SHA-256 verification of 100% of payloads before application. Missing artifacts prevent entering `Verifying`.
- **`USVC3` (Active Slot Non-Interference)**: The active running partition is never targeted or mutated. Only `current_slot.other()` is updated.
- **`USVC4` (Atomic State Persistence)**: State files (`slot_status.json`, `update_status.json`) are written via `.tmp` files with atomic `rename()`.
- **`USVC5` (Rollback Safeguard)**: Functional boot partition is preserved in `rollback_slot` and restored on failure.
- **`USVC6` (Deterministic Error Reporting)**: Errors are mapped to `UPD_STATE_ERROR`, `UPD_DIGEST_ERROR`, `UPD_SLOT_ERROR`, `UPD_VALIDATION_ERROR`.

### 5.4 Core Service Threat Mitigations
| Threat ID | Threat Category | Implemented Mitigation |
|---|---|---|
| `THREAT-USVC-01` | Symlink Hijacking | Destination path checked with `symlink_metadata()`; existing symlinks are rejected. |
| `THREAT-USVC-02` | Disk Quota DoS | Cumulative staged bytes checked against `max_payload_bytes` before writing. |
| `THREAT-USVC-03` | Stale Temporary Files | Pre-existing `.tmp` files unlinked before writing state. |
| `THREAT-USVC-04` | Corrupted State File | `slot_status.validate()?` executed immediately after loading state JSON from disk. |

