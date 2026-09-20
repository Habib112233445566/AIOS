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

---

## 6. Operator CLI Subsystem

The operator CLI (`aiosh update` / `aiosh upd`) provides an interactive and machine-readable command-line interface for human administrators and autonomous agents to manage update staging, application, boot confirmation, and rollback.

### 6.1 Command Grammar
```bash
aiosh update <subcommand> [args...] [flags...]
aiosh upd <subcommand> [args...] [flags...]
```

### 6.2 Available Subcommands
- **`status`**: Queries the overall update service state, active partition slot, version numbers, and transfer progress percentage.
- **`slots`**: Displays detailed partition slot status (current slot, target slot, rollback slot, version strings, and successful boot indicators for Slot A and Slot B).
- **`check <manifest.json>`**: Reads and verifies the specified update manifest file, validating metadata, target artifacts, release channel, and signature. Transitions state to `Downloading`.
- **`apply`**: Finalizes update staging, verifies payloads against cryptographic digests, sets the next boot slot to the updated partition, and transitions state to `ReadyToReboot`.
- **`confirm [version]`**: Confirms successful boot of the new version on the updated partition slot. Sets `slot_successful` to `true` and resets state to `Idle`.
- **`rollback`**: Triggers immediate partition rollback to the designated fallback slot (`rollback_slot`) and resets state to `Idle`.

### 6.3 Command Flags & Options
- `--state-dir <path>`: Specifies custom directory for `slot_status.json` and `update_status.json` persistence (default: `/var/lib/aiosh/updates`).
- `--staging-dir <path>`: Specifies directory for payload artifacts (default: `/var/lib/aiosh/updates/staging`).
- `--version <ver>`: Overrides current running system version (default: `1.0.0`).
- `--slot <slot>`: Overrides default active partition slot (`slot_a` or `slot_b`).
- `--json`: Formats all stdout output using the standardized JSON result envelope.
- `--help`, `-h`: Displays usage instructions and supported subcommands.

### 6.4 Exit Code Contract
| Exit Code | Meaning | Examples |
|---|---|---|
| `0` | **Success** | Query succeeded, manifest accepted, update applied, boot confirmed. |
| `1` | **Domain / Operational Failure** | Manifest read error, state machine transition failure, corrupt payload digest, confirmation when not in `ReadyToReboot`. |
| `2` | **Syntax / Path Hygiene Error** | Unknown subcommand, missing required arguments, path length $> 1024$, control characters in paths or versions. |

### 6.5 JSON Output Envelope Contract
When invoked with `--json`, commands emit a structured envelope:
```json
{
  "code": 0,
  "data": { ... },
  "error": null
}
```
On error, `code` is non-zero, `data` is `null`, and `error` contains a machine-readable code and descriptive message:
```json
{
  "code": 2,
  "data": null,
  "error": {
    "code": "PATH_TOO_LONG",
    "message": "state-dir path cannot exceed 1024 characters"
  }
}
```

### 6.6 Operator CLI Invariants (UCLI1..UCLI6)
- **`UCLI1` (Deterministic Routing)**: Only authorized subcommands are dispatched; unrecognized subcommands immediately return exit code 2.
- **`UCLI2` (Path Hygiene Enforcement)**: Every filesystem path argument (`--state-dir`, `--staging-dir`, `check <path>`) is bounded to $\le 1024$ bytes and checked for control characters.
- **`UCLI3` (Structured Serialization)**: `--json` flag guarantees canonical JSON envelope output across both stdout successes and error conditions.
- **`UCLI4` (Audit Trail Integrity)**: All operator interactions are emitted as structured audit events via `classify_and_emit` into the SQLite WAL audit ring.
- **`UCLI5` (Hermetic Isolation)**: Custom state and staging directory flags enable completely isolated operation without side-effects on host partitions.
- **`UCLI6` (Terminal Safety)**: All strings printed in human-readable mode are filtered through `sanitize_terminal()`, preventing ANSI injection attacks.

---

## 7. Model Context Protocol (MCP) & Agent API Subsystem

