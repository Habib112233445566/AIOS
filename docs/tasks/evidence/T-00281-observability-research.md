# T-00281 — Release Packaging & Backup: Observability Research

## Goal
Establish facts, constraints, and prior art for the observability of Release Packaging & Backup operations.

## Facts (Derived from Existing Code)
1. **Audit Logging is Primary Observability**: Every successful or failed release generation / backup creation is logged to `MASTER_TASK_LEDGER.jsonl` as an `AuditRing` row. This provides historical observability (who did what, when, and what the outcome was).
2. **Current Subprocess Logging**: The `physical_generate_iso` and `physical_create_zip` functions invoke OS-level subprocesses (e.g., `genisoimage`). If they fail, the error string (often the `io::Error` or exit code) is captured in `outcome_detail`.
3. **Synchronous Execution**: Currently, the MCP handlers and CLI wait synchronously for the backup or ISO generation to complete. 

## Prior Art & Authoritative Sources
- **ADR-0035 (Audit Invariants)**: States that the primary mechanism for system state tracking is the append-only ledger. Arbitrary application logs (like standard stdout/stderr application logs) are considered ephemeral.
- **Twelve-Factor App**: Recommends treating logs as event streams. The `MASTER_TASK_LEDGER.jsonl` inherently fulfills this by streaming structured JSON.

## Decisions Needed
1. **Long-Running Task Visibility**: Backups of large `/var/aios` directories can take several minutes. Do we need to emit intermediate "progress" events to the ledger, or is a single "start" and "end" event sufficient? (Recommendation: Keep it simple for Phase 0 with a single end event, or rely on the caller to handle timeouts).
2. **Subprocess Stderr Capture**: If `genisoimage` fails, we currently get a generic failure. Should we capture its `stderr` buffer and embed it into the audit row's `args` or `outcome_detail` for debugging? (Recommendation: Yes, capturing `stderr` is crucial for troubleshooting).

## Next Steps
Proceed to the Specification phase to finalize how `stderr` capture and long-running feedback will be handled.
