# T-00352 — Dependency & Toolchain Pinning / configuration: Specification

## Goal
Specify the exact format and location for the root configuration files enforcing dependency and toolchain pinning in AIOS.

## Specifications

### 1. Root `config/toolchain.json`
This is the master configuration parsed by `aiosh_core::toolchain_config::ToolchainManifest`.
- **Location**: `config/toolchain.json`
- **Format**:
  ```json
  {
    "rust_version": "1.99.0",
    "python_version": "3.14",
    "node_version": "v24.18",
    "enforce_hashes": false
  }
  ```
- **Note**: `enforce_hashes` is set to `false` for now, pending strict lockfiles across all languages in later phases.

### 2. Rust Toolchain Pin (`rust-toolchain.toml`)
- **Location**: `rust-toolchain.toml` (Root directory)
- **Format**:
  ```toml
  [toolchain]
  channel = "nightly"
  ```
- **Rationale**: Currently, Rust 1.99.0 is only available on nightly. `cargo` and `rustup` will natively read this file and auto-switch to the nightly channel. `aiosh toolchain check` handles version verification.

### 3. Python Toolchain Pin (`.python-version`)
- **Location**: `.python-version` (Root directory)
- **Format**:
  ```
  3.14
  ```
- **Rationale**: Standard ecosystem format parsed natively by `pyenv`, `uv`, and `rye`.

### 4. Node Toolchain Pin (`.nvmrc`)
- **Location**: `.nvmrc` (Root directory)
- **Format**:
  ```
  v24.18
  ```
- **Rationale**: Standard ecosystem format parsed natively by `nvm`.

## Migration
The temporary `code/aiosh-rust/config/toolchain.json` must be removed to avoid shadowing the root configuration. All tests should run against the root config.