The Model Context Protocol (MCP) tool surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`) exposes programmatic, structured JSON-RPC tools for autonomous AI agents, orchestrators, and external management APIs to discover, inspect, stage, apply, confirm, and roll back system updates.

### 7.1 Tool Manifest & Schemas

| Tool Name | Method / Description | Input Parameters | Output Payload |
|---|---|---|---|
| `aios.update.status` | Queries update engine state | `state_dir` (opt string) | `{ "state": string, "active_slot": string, "current_version": string, "target_version": opt string, "progress_percent": u8 }` |
| `aios.update.slots` | Queries A/B partition slot details | `state_dir` (opt string) | `{ "current_slot": string, "target_slot": string, "rollback_slot": opt string, "slot_a_version": string, "slot_b_version": string, "slot_a_successful": bool, "slot_b_successful": bool }` |
| `aios.update.check` | Checks & validates manifest | `manifest` (opt object), `manifest_path` (opt string), `state_dir` (opt string), `staging_dir` (opt string) | `{ "state": "downloading", "target_version": string, ... }` |
| `aios.update.apply` | Verifies digests & sets next boot slot | `state_dir` (opt string), `staging_dir` (opt string) | `{ "next_boot_slot": string, "status": object }` |
| `aios.update.confirm` | Confirms stable boot of updated slot | `version` (opt string), `state_dir` (opt string) | `{ "confirmed_version": string, "slot_status": object }` |
| `aios.update.rollback` | Rolls back to previous functional slot | `state_dir` (opt string) | `{ "restored_slot": string, "slot_status": object }` |

### 7.2 Copy-Pasteable JSON-RPC Examples

#### Query Status
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.update.status",
    "arguments": {
      "state_dir": "/var/lib/aiosh/updates"
    }
  }
}
```

#### Check Manifest
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.update.check",
    "arguments": {
      "manifest": {
        "update_id": "upd-2026-09-20-01",
        "version": "2.0.0",
        "channel": "stable",
        "artifacts": [
          {
            "target": "rootfs",
            "file_name": "rootfs.raw",
            "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "size_bytes": 1048576
          }
        ],
        "release_notes": "Security maintenance release",
        "published_at": "2026-09-20T00:00:00Z"
      }
    }
  }
}
```

#### Confirm Successful Boot
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.update.confirm",
    "arguments": {
      "version": "2.0.0"
    }
  }
}
```

### 7.3 Operational Invariants (UMCP1..UMCP6)
- **`UMCP1` (Structured Input Validation)**: All tools validate parameter types, reject missing required parameters, and return structured error envelopes.
- **`UMCP2` (Path & Traversal Hygiene)**: Enforces path length $\le 1024$ bytes, rejects control characters, and strictly prohibits `..` parent directory traversal components on `state_dir`, `staging_dir`, and `manifest_path`.
- **`UMCP3` (Memory & Symlink Bounds)**: Manifest files are capped at 1MB and symlink manifests are rejected (`symlink_metadata`). Version strings are bounded to $\le 64$ characters without whitespace.
- **`UMCP4` (PEP Policy Gating)**: Evaluates PEP permissions and records caller identity and grant ID for every tool call.
- **`UMCP5` (Audit Trail Emission)**: Every call writes an immutable audit record to the SQLite WAL ring via `dispatch::recorded_call`.
- **`UMCP6` (State Machine Gating)**: Tool calls enforce lifecycle constraints (e.g. `confirm` is only valid when state is `ready_to_reboot`).

### 7.4 Constraints & Known Limitations
1. **Isolated State**: In testing or containerized environments, callers must specify `--state-dir` / `state_dir` pointing to a writable directory.
2. **Reboot Execution**: `confirm` and `rollback` update the boot flags and partition metadata. Physical rebooting is deferred to system reboot controllers (`systemctl reboot` or hypervisor hooks).
3. **Staged Artifacts**: `apply` requires 100% of artifacts declared in the manifest to be present in `staging_dir` with matching cryptographic digests.

### 7.5 Task Evidence Links
- Research: [`docs/tasks/evidence/T-01931-mcp-surface-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01931-mcp-surface-research.md)
- Specification: [`docs/tasks/evidence/T-01932-mcp-surface-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01932-mcp-surface-specification.md)
- Scaffold: [`docs/tasks/evidence/T-01933-mcp-surface-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01933-mcp-surface-scaffold.md)
- Implementation: [`docs/tasks/evidence/T-01934-mcp-surface-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01934-mcp-surface-implementation.md)
- Unit Testing: [`docs/tasks/evidence/T-01935-mcp-surface-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01935-mcp-surface-unit-test.md)
- Integration: [`docs/tasks/evidence/T-01936-mcp-api-surface-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01936-mcp-api-surface-integration.md)
- Security Review: [`docs/tasks/evidence/T-01937-mcp-api-surface-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01937-mcp-api-surface-security-review.md)
- Hardening: [`docs/tasks/evidence/T-01938-mcp-api-surface-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01938-mcp-api-surface-hardening.md)

---

## 8. System Update Configuration & Policy Subsystem

The configuration subsystem (`code/aiosh-rust/aiosh-core/src/system_update_config.rs`) defines persistent, bounded, and auditable configuration models for the system update engine, supporting file-based persistence, environment variable overrides, and defensive input validation.

