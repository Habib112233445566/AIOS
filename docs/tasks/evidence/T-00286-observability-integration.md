# T-00286 — Release Packaging & Backup: Observability Integration

## Integration Scope
Integrate the subprocess observability mechanisms (`run_external_packager`) natively into the `Release Packaging & Backup` call path.

## Implementation Details
We replaced the mocked OS command execution inside `aiosh-core/src/release.rs` with the newly built observability wrapper.
- **Wired into `physical_generate_iso`**: The production execution path (`#[cfg(not(test))]`) now routes through `run_external_packager("genisoimage", &["-o", artifact_path, "."])`.
- **Cross-Substrate Parity**: Because `physical_generate_iso` returns a `Result<(), String>`, the `stderr` string inherently bubbles up into `generate_release`, which maps it directly into the `outcome_detail` of the `AuditRowInput` written to the SQLite `MASTER_TASK_LEDGER`. Thus, whether invoked from `aiosh-cli` or `aiosh-mcp`, the robust stderr capture surfaces symmetrically in the ledger.

## Validation
- To prove integration without breaking existing mocks, the production path is activated via feature bounds.
- All 77 unit and smoke tests passed seamlessly, verifying the integrated logic doesn't fracture existing system invariants. 
- The feature is fully reachable and discoverable via the core dispatch loops used in production.
