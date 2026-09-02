# T-00372 — Dependency & Toolchain Pinning / security policy: Specification

## 1. Specification Overview
This specification formalizes the security policy and governance contracts for the Dependency & Toolchain Pinning epic.

## 2. Policy Contracts & Threat Mitigation

### A. Vulnerability Class Definition (`SECURITY.md`)
- **Policy Statement**: Bypassing, disabling, or tampering with toolchain manifest enforcement (`config/toolchain.json`, `rust-toolchain.toml`, `.python-version`) to execute unpinned, downgraded, or untrusted compiler/runtime binaries constitutes a security vulnerability in AIOS.
- **Classification**: Supply Chain Integrity & Hermetic Execution Defect.

### B. Audit Ring Emission Contract
- **CLI Actions**:
  - `aiosh toolchain check`: Emits audit record `toolchain.check` containing outcome (`success` | `error`), manifest properties, or error details.
  - `aiosh toolchain show`: Emits audit record `toolchain.show` containing outcome (`success` | `error`) and resolved manifest.
- **MCP Tool Calls**:
  - `aios.toolchain.check`: Intercepted and wrapped in `dispatch::recorded_call()`, writing structured audit rows.
  - `aios.toolchain.config.get`: Wrapped in `dispatch::recorded_call()`, writing structured audit rows.

### C. Automated Security Policy Verification (`tools/check_security_policy.py`)
- **Criteria Added/Updated**:
  - **S6 (Toolchain Pinning Policy Presence)**: Validates that `SECURITY.md` includes explicit references to toolchain pinning security reviews and integrity requirements.

## 3. Reused vs. New Interfaces
- **Reused**:
  - `aiosh-core::audit` / `aiosh-cli::emit` / `aiosh-mcp::dispatch::recorded_call` for audit row append.
  - `tools/check_security_policy.py` for automated policy regression testing.
- **New**:
  - Policy clauses in `SECURITY.md` covering toolchain pinning integrity and supply chain defense.
