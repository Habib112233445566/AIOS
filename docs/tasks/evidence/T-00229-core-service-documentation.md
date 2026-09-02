# T-00229 — Phase 0 — Release Packaging & Backup / Core Service: Documentation

## Goal
Document the core service of Release Packaging & Backup for operators and agents.

## Completion Notes
1. **README Update (`docs/README.md`)**:
   - Upgraded the "Release Packaging & Backup" epic scope note in the README from the old `T-0211 - T-0220` bound to the finished `T-0211 - T-0229` bound.
   - Added a copy-pasteable JSON payload for invoking the `aios.backup.create` MCP tool.
   - Honestly documented the primary limitations of the implementation:
     - ISO generation operates in a mocked state under Windows due to `genisoimage` missing dependencies, but fully supports the audit ring pipeline.
     - Path constraints (file size max 2GB, symlinks skipped) enforce deterministic backup completion.
     - Cross-substrate parity notes: Python operates perfectly on Windows; Rust is maintained synchronously but drops some functionality until C dependency issues (`libc`, `zip`) are natively supported or ported completely via target conditionals.

## Acceptance Criteria Verified
- [x] Docs updated with working examples (MCP JSON structure).
- [x] Limitations are stated honestly and not omitted (mock files, symlink skipping, 2GB limits).
- [x] Task evidence files linked/documented.