### 8.1 Configuration Data Model (`SystemUpdateConfig`)

| Field Name | Type | Default Value | Description |
|---|---|---|---|
| `state_dir` | `PathBuf` | `/var/lib/aiosh/updates` | Directory where partition and engine state JSON files are persisted. |
| `staging_dir` | `PathBuf` | `/var/lib/aiosh/updates/staging` | Dedicated directory where update payload artifacts are staged. |
| `default_channel` | `UpdateChannel` | `stable` | Default update release channel (`stable`, `beta`, `nightly`). |
| `check_interval_secs` | `u64` | `86400` (24h) | Automatic update check polling interval (between 60 and 2,592,000s). |
| `allow_auto_apply` | `bool` | `false` | Whether staged updates can be automatically applied without operator sign-off. |
| `auto_rollback_on_failure` | `bool` | `true` | Whether failed boots immediately trigger fallback partition activation. |
| `max_payload_bytes` | `u64` | `10737418240` (10 GB) | Cumulative payload byte limit for staged update artifacts. |
| `min_free_space_bytes` | `u64` | `1073741824` (1 GB) | Minimum required disk space reserve before staging payloads. |
| `max_download_rate_bps` | `Option<u64>` | `None` | Optional bandwidth throttle in bytes per second. |
| `trusted_keys` | `Vec<String>` | `[]` | List of trusted public key digests for release signature validation (max 32). |

### 8.2 Copy-Pasteable Example Configuration (`system_update.json`)
```json
{
  "state_dir": "/var/lib/aiosh/updates",
  "staging_dir": "/var/lib/aiosh/updates/staging",
  "default_channel": "stable",
  "check_interval_secs": 86400,
  "allow_auto_apply": false,
  "auto_rollback_on_failure": true,
  "max_payload_bytes": 10737418240,
  "min_free_space_bytes": 1073741824,
  "trusted_keys": [
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  ]
}
```

### 8.3 Environment Variable Overrides
The engine evaluates environment overrides upon startup via `SystemUpdateConfig::from_env()`:
- `AIOSH_UPDATE_STATE_DIR`: Overrides `state_dir`.
- `AIOSH_UPDATE_STAGING_DIR`: Overrides `staging_dir`.
- `AIOSH_UPDATE_CHANNEL`: Overrides `default_channel` (`stable`, `beta`, `nightly`).
- `AIOSH_UPDATE_CHECK_INTERVAL_SECS`: Overrides `check_interval_secs`.
- `AIOSH_UPDATE_AUTO_APPLY`: Overrides `allow_auto_apply` (`true`/`false`).
- `AIOSH_UPDATE_AUTO_ROLLBACK`: Overrides `auto_rollback_on_failure` (`true`/`false`).
- `AIOSH_UPDATE_MAX_PAYLOAD_BYTES`: Overrides `max_payload_bytes`.

### 8.4 Operational Invariants (UCONF1..UCONF6)
- **`UCONF1` (Path Hygiene)**: `state_dir` and `staging_dir` must be non-empty UTF-8, length $\le 1024$ bytes, zero control characters, and zero `..` parent directory traversal components.
- **`UCONF2` (Resource & Interval Bounds)**: `check_interval_secs` is bounded between 60s and 30 days. `max_payload_bytes` is bounded between 1MB and 10GB. `min_free_space_bytes` is bounded between 1MB and 100GB.
- **`UCONF3` (Key Bounds)**: `trusted_keys` is capped at 32 entries, each $\le 256$ characters and free of control characters.
- **`UCONF4` (Environment Ingestion)**: Environment variables are sanitized and bounded prior to ingestion.
- **`UCONF5` (Atomic & Bounded Persistence)**: Configuration files are capped at 1MB (`MAX_UPDATE_CONFIG_FILE_BYTES = 1_048_576`). Saving writes to `.tmp.<pid>` and atomically renames.
- **`UCONF6` (Fail-Safe Defaults)**: Missing or corrupt files fall back to safe default configuration without system panic.

### 8.5 Constraints & Known Limitations
1. **File Size Cap**: Configuration files exceeding 1MB are rejected to prevent memory exhaustion.
2. **Minimum Interval Floor**: Polling interval cannot be set below 60 seconds to prevent denial of service against update mirrors.
3. **Symlink Rejection**: Configuration files that are symlinks are rejected to prevent file redirection attacks.

