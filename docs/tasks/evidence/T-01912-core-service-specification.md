# Task Evidence: T-01912 - System Update Mechanism / core service: Specification

## 1. Overview
- **Task ID**: `T-01912`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Formally specify the System Update Core Service interfaces, state coordination contracts, atomic persistence layout, and verification pipelines.

---

## 2. Formal Specification

### 2.1 Configuration (`SystemUpdateServiceConfig`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemUpdateServiceConfig {
    pub state_dir: PathBuf,
    pub staging_dir: PathBuf,
    pub max_payload_bytes: u64,
    pub auto_rollback_on_failure: bool,
}
```

### 2.2 Core Service Struct (`SystemUpdateService`)
```rust
pub struct SystemUpdateService {
    pub config: SystemUpdateServiceConfig,
    pub slot_status: SystemSlotStatus,
    pub update_status: SystemUpdateStatus,
    pub active_manifest: Option<UpdateManifest>,
}
```

### 2.3 Operations & Lifecycle Contracts
1. `new(current_version: impl Into<String>, active_slot: UpdateSlot, config: SystemUpdateServiceConfig) -> Self`
   - Initializes `slot_status` with `current_slot = active_slot`, `target_slot = active_slot.other()`.
   - Initializes `update_status` in `UpdateState::Idle`.
2. `check_manifest(&mut self, manifest: UpdateManifest) -> Result<(), String>`
   - Validates manifest integrity via `manifest.validate()`.
   - Transitions `update_status` from `Idle` -> `Checking` -> `Downloading`.
   - Stores `active_manifest = Some(manifest)`.
3. `stage_artifact(&mut self, target: PartitionTarget, data: &[u8]) -> Result<(), String>`
   - Validates target exists in `active_manifest`.
   - Checks byte length against declared artifact size.
   - Computes SHA-256 digest of `data` and verifies matching hash (`USVC2`).
   - Writes payload atomically to `staging_dir/{file_name}`.
4. `verify_staged(&mut self) -> Result<(), String>`
   - Validates that all artifacts declared in `active_manifest` have been staged and verified.
   - Transitions state: `Downloading` -> `Verifying`.
5. `apply_update(&mut self) -> Result<(), String>`
   - Transitions state: `Verifying` -> `Applying`.
   - Swaps target slot into next boot position (`slot_status.switch_slot()`).
   - Transitions state: `Applying` -> `ReadyToReboot`.
6. `confirm_boot(&mut self, running_version: &str) -> Result<(), String>`
   - Validates system booted on the target slot.
   - Marks current slot success: `slot_status.mark_slot_success(current_slot, running_version)`.
   - Transitions state: `ReadyToReboot` -> `Verified` -> `Idle`.
7. `rollback(&mut self) -> Result<(), String>`
   - Restores slot from `rollback_slot`.
   - Transitions state: `ReadyToReboot` -> `RolledBack` -> `Idle`.
8. `save_state_to_dir(&self, dir: &Path) -> Result<(), String>`
   - Atomically persists `slot_status.json` and `update_status.json` using temp file + rename pattern.

---

## 3. Invariants Enforced
- `USVC1`: Staging directory sandboxing; path traversal forbidden.
- `USVC2`: Cryptographic digest verification of all artifacts before application.
- `USVC3`: Target slot is always `current_slot.other()`; live running slot is never overwritten.
- `USVC4`: Atomic metadata persistence via temporary file replacement.
- `USVC5`: Bounded payload byte limit checking against `MAX_UPDATE_PAYLOAD_SIZE`.
- `USVC6`: Deterministic error classification and state rollback on failure.
