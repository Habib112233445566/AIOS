# T-00351 — Dependency & Toolchain Pinning / configuration: Research

## Current State

The data model and enforcement logic for toolchain pinning (`aiosh_core::toolchain_config::ToolchainManifest` and `aiosh_core::toolchain_service::enforce_toolchain`) have been successfully implemented and exposed via CLI (`aiosh toolchain check`) and MCP (`aios.toolchain.check`). 

However, the actual authoritative configuration files (the ecosystem files that enforce these rules natively, and the root `toolchain.json` payload) do not yet exist in the repository root. Currently, a mock/test configuration file exists at `code/aiosh-rust/config/toolchain.json`, which was used for validation during earlier tasks.

### Missing Configurations:
1. **Root `config/toolchain.json`**: The central source of truth for AIOS tools.
2. **`rust-toolchain.toml`**: The Rust-native way to pin the compiler version, ensuring `cargo` uses the exact version defined in `toolchain.json`.
3. **`.python-version`**: The Python ecosystem file for `pyenv` or `uv` to pick the correct runtime.
4. **`.nvmrc`**: The Node.js ecosystem file for `nvm`.

## Ecosystem Version Targeting
Based on local system constraints (and for Phase 0 alignment), the targeted versions are:
- **Rust**: `1.99.0` (or `nightly` if stable is unavailable, which is currently the case on Windows). To avoid divergence, we will pin `rust-toolchain.toml` to `nightly` and set `toolchain.json` to expect the exact version output `1.99.0-nightly` or simply `1.99.0` if `enforce_toolchain` strips channel strings.
- **Python**: `3.14`
- **Node.js**: `v24.18`

## Action Plan for the Sub-Epic
1. **Specification (T-00352)**: Define the canonical JSON layout for the root `config/toolchain.json` and how ecosystem files will map to it.
2. **Scaffold & Implementation (T-00353 & T-00354)**: Create the actual configuration files (`rust-toolchain.toml`, `.python-version`, `.nvmrc`, and `config/toolchain.json`) in the repository root. Move the temporary one from `code/aiosh-rust`.
3. **Unit / Integration Testing (T-00355 & T-00356)**: Ensure `aiosh toolchain check` parses the new root configuration when invoked from any directory (if possible), or test it directly.
4. **Security & Hardening (T-00357 & T-00358)**: Ensure the JSON configuration is strictly formatted and cannot be tampered with unintentionally.
5. **Documentation & Verification (T-00359 & T-00360)**: Update README/spec and run final smoke tests.
