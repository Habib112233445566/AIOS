# T-01591 — Filesystem Layout recovery & validation: Research

## Metadata
- **Task ID:** `T-01591`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / recovery & validation
- **Status:** Complete — researched filesystem layout recovery mechanisms, corruption detection, atomic failover, and validation semantics.
- **Date:** 2026-09-19
- **Depends on:** `T-01590` (Sub-Epic 9 Milestone Closure)
- **Feeds:** `T-01592` (Recovery & Validation Specification)
- **Artifacts:** `docs/tasks/evidence/T-01591-recovery-validation-research.md`, `docs/tasks/evidence/T-01591-research.md`

---

## 1. Existing Capabilities & Prior Art

### 1.1 Atomic Persistence & Corruption Handling
- `FilesystemLayoutStore` uses staging files (`<store>.tmp.<pid>`) with `O_CREAT | O_EXCL` and `fsync` before atomic `rename`.
- Symlink overwrite protection prevents redirection attacks.
- Corrupted stores (syntax errors, unknown keys, invalid invariants) fail closed with `LOAD_STORE_FAILED` and are never overwritten in place.

### 1.2 Recovery Requirements
1. **Store Corruption Recovery**:
   - Detection of malformed JSON or invalid invariants before state mutation.
   - Backup creation (`.bak`) or snapshotting prior to destructive transitions.
   - Fallback to canonical presets (`aios-uefi-standard-v1`) if uninitialized.
2. **Mount & Partition Consistency Validation**:
   - Verification that required mount points (`/`, `/boot/efi`) are structurally sound.
   - Detection of partition overlap or mount point collisions.

---

## 2. Facts vs. Assumptions

| Item | Status | Details |
|---|---|---|
| Fail-closed on corruption | **Fact** | `aiosh` and `aiosh-mcp` refuse corrupted stores without modifying existing on-disk files. |
| Atomic persistence | **Fact** | Stage-and-rename guarantees crash consistency on supported filesystems. |
| In-memory preset fallback | **Fact** | When `--store` is absent or uninitialized, in-memory presets remain accessible. |
| Automated recovery test suite | **Assumption** | An automated suite `test_fs_layout_recovery_validation.py` (Criterion `FL14`) can systematically exercise recovery from corruption, backup recovery, and invariant validation. |

---

## 3. Decisions & Open Questions

- **Decision 1**: Recovery & validation test suite (Criterion `FL14`) will assert:
  - R1: Corruption detection (invalid JSON, truncated files, schema violations).
  - R2: Backup and fallback integrity (in-memory presets available during store outages).
  - R3: Invariant validation recovery (repairing invalid configs by restoring valid presets).
  - R4: Atomic write recovery (interrupted/failed writes leave the original store untouched).
  - R5: Recovery audit logging (audit events record recovery operations with honest outcomes).

---

## 4. Acceptance Confirmation

- [x] Evidence file exists and separates facts from assumptions.
- [x] No code changed; decisions needed are listed explicitly.