### 8.6 Task Evidence Links
- Research: [`docs/tasks/evidence/T-01941-configuration-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01941-configuration-research.md)
- Specification: [`docs/tasks/evidence/T-01942-configuration-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01942-configuration-specification.md)
- Scaffold: [`docs/tasks/evidence/T-01943-configuration-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01943-configuration-scaffold.md)
- Implementation: [`docs/tasks/evidence/T-01944-configuration-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01944-configuration-implementation.md)
- Unit Testing: [`docs/tasks/evidence/T-01945-configuration-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01945-configuration-unit-test.md)
- Integration: [`docs/tasks/evidence/T-01946-configuration-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01946-configuration-integration.md)
- Security Review: [`docs/tasks/evidence/T-01947-configuration-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01947-configuration-security-review.md)
- Hardening: [`docs/tasks/evidence/T-01948-configuration-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01948-configuration-hardening.md)

---

## 9. Automated Testing & End-to-End Verification Subsystem

### 9.1 Overview & Architecture
The AIOS System Update Mechanism includes a comprehensive, automated end-to-end test harness designed to validate all operational pathways without requiring root privileges or physical disk partitioning:
- **Rust Native Integration Test Harness**: Located at [`code/aiosh-rust/aiosh-core/tests/test_system_update_e2e.rs`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/tests/test_system_update_e2e.rs). Executes via `cargo test` and provides sub-millisecond validation of lifecycle state machines, cryptographic digest calculations, quota boundaries, and RAII temporary directory cleanup.
- **Python / MCP Integration Smoke Suite**: Located at [`code/aiosh-mcp/tests/test_system_update_e2e_smoke.py`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-mcp/tests/test_system_update_e2e_smoke.py). Validates cross-substrate JSON serialization, tool discovery, and client-level orchestration.

### 9.2 Invariants Enforced (UTEST1 - UTEST6)
1. **`UTEST1` (Clean A/B Update Lifecycle)**: Validates complete progression from `SlotA` -> `Checking` -> `Downloading` -> `Verifying` -> `Applying` -> `ReadyToReboot` -> `confirm_boot("2.0.0")` -> `Verified` -> `Idle`, with target slot switching to `SlotB`.
2. **`UTEST2` (Cryptographic Fault Injection)**: Validates bit-flip detection (100% digest verification failure returning `UPD_DIGEST_ERROR`) and payload size truncation (immediate rejection before disk write returning `UPD_VALIDATION_ERROR`). Asserts service transitions to `Failed` state without slot mutation.
3. **`UTEST3` (Boot Failure & Rollback Simulation)**: Validates that if a staged slot fails boot health checks, calling `rollback()` safely restores the active partition pointer to `SlotA` and resets status to `Idle`.
4. **`UTEST4` (Quota & Symlink Traversal Defense)**: Asserts that payloads exceeding `max_payload_bytes` or attempting symlink redirection are rejected before writing to storage.
5. **`UTEST5` (Out-of-Order State Transitions)**: Asserts that illegal state leaps (e.g. calling `apply_update()` from `Idle` or `Downloading`) are rejected with `UPD_STATE_ERROR` with zero state mutation.
6. **`UTEST6` (Cross-Substrate Parity)**: Validates that JSON representations of slot states, update states, and manifests match across Rust core and Python MCP environments.

### 9.3 Invocation Examples

#### Run Rust E2E Test Suite
```bash
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_e2e -- --nocapture
```

#### Run Python End-to-End Smoke Test
```bash
python code/aiosh-mcp/tests/test_system_update_e2e_smoke.py
```

### 9.4 Constraints & Known Limitations
- **User-Space Emulation**: Tests operate entirely in user space utilizing mock filesystem directories. Real physical partition flips on EFI block devices (`/dev/sda1`, `/dev/sda2`) are orchestrated by the kernel/bootloader integration layer (`bootloader_env.rs`) rather than the user-space test harness.
- **Mock Signatures**: In environments without a configured hardware secure element or Ed25519 keyring, signature verification uses mock keys.

### 9.5 Evidence Artifacts
- Research: [`docs/tasks/evidence/T-01951-automated-tests-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01951-automated-tests-research.md)
- Specification: [`docs/tasks/evidence/T-01952-automated-tests-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01952-automated-tests-specification.md)
- Scaffold: [`docs/tasks/evidence/T-01953-automated-tests-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01953-automated-tests-scaffold.md)
- Implementation: [`docs/tasks/evidence/T-01954-automated-tests-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01954-automated-tests-implementation.md)
- Unit Testing: [`docs/tasks/evidence/T-01955-automated-tests-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01955-automated-tests-unit-test.md)
- Integration: [`docs/tasks/evidence/T-01956-automated-tests-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01956-automated-tests-integration.md)
- Security Review: [`docs/tasks/evidence/T-01957-automated-tests-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01957-automated-tests-security-review.md)
- Hardening: [`docs/tasks/evidence/T-01958-automated-tests-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01958-automated-tests-hardening.md)
