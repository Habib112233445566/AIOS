# Task Evidence: T-01917 - System Update Mechanism / core service: Security Review

## 1. Overview
- **Task ID**: `T-01917`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Perform comprehensive security review and threat modeling for `SystemUpdateService`, analyzing file operations, staging sandboxes, permission boundaries, and state mutations.

---

## 2. Threat Modeling & Vulnerability Analysis

| Threat ID | Threat Category | Description | Severity | Target Mitigation |
|---|---|---|---|---|
| `THREAT-USVC-01` | Symlink Hijacking | Malicious pre-existing symlink in staging directory redirecting payload writes to critical host files (`/etc/shadow`). | **CRITICAL** | Check `fs::symlink_metadata()` before writing to ensure destination is not a symlink; remove existing file if it is a symlink. |
| `THREAT-USVC-02` | Disk Exhaustion DoS | Accumulating oversized payloads across multiple artifacts exceeding host disk quotas. | **HIGH** | Track total accumulated bytes across all staged artifacts against `max_payload_bytes` before staging. |
| `THREAT-USVC-03` | Stale Temporary Files | Orphaned `.tmp` files accumulating if process aborts during atomic state persistence. | **MEDIUM** | Remove pre-existing `.tmp` files and ensure atomic replace. |
| `THREAT-USVC-04` | Insecure Permissions | State or staged payload files created with loose permissions (e.g. world-readable). | **MEDIUM** | Set restrictive Unix file permissions (`0600` for files, `0700` for directories). |
| `THREAT-USVC-05` | Premature Confirmation | Calling `confirm_boot` while still in unverified state or if booted slot doesn't match target slot. | **HIGH** | Enforce `UpdateState::ReadyToReboot` check and validate active slot. |
| `THREAT-USVC-06` | Corrupted State File | Tampered or malformed JSON loaded from disk into memory without validation. | **MEDIUM** | Run `.validate()` immediately upon deserialization in `load_state_from_dir`. |

---

## 3. Required Hardening Actions for T-01918
1. **Symlink Defense**: In `stage_artifact`, verify `dest_path` is not a symbolic link. If an existing symlink is detected, reject or unlink it before writing.
2. **Quota Enforcement**: Check cumulative staged byte size plus new artifact size against `config.max_payload_bytes`.
3. **State Deserialization Validation**: In `load_state_from_dir`, invoke `slot_status.validate()?` before returning the service instance.
4. **Permissions Guard**: Ensure directories are created with restricted mode on Unix platforms.

---

## 4. Status
Threat modeling complete. Mitigations identified for implementation in `T-01918`.
