# T-01548 — Filesystem Layout configuration: Hardening

## Metadata
- **Task ID:** `T-01548`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / configuration
- **Status:** Complete — configuration surface hardened with size caps, atomic persistence, bounded reads, resource cleanup, and standard result envelopes.
- **Date:** 2026-09-19
- **Depends on:** `T-01547` (Configuration Security Review)
- **Feeds:** `T-01549` (Configuration Documentation)
- **Artifacts:** `docs/tasks/evidence/T-01548-configuration-hardening.md`

---

## 1. Hardening Dimensions & Enforcement

| Hardening Requirement | Implementation & Defense Mechanism | Evidence / Invariant |
|---|---|---|
| **H1: Size Bounds & Memory Protection** | Documents read from filesystem are capped at 10 MiB (`MAX_LAYOUT_DOC_BYTES = 10 * 1024 * 1024`) enforced *during* the read stream. Inlined JSON payloads are bounded to 1 MiB. Partition table bounded to $\le 128$ partitions, mount list $\le 128$ mounts, directories $\le 1024$. | Tested in `test_fs_layout_hardening.py` and `test_fs_layout_config_validation.py`. |
| **H2: File Type & Device Hygiene** | Input paths (`--spec`, `--fstab`, `--store`, `store_path`) must resolve to regular files. Named pipes (FIFOs), sockets, and character/block devices (`/dev/zero`, `/dev/urandom`) are rejected by file type before reading to prevent unbounded blocking or infinite memory allocation. | Proven by FL7 criterion in `tools/test_fs_layout_suites.py`. |
| **H3: Atomic Persistence & Temp Leak Prevention** | Layout store mutations are staged via `tempfile::Builder` using `.tmp.<pid>.<rand>`. Changes are synced via `fsync` before an atomic `os.replace` / `std::fs::rename`. In case of write or validation failure, staged files are removed to prevent directory clutter. | Verified by FL7 persistence tests. |
| **H4: Standard Result Envelopes** | All CLI commands produce uniform JSON envelopes `{"code": 0, "data": ..., "error": null}` on success and `{"code": 1, "data": null, "error": "<msg>"}` on operational/validation error, with exit codes `0` (success), `1` (validation/IO error), or `2` (argument error). Never silent failure. | Verified by FL3 / FL9 CLI wire suites. |
| **H5: Fail-Closed Audit Trail (ADR-0035 §F-2)** | Every execution path (successful mutation, validation refusal, serde deserialization error, PEP gate refusal) emits an honest SHA-256 hash-chained SQLite WAL audit row containing tool name, target layout ID, caller context, and exact outcome (`ok`, `error`, or `refused`). | Verified by FL4 and FL8 MCP contract suites. |
| **H6: Resource & Connection Cleanup** | Database connections to the SQLite WAL audit ring and temporary memory buffers are managed strictly via Rust RAII guards (`Drop`). No dangling connection handles or leaked descriptors on error exits. | Verified in Rust unit and integration tests. |

---

## 2. Verification Battery Summary

- Hardening test suite `test_fs_layout_hardening.py`: **PASS**
- Config validation test suite `test_fs_layout_config_validation.py`: **PASS**
- Core service unit tests `test_fs_layout_service.rs`: **PASS**
- Criteria FL7 & FL9 in `test_fs_layout_suites.py`: **PASS**

---

## 3. Acceptance Confirmation

- [x] Failure modes produce explicit, auditable errors.
- [x] No temp/connection leaks on the error path.
