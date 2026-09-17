# T-01508: Filesystem Layout - Data Model: Hardening

## Metadata
- **Task ID:** `T-01508`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout (`aiosh-core::fs_layout`, `aiosh-cli`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (8/10) — Data Model Hardening
- **Dependencies:** `T-01507` (Security Review)
- **Next Task:** `T-01509` (Documentation)

---

## 1. Executive Summary & Hardening Measures

Task `T-01508` hardened the Filesystem Layout subsystem against memory exhaustion, combinatorial search explosions, input corruption, oversized file reads, and silent failures:

### 1.1 Structural & Cardinality Upper Bounds
- **Partition Count**: Capped at $\le 128$ partitions per layout (standard GPT table limit).
- **Mount Point Count**: Capped at $\le 128$ mount points per layout, preventing $O(N^2)$ mount hierarchy topology validation explosion.
- **Directory Count**: Capped at $\le 1024$ explicit directory specifications.
- **String Length Enforcements**:
  - `id`: Capped at 64 characters; strictly restricted to ASCII alphanumeric, hyphens (`-`), underscores (`_`), and dots (`.`).
  - `name`: Capped at 128 characters.
  - `description`: Capped at 1024 characters.
  - `device`: Capped at 256 characters; whitespace and control characters rejected.
  - `options`: Empty options rejected; each option checked for whitespace and control characters.
  - `path`: Capped at 1024 characters.
  - `owner` & `group`: Capped at 64 characters each; whitespace and control characters rejected.
  - `symlink_target`: Capped at 1024 characters; control characters rejected.

### 1.2 File I/O Protection (`aiosh-cli`)
- Implemented 10 MiB upper bound on `--spec <path>` file ingestion. File metadata size is checked *before* loading file content into memory, preventing denial-of-service via huge file reads.
- Bounded `--spec` CLI argument length to 1024 characters, rejecting any strings with control characters.

### 1.3 Error Envelope & Audit Hardening
- Standard JSON result envelope emitted on all CLI error paths (`{ "code": 1|2, "data": null, "error": { "code": "...", "message": "..." } }`).
- Failures on all paths emit structured audit rows with status `"failure"` and error details, preserving audit non-repudiation.

---

## 2. Verification Output

All tests in `aiosh-core` and `aiosh-cli` passed cleanly with zero regressions:
```
$ cargo test -p aiosh-core --test test_fs_layout_data_model
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

$ cargo test -p aiosh-cli --bin aiosh test_cmd_fs_layout_flow
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 1.80s
```

All acceptance criteria satisfied:
- [x] Failure modes produce explicit, auditable errors.
- [x] No temp/connection leaks on error paths.
- [x] Resource bounds and file size caps actively enforced.
