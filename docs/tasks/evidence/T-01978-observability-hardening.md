# Task Evidence: T-01978 - System Update / observability: Hardening (Sub-Epic 8)

## Overview
- **Task ID**: `T-01978`
- **Subsystem**: `SystemUpdateObservability` (`aiosh-core::system_update_observability`)
- **Objective**: Implement defensive hardening against symlink traversal, integer overflow in byte aggregation, payload tampering, and race conditions during persistence.

## Hardening Measures Implemented

1. **Symlink Defense & Regular File Validation**:
   - In `SystemUpdateObservabilityReport::generate()`, staged payload byte aggregation now uses `std::fs::symlink_metadata(p)` rather than `std::fs::metadata(p)`.
   - Explicitly verifies `meta.file_type().is_file()`, guaranteeing that symlinks or directory fixtures mistakenly present in `staged_artifacts` are never traversed or tallied.

2. **Arithmetic Overflow Protection**:
   - Replaced unchecked sum aggregation with `.fold(0u64, |acc, b| acc.saturating_add(b))` to prevent integer overflow on anomalous multi-terabyte staging filesystems.

3. **Atomic Snapshot Persistence (`save_to_file`)**:
   - Added `SystemUpdateObservabilityReport::save_to_file(&self, path: &Path)`:
     - 1 MB serialization size ceiling: rejects reports exceeding `1024 * 1024` bytes.
     - Destination symlink rejection: verifies destination path is not an existing symlink prior to writing.
     - Parent directory creation: automatically ensures parent directories exist.
     - Safe temp file pattern: writes to `{path}.tmp.{pid}` and uses atomic `rename`.
     - Error unlinking: immediately unlinks the temporary file on write or rename failure.

4. **Telemetry Text Sanitization (`sanitize_telemetry_text`)**:
   - Strips ASCII control characters (`!c.is_control()`).
   - Clamps string length to 256 characters (`.take(256)`).
   - Trims whitespace to eliminate newline and padding-based formatting attacks.
