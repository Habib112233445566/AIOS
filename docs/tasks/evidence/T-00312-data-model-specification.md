# T-00312 — Dependency & Toolchain Pinning: Data Model Specification

## 1. Overview
The "Dependency & Toolchain Pinning" data model guarantees that any AIOS build, task execution, or agent sandbox runs under explicitly pinned toolchains, preserving reproducibility and protecting against environmental poisoning. 

This specification defines the `ToolchainManifest` contract within `aiosh-core`. Rather than replacing `Cargo.lock` or `requirements.txt`, this data model acts as the **governance overlay**, verifying that ecosystem-specific locks and runtime toolchains match the AIOS secure baseline.

## 2. Inputs & Data Model Contract

The core data structure is `ToolchainManifest`.

### 2.1 Interface Definition
```rust
pub struct ToolchainManifest {
    pub rust_version: String,           // e.g., "1.98.0"
    pub python_version: String,         // e.g., "3.10.12"
    pub node_version: Option<String>,   // e.g., "20.10.0"
    pub enforce_hashes: bool,           // Must be true for production builds
}
```

### 2.2 Input Source
The manifest is loaded from `$AIOSH_TOOLCHAIN_CONFIG` (falling back to `config/toolchain.json`).
**Format:**
```json
{
  "rust_version": "1.80.0",
  "python_version": "3.10.12",
  "node_version": "20.10.0",
  "enforce_hashes": true
}
```

## 3. Behaviors and Outputs

### 3.1 Happy Path
1. The AIOS core (`aiosh-core::toolchain_config`) loads the JSON file via `ToolchainManifest::from_env()`.
2. The manifest successfully parses integers and strings without error.
3. The component exposes validation functions to compare host environments against these exact strings (e.g., matching the output of `rustc --version`).

### 3.2 Error Cases
1. **Missing or Malformed JSON**: Fails synchronously with an actionable string error describing the syntax or path failure.
2. **Empty Strings**: If `rust_version` or `python_version` are empty, it returns a hard error (e.g., `invalid toolchain config: rust_version cannot be empty`).
3. **File Size/Traversal constraints**: The JSON reader will apply the standard `aiosh-core` bounding checks (file size capped at 64KB, no relative `../` or absolute path breakouts allowed for the config path), mirroring the `release_config.rs` pattern.

## 4. Reused vs New Interfaces
- **Reused**: The pattern of bounded config parsing (`parse_usize`, environment variable fallbacks, `to_json_with_sources`) perfectly mirrors `CiConfig` and `ReleaseConfig`.
- **New**: The `ToolchainManifest` struct is a new schema specifically defining runtime toolchain boundaries.

## 5. Persistence & Audit Effects
The config parser itself is read-only and memory-bound. However, downstream task execution components that *consume* this data model will emit `AuditRow` records if an agent attempts to execute a build using a non-compliant toolchain (preventing silent drift).

## 6. Authoritative Scope
This specifies the *Data Model* only. Ecosystem integration (actually running `python --version` or invoking `cargo`) belongs in the subsequent Core Service components. This data model specifies strictly *what* the rules are.
