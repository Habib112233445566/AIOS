# Task Evidence: T-01911 - System Update Mechanism / core service: Research

## 1. Overview
- **Task ID**: `T-01911`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Research core update service architectures, A/B staging workflows, cryptographic verification pipelines, rollback orchestration, and state persistence patterns.

---

## 2. Research Findings

### 2.1 A/B Dual Slot Management
- **Partition Isolation**: Operating systems like Android, ChromeOS, and modern ostree/systemd-sysupdate use strict A/B isolation.
- **Running Slot Immutability**: The active root filesystem (`/`) is mounted read-only. Updates are staged onto the alternate inactive partition (`SlotA` or `SlotB`).
- **Boot Target Selection**: The bootloader environment variable (e.g. EFI boot order or systemd boot next) is set only after payload verification succeeds.
- **Rollback Guarantee**: If the new slot fails health checks (watchdog or self-tests), the bootloader or update service reverts to `rollback_slot`.

### 2.2 Core Service Lifecycle & Invariants (`USVC1..USVC6`)
1. **`USVC1` (Isolated Staging)**: All downloaded artifacts and manifests reside in a dedicated sandboxed staging directory (e.g. `/var/lib/aiosh/updates/staging`). Traversal paths are rejected.
2. **`USVC2` (Cryptographic Gate)**: No payload may be written or marked ready without 100% SHA-256 digest match against manifest declarations.
3. **`USVC3` (Active Slot Non-Interference)**: The active running slot is never targeted for writing. Target is always `current_slot.other()`.
4. **`USVC4` (Atomic State Persistence)**: Status and slot records are saved using atomic write patterns (`.tmp` + rename).
5. **`USVC5` (Storage & Resource Safety)**: Staged sizes are bounded by `MAX_UPDATE_PAYLOAD_SIZE`, and file read operations are bounded.
6. **`USVC6` (Deterministic Audit & Error Taxonomy)**: Every stage transition emits deterministic status records with error taxonomy (`UPD_STATE_ERROR`, `UPD_DIGEST_ERROR`, `UPD_SLOT_ERROR`).

---

## 3. Architecture for Sub-Epic 2
- Module: `code/aiosh-rust/aiosh-core/src/system_update_service.rs`
- Struct: `SystemUpdateService` managing:
  - `slot_status: SystemSlotStatus`
  - `update_status: SystemUpdateStatus`
  - `staging_dir: PathBuf`
  - Methods: `check_update`, `stage_artifact`, `verify_artifacts`, `apply_update`, `confirm_boot`, `rollback`, `save_state`, `load_state`.
