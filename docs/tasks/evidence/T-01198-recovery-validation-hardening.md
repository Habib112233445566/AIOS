# T-01198: Base Image Build Recovery & Validation Hardening

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Recovery & Validation  
**Task ID:** T-01198  

## 1. Hardening Deliverables
1. **Size Caps & Limits**:
   - On-disk image registry file size strictly capped at 10 MiB in `ImageStore::load_from_path`.
   - Store path length bounded to 4096 characters in CLI and MCP tool calls.
   - Manifest IDs bounded to 128 characters, restricted to printable ASCII graphic characters.
   - Package lists capped at 1024 packages per manifest to prevent heap exhaustion.
   - Size budget ceiling enforced at 100 GiB.
2. **Input Sanitization**:
   - Rejects null bytes (`\0`) and control characters (`< 0x20` or `0x7f`) across IDs, paths, and package names.
3. **Structured Failure Envelopes**:
   - Unhealthy states and parse errors emit ADR-0035 standard JSON error structures.
4. **Honest Audit Logging**:
   - Failed checks and repair actions write honest audit rows (`status="failure"` or exit code 1) to SQLite WAL (`audit.db`).
5. **Non-Destructive Safety (RV4)**:
   - Preserves corrupted data in timestamped forensic backup files (`.bak.<timestamp>`) before overwriting with clean default manifests.
