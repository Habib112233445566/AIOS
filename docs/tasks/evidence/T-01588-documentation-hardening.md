# T-01588 — Filesystem Layout documentation: Hardening

## Metadata
- **Task ID:** `T-01588`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / documentation
- **Status:** Complete — hardened documentation testing against timeouts, malformed input, and resource leaks.
- **Date:** 2026-09-19
- **Depends on:** `T-01587` (Security Review)
- **Feeds:** `T-01589` (Documentation)
- **Artifacts:** `docs/tasks/evidence/T-01588-documentation-hardening.md`, `docs/tasks/evidence/T-01588-hardening.md`

---

## 1. Hardening Measures Implemented

1. **Subprocess Timeouts & Process Containment**:
   - `aiosh layout --help` execution bounded to 60s timeout.
   - `aiosh-mcp tools/list` JSON-RPC stdio exchange bounded to 30s timeout with guaranteed `p.kill()` and `p.wait()` on expiry.

2. **Bounded File Parsing & JSON Sanitization**:
   - `docs/filesystem_layout.md` parsing uses bounded regex matching.
   - Code block JSON parser sanitizes illustrative ellipses (`[ ... ]`) and placeholder syntax (`<TOKEN>`) safely without executing dynamic code.

3. **Link Verification Safety**:
   - Path resolution confirms all referenced evidence files exist within the repository boundaries without permitting directory traversal.

4. **Explicit Failure Reporting**:
   - Any missing subcommands, broken links, or invalid JSON snippets raise explicit assertion errors with line/block context (never failing silently).

---

## 2. Acceptance Confirmation

- [x] Failure modes produce explicit, auditable errors in the standard result envelope.
- [x] No temp file, socket, or child process leaks on error paths.
