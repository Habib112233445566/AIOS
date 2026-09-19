# T-01549 — Filesystem Layout configuration: Documentation

## Metadata
- **Task ID:** `T-01549`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / configuration
- **Status:** Complete — configuration contract documented in `docs/filesystem_layout.md`, including copy-pasteable operator and agent examples, D1..D9/FL1..FL6 rules, and honest limitations.
- **Date:** 2026-09-19
- **Depends on:** `T-01548` (Configuration Hardening)
- **Feeds:** `T-01550` (Configuration Verification & Evidence)
- **Artifacts:** `docs/tasks/evidence/T-01549-configuration-documentation.md`

---

## 1. Documentation Updates Summary

1. **Architecture & Guide (`docs/filesystem_layout.md`)**:
   - Updated with Sub-Epic 5 Configuration overview and complete evidence chain (`T-01541`..`T-01550`).
   - Detailed specification of validation rules D1..D9 and invariants FL1..FL6:
     - `FL1`: Exactly one root mount `/` with pass number 1.
     - `FL2`: Path hygiene (absolute, max 1024 chars, no control characters, no `.` or `..` traversals).
     - `FL3`: Mount hierarchy and parent-before-child ordering.
     - `FL4`: Mandatory CIS options (`nodev`, `nosuid`, `noexec`) on `/tmp` and `/dev/shm`.
     - `FL5`: Partition table boundaries ($\le 128$ partitions, ESP $\ge 100$ MiB formatted as `vfat`).
     - `FL6`: Mandatory required mount (`at least one mount must be marked required == true`).
     - Directory permission mode bounds: octal mask `1..=0o7777` (`mode: 0` rejected).
     - `symlink_target` UsrMerge confinement: must be relative under `usr/`.
     - `created_at` RFC 3339 UTC format; fstab `dump` frequency `0` or `1`.
     - Strict deserialization: `deny_unknown_fields` on specs and `FilesystemLayoutStore`.

2. **Operator CLI Usage Examples**:
   ```bash
   # Validate an external filesystem layout specification
   aiosh layout validate --spec config/layouts/custom_uefi.json

   # Check active or registered layout against all FL1..FL6 invariants
   aiosh layout check --standard

   # Register a new layout specification with an explicit store
   aiosh layout register --spec /etc/aios/layout.json --store /var/lib/aios/layouts.json
   ```

3. **Agent MCP Tool Call Examples**:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "tools/call",
     "params": {
       "name": "aios.fs_layout.validate",
       "arguments": {
         "spec": "{\"id\":\"custom-layout\",\"name\":\"Custom\",\"description\":\"Desc\",\"target_disk_min_bytes\":68719476736,\"partitions\":[],\"mounts\":[{\"path\":\"/\",\"device\":\"/dev/sda1\",\"fs_type\":\"ext4\",\"options\":[\"rw\"],\"dump\":0,\"pass\":1,\"required\":true}],\"directories\":[],\"created_at\":\"2026-09-19T00:00:00Z\"}"
       }
     }
   }
   ```

---

## 2. Honest Limitations Documented

- **Explicit Store Path Required:** Neither CLI nor MCP infers a default store path when modifying state. Operators and callers must supply `--store <path>` or `store_path` to avoid accidental mutation of untracked system state.
- **Single-Threaded Server & FIFO Blocking:** While non-regular files are rejected upfront, if an external tool feeds a slow stream over a regular file mount, single-threaded processing in `aiosh-mcp` is bound to the stdio event loop.
- **In-Memory Store Isolation:** In the absence of a `--store` flag, CLI commands operate on an in-memory instance populated with built-in presets (`standard_uefi`, `minimal_container`), and mutations are discarded upon process exit.

---

## 3. Acceptance Confirmation

- [x] Docs updated with working example.
- [x] Limitations are stated, not omitted.
