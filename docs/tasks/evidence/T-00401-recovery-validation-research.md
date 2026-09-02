# T-00401 — Dependency & Toolchain Pinning / recovery & validation: Research

## 1. Goal
Establish facts, constraints, drift scenarios, and prior art for the recovery and validation mechanisms of Dependency & Toolchain Pinning in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase):
1. **Manifest File Dependencies**: AIOS toolchain enforcement depends on `config/toolchain.json`, `rust-toolchain.toml`, and `.python-version`.
2. **Drift Failure Modes**:
   - Corrupted or invalid JSON in `config/toolchain.json`.
   - Missing configuration files when `$AIOSH_TOOLCHAIN_CONFIG` points to non-existent paths.
   - Host toolchain drift (compiler upgraded or downgraded outside of pinned versions).
3. **Validation Capabilities**:
   - Pure structural validation of manifest syntax and schema without spawning host compiler binaries.
   - Host environment conformance validation comparing detected binary versions with active pins.
4. **Recovery Capabilities**:
   - Restoring canonical default toolchain configuration (`ToolchainManifest::default()`).
   - Emitting actionable remediation instructions for operators (e.g., `rustup default 1.99.0` or updating `.python-version`).

### Assumptions:
1. Manifest validation should be runnable in airgapped or offline CI environments without requiring internet access or compiler execution.
2. In-memory fallback recovery must protect services from crashing when local config files are corrupted.

## 3. Prior Art & Authoritative Standards
- **Rustup Toolchain Diagnostics**: `rustup check` and `rustup toolchain list` for verifying active vs desired compiler channels.
- **Cargo Lockfile Verification**: `cargo check --locked` enforcing that build manifests match lockfiles without network mutation.
- **Pyenv Local Validation**: `.python-version` verification and reconciliation.

## 4. Decisions Needed
1. **Validation Surface**: Should toolchain validation support a `--manifest-only` dry-run mode that validates JSON structure without probing host binaries?
   - *Decision*: Yes, support dry-run structural validation alongside full environment checking.
2. **Corrupted Config Recovery**: When `config/toolchain.json` is missing or corrupted, should `aiosh` gracefully fall back to compile-time defaults?
   - *Decision*: Yes, fallback with an explicit warning/diagnostic log.

## 5. Next Steps
Advance to Specification (T-00402) to formalize the recovery and validation contracts and API endpoints.
