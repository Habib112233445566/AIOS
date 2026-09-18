# T-01519: Filesystem Layout - Core Service: Documentation

## Metadata
- **Task ID:** `T-01519`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (10/10) — Core Service Documentation
- **Dependencies:** `T-01518` (Core Service Hardening)
- **Next Task:** `T-01520` (Filesystem Layout / core service: Verification & Evidence)

---

## 1. Documentation Updates Delivered

1. **System Guide Updated**: [`docs/filesystem_layout.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/filesystem_layout.md)
   - Updated system architecture diagram with `FilesystemLayoutService`, `FilesystemLayoutStore`, probe engine, diff engine, and atomic persistence.
   - Documented Core Service invariants `CS1..CS5`.
   - Added operator CLI documentation for `aiosh layout list`, `aiosh layout probe`, and `aiosh layout diff`.
   - Added MCP documentation for `aios.fs_layout.list`, `aios.fs_layout.probe`, and `aios.fs_layout.diff`.
   - Honestly stated constraints: user-space declarative control, cardinality caps (128 partitions, 128 mounts, 1024 dirs), 10 MiB ingestion ceiling, and built-in preset immutability.
   - Linked task evidence records for `T-01511..T-01518`.

---

## 2. Example Usages

### CLI Command Example
```bash
# List all registered layout profiles
aiosh layout list

# Probe target device feasibility (100 GiB)
aiosh layout probe --bytes 107374182400

# Compute differential comparison
aiosh layout diff aios-uefi-standard-v1 aios-container-minimal-v1 --json
```

### MCP Tool Example
```json
{
  "name": "aios.fs_layout.probe",
  "arguments": {
    "layout_id": "aios-uefi-standard-v1",
    "target_disk_bytes": 107374182400
  }
}
```

Acceptance criteria satisfied:
- Docs updated with working examples.
- Limitations are stated, not omitted.
