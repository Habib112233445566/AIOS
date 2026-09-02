# T-00738 — Secrets & Access Hygiene / CLI surface: Hardening

## 1. Hardening Deliverables
- **Bounded Resource Consumption**:
  - File scanner enforces `DEFAULT_MAX_SECRET_FILE_BYTES` (16 MiB) and customizable `--max-bytes`.
  - Line scanner clamps scan targets to `MAX_LINE_SCAN_LENGTH` (4096 bytes) preventing CPU/memory spikes on huge minified strings.
  - Early-exit binary null-byte sniffing skips large compiled objects, media, and blobs.
- **Defensive Error Handling**:
  - Explicit error messages returned via `err_out` and standard JSON envelope with `{"ok": false, ...}`.
  - Syntax/usage errors return exit code `2` with actionable usage messages.
  - Non-existent files or directories fail loudly with audited error rows.
- **Zero Resource Leakage**:
  - Direct scoped file handles automatically closed by Rust RAII upon function return.
  - SQLite WAL audit writes cleanly flushed.
