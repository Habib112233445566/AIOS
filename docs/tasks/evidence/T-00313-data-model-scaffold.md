# T-00313 — Dependency & Toolchain Pinning: Data Model Scaffold

## Overview
We created the skeleton for the Dependency & Toolchain Pinning data model in `code/aiosh-rust/aiosh-core/src/toolchain_config.rs`.

## Scaffolded Interfaces
- `ToolchainManifest`: A struct holding `rust_version`, `python_version`, `node_version`, and `enforce_hashes`.
- `ToolchainManifest::from_env()`: Reads the config, falling back to OS environment.
- `ToolchainManifest::from_source()`: Reads via custom getter for dependency injection.
- JSON exporter endpoints (`to_json_with_sources`).

All functions are stubbed to return `unimplemented!()`.

## Validation
- The module was exposed via `pub mod toolchain_config;` in `lib.rs`.
- `cargo test -p aiosh-core` successfully compiles the workspace with zero warnings for this new module.
- A test stub `test_from_env_panics_in_scaffold` explicitly references the interface, fulfilling the acceptance criteria.
