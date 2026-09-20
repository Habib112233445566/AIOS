# Task Evidence: T-01997 - System Update / recovery & validation: Security Review (Sub-Epic 10)

## 1. Overview
- **Task ID**: `T-01997`
- **Sub-Epic**: 10 (System Update Recovery & Validation Subsystem)
- **Goal**: Perform comprehensive security review and threat modeling for the system update recovery and validation subsystem.

---

## 2. Threat Model Analysis (THREAT-UVAL-01..06)

| Threat ID | Threat Description | Attack Vector / Trigger | Severity | Mitigation Strategy |
|---|---|---|---|---|
| `THREAT-UVAL-01` | **Directory Traversal in State Paths** | Adversary provides `../../etc/shadow` or relative path with `..` to `validate_update_store_path` or recovery functions. | High | Enforce UTF-8, length $\le 1024$, reject `..` path components, reject control/NUL characters, and require `.json` extension. |
| `THREAT-UVAL-02` | **Memory Exhaustion via Oversized State File** | Adversary crafts multi-gigabyte `slot_status.json` or `update_status.json` to trigger OOM during recovery read. | High | Check `symlink_metadata().len() <= MAX_UPDATE_STORE_SIZE` (1 MB) before invoking `fs::read_to_string`. |
| `THREAT-UVAL-03` | **Symlink Hijacking / Arbitrary File Quarantine** | Symlink placed at `slot_status.json` pointing to sensitive system file; recovery renames target to `.corrupt.<ts>`. | Critical | Reject symlinks via `symlink_metadata(&path).file_type().is_symlink()`; fail or refuse to quarantine symlinks. |
| `THREAT-UVAL-04` | **Arbitrary File Deletion via Staging Symlinks** | Symlink in staging directory pointing to critical host directory or file; pruning unlinks outside staging. | High | Check `meta.file_type().is_symlink()`; never follow symlinks during dangling artifact cleanup. |
| `THREAT-UVAL-05` | **Temporary File Leakage on I/O Failure** | Failure during atomic write of `.tmp.<pid>` leaves orphaned temporary files in state directory. | Low | Wrap temporary file writes in error cleanup blocks that proactively remove `.tmp.<pid>` on error. |
| `THREAT-UVAL-06` | **Split-Brain Slot Pointer Inconsistency** | Stored state has `current_slot == target_slot`, causing boot loop or simultaneous dual-slot activation. | High | Detect slot equality in `validate_update_state` and automatically resolve to alternate slot in `recover_update_state_in_memory`. |

---

## 3. Remediation & Hardening Plan for T-01998
1. Update `check_update_files` and `recover_update_files_with_backup`:
   - Inspect file size before read ($\le 1 \text{ MB}$).
   - Inspect symlink status before read/rename; reject symlinks explicitly.
   - Clean up temporary files on serialization/write error.
2. In staging artifact cleanup, skip and/or safely remove symlinks without following them.
3. Validate state directory path with `validate_update_store_path` or equivalent directory validator.
