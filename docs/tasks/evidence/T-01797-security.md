# Task Evidence: T-01797 - Hardware Detection / Recovery & Validation: Security Review

## Metadata
- **Task ID:** `T-01797`
- **Sub-Epic:** Sub-Epic 10: Hardware Detection / Recovery & Validation
- **Component:** `aiosh-core::hardware_recovery`, `aiosh-core::hardware_service`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Security Objectives
1. Threat model the Hardware Recovery & Validation subsystem.
2. Review file-handling, quarantine backup creation, and path validation for arbitrary file overwrite or traversal vulnerabilities.
3. Identify abuse scenarios and specify hardening remediations for `T-01798`.

## Threat Analysis & Findings

| Threat ID | Category | Description | Severity | Status / Remediation |
|:---|:---|:---|:---|:---|
| **THREAT-HREC-01** | Path Traversal / Arbitrary File Overwrite | Calling `recover_inventory_file` with relative paths containing `..` or unverified destinations could overwrite critical system files with JSON. | High | Identified for hardening in `T-01798`: Implement `validate_store_path` to reject `..`, control characters, and ensure paths end with `.json`. |
| **THREAT-HREC-02** | Symlink Following on Quarantine & Write | If the target store path or its `.bak` sibling is a pre-existing symlink, `fs::write` or `fs::copy` could follow symlinks to unintended files (similar to N-5). | Medium | Identified for hardening in `T-01798`: Verify path is a regular file and does not follow symlinks outside allowed directories; write atomically. |
| **THREAT-HREC-03** | Disk Exhaustion via Repeated Quarantine | An attacker repeatedly triggering recovery on corrupted files could flood the filesystem with timestamped `.bak` files. | Low | Mitigated: Recovery is only invoked upon corruption detection; store file size is strictly capped at `MAX_STORE_FILE_SIZE = 10 MB`. |
| **THREAT-HREC-04** | Stale / Malformed Sysfs Path Injections | Devices with crafted `sysfs_path` attempting to probe non-sysfs host locations (e.g. `/proc/kcore` or `/dev/urandom`). | Low | Mitigated: `check_paths` only calls `Path::exists()`, never reads or writes to device paths. Paths are length-bounded to `MAX_PATH_LEN = 512`. |

## Conclusion
The Recovery & Validation subsystem provides robust data model healing, but file-level path handling requires explicit path traversal sanitization (`THREAT-HREC-01`) and symlink safety checks (`THREAT-HREC-02`). These remediations are scheduled for implementation in `T-01798`.
