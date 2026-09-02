# T-00276 — Security Policy: Integration

## Integration Scope
Integrate the Release Packaging & Backup security policy with the broader AIOS policy enforcement mechanisms.

## Implementation & Integration
- **`is_irreversible` Integration**: The security policy natively hooks into `aiosh-core/src/pep.rs`. The CLI and MCP tools that utilize the `aiosh-core` dispatch routines automatically inherit this gating. 
- **Dispatcher Parity**: Because `aiosh-mcp` and `aiosh-cli` defer their heavy state mutations (like dispatch and audits) to the core Rust bindings (either via direct library linkage or through FFI/subprocess bounds depending on substrate), the additions of `aios.backup.*` and `aios.release.*` to the irreversible list provide system-wide coverage.
- **Audit Ledger Cross-Substrate Parity**: Any successful or failed generation attempts by `ReleaseCtx::generate_release` and `create_backup` immediately serialize canonical JSON to the shared SQLite WAL database, ensuring both Python and Rust layers observe the exact same artifact history.

## Validation
- The `rust_smoke` suite (and underlying `cargo test` run in task 274) asserts the integrated behavior end-to-end within the library scope.
- Integration is functionally complete and production-ready.
