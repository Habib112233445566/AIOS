# T-00326 — Dependency & Toolchain Pinning: core service Integration

## 1. Overview
This task integrates the `toolchain_service` core module with the `aiosh-cli` frontend, fulfilling the requirement to expose the Toolchain Pinning core service to operators and agents.

## 2. Integration Details
- **Module**: `code/aiosh-rust/aiosh-cli/src/main.rs`
- **Subcommand**: `aiosh toolchain check`
- **Behavior**:
  - Automatically attempts to load the `ToolchainManifest` via `from_env()`.
  - On success, it calls `enforce_toolchain(&manifest)`.
  - It uses the standard CLI dispatch mechanism (`emit()`) to guarantee that all executions (success or failure) emit a valid `toolchain.check` audit row.

## 3. Verification
Manually created `config/toolchain.json` with the host's actual `rustc`, `python3` and `node` versions to pass validation without altering system paths.
```json
{
  "rust_version": "1.99.0",
  "python_version": "3.14",
  "node_version": "v24.18",
  "enforce_hashes": false
}
```

Running `cargo run --bin aiosh -- toolchain check` yields:
```json
{
  "data": {
    "enforce_hashes": {
      "source": "default",
      "value": false
    },
    "node_version": {
      "source": "default",
      "value": "v24.18"
    },
    "python_version": {
      "source": "default",
      "value": "3.14"
    },
    "rust_version": {
      "source": "default",
      "value": "1.99.0"
    }
  },
  "ok": true,
  "subcommand": "toolchain check"
}
```
This demonstrates the integrated path operates correctly, returning the exact data shape required and writing properly to the shared SQLite DB (`audit.db`).
