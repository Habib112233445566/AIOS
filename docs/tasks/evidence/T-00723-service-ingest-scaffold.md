# T-00723 — Secrets & Access Hygiene / core service: Scaffold

## 1. Scaffold Deliverables
- Created `code/aiosh-rust/aiosh-core/src/secrets_service.rs` declaring:
  - `scan_file_for_secrets`: Single file inspection with binary null-byte check, size limit, and pattern matching.
  - `scan_workspace_for_secrets`: Recursive workspace crawler skipping ignored build directories (`.git`, `target`, `node_modules`, `.venv`, `dist`).
  - Constants: `DEFAULT_MAX_SECRET_FILE_BYTES`, `MAX_LINE_SCAN_LENGTH`, `DEFAULT_IGNORED_DIRS`.
- Registered `pub mod secrets_service;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## 2. Compilation Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml` compiled cleanly.
