# T-01224: Package Management - CLI Surface: Implementation

## Metadata
- **Task ID:** `T-01224`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Package Management CLI Surface Implementation
- **Status:** Complete

## 1. Implementation Deliverables
- Implemented full functionality for `aiosh package apply` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - Input resolution: accepts either `--actions <file_or_json>` or `--plan <file_or_json>`, loading content from file paths or inline JSON with strict 1 MiB payload protection.
  - Plan execution: plans transaction if raw actions are provided (`store.plan_transaction`), or deserializes pre-planned transactions.
  - Dry run preview: respects `--dry-run` flag, executing state changes transiently without writing to disk.
  - Store persistence: when not in dry-run mode, persists state changes to the target `--store <path>` atomically using temporary file writes and RAII cleanup.
  - Execution summary: prints formatted terminal summaries (packages installed, removed, upgraded, and net delta bytes) or structured JSON result envelopes when `--json` is supplied.
  - Audit integration: records all execution outcomes via `classify_and_emit` into the SQLite WAL ring (`audit.db`).
- Test suite enhancements in `test_cmd_package_flow`:
  - Verified `apply` missing argument error handling (exit code 2).
  - Verified `apply` dry run from raw actions (exit code 0).
  - Verified `apply` dry run from pre-planned transaction JSON (exit code 0).
  - Verified `apply` real transaction execution and persistent disk state roundtrip (exit code 0).
