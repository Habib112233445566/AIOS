# T-00402 — Dependency & Toolchain Pinning / recovery & validation: Specification

## 1. Specification Overview
This specification formalizes the contracts, interfaces, and recovery semantics for Dependency & Toolchain Pinning in AIOS.

## 2. Core Operations

### A. Manifest Structural Validation (`validate_toolchain_manifest`)
- **Signature**: `pub fn validate_toolchain_manifest(path: &str) -> Result<ToolchainManifest, String>`
- **Behavior**:
  - Reads configuration file bounded by the 64KB max file size limit.
  - Parses JSON into strongly-typed `ToolchainManifest`.
  - Asserts that all declared version strings are non-empty and well-formed.
- **Return Value**: `Ok(ToolchainManifest)` if valid; `Err(String)` if file is missing, oversized, or malformed.

### B. Default Configuration Recovery (`recover_default_toolchain`)
- **Signature**: `pub fn recover_default_toolchain() -> ToolchainManifest`
- **Behavior**:
  - Instantiates in-memory canonical compile-time default `ToolchainManifest`:
    - `rust_version`: `"1.99.0"`
    - `python_version`: `"3.14"`
    - `node_version`: `Some("v24.18".into())`
    - `enforce_hashes`: `false`
- **Usage**: Serves as the fallback recovery baseline when disk configuration has suffered corruption.

### C. Toolchain Drift Reconciliation (`reconcile_toolchain`)
- **Signature**: `pub fn reconcile_toolchain(manifest: &ToolchainManifest) -> ToolchainReconciliationReport`
- **Behavior**:
  - Runs host probes for each toolchain component.
  - Generates status per runtime (`RustStatus`, `PythonStatus`, `NodeStatus`) with explicit drift status and remediation guidance (e.g. `"Run: rustup default 1.99.0"`).

## 3. PEP & Audit Policy
- Validation and reconciliation checks are read-only diagnostics and emit audit logs upon invocation.
- Any disk-modifying recovery (restoring default `config/toolchain.json`) is PEP-gated as `aios.toolchain.recover` (`is_irreversible`).
