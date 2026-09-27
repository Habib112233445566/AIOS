# T-02323: Audit Chain Extensions CLI Surface Scaffold

## Overview
This task scaffolds the CLI surface for Audit Chain Extensions in `code/aiosh-rust/aiosh-cli/src/main.rs`.

## Scaffolded Commands
1. **`cmd_audit` Dispatcher**: Extended match arms to route subcommands:
   - `query` -> `cmd_audit_query`
   - `ancestry` -> `cmd_audit_ancestry`
   - `sign-verify` -> `cmd_audit_sign_verify`
   - `inspect` -> `cmd_audit_inspect`
2. **Function Skeletons**:
   - `cmd_audit_query`: Parses `--session`, `--trace`, `--actor`, `--tool`, `--parent`, `--limit` flags and wraps `AuditChainService::query_events`.
   - `cmd_audit_ancestry`: Parses positional hash and `--depth` flag and wraps `AuditChainService::trace_ancestry`.
   - `cmd_audit_sign_verify`: Parses positional hash and wraps `AuditChainService::verify_event_signature`.
   - `cmd_audit_inspect`: Parses positional hash and wraps `AuditChainService::get_row_by_hash`.
3. **Compilation**: `cargo check -p aiosh-cli` compiles cleanly with zero warnings.
