# T-01298: Package Management Recovery & Validation Hardening

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Recovery & Validation  
**Task ID:** T-01298  

---

## 1. Hardening Overview
Task `T-01298` hardens the **Recovery & Validation** subsystem of Package Management against failure modes, edge cases, resource exhaustion, and malformed inputs.

---

## 2. Hardening Measures Implemented

### 1. Bounded Buffer Reads & Stream Size Caps
- Store files are strictly capped at 10 MiB (`10 * 1024 * 1024` bytes).
- Reading uses `std::io::Read::take(10 * 1024 * 1024 + 1)` preventing memory exhaustion attacks via sparse files or endless streams.

### 2. Entity & Depth Limits
- Maximum registered packages in store: $\le 10,000$.
- Maximum dependencies per package: $\le 256$.
- Maximum package description length: $\le 4,096$ bytes.
- Maximum version and architecture length: $\le 64$ characters.

### 3. Non-Destructive Collision-Resistant Quarantine (RV4)
- `create_backup_file` guarantees that existing backup files are never overwritten:
  ```rust
  let mut counter = 1;
  while backup_path.exists() {
      backup_name = format!("{}.bak.{}_{}", base_name, ts, counter);
      backup_path = path.with_file_name(&backup_name);
      counter += 1;
  }
  ```
- File permissions are restricted to `0o644` on Unix platforms.

### 4. Explicit Error Envelopes (ADR-0035 / ADR-0036)
- Never fails silently.
- All failure modes produce structured error objects:
  - `LOAD_STORE_FAILED`: Store file missing or unreadable without `--fix`.
  - `RECOVERY_FAILED`: Filesystem I/O failure during backup or reseed.
  - `INVALID_ARGUMENT`: Store path exceeds 1024 characters or contains illegal control characters.
  - `UNHEALTHY_STORE`: Validation errors detected in specification.

### 5. Honest Audit Emission on Error Paths
- Every error and recovery path writes an audit row to the SQLite WAL hash-chained audit ring before returning, recording the exact error message and digital forensic artifact references.
