# T-00382 — Dependency & Toolchain Pinning / observability: Specification

## 1. Specification Overview
This specification formalizes the observability contracts, audit event schemas, and error diagnostics for Dependency & Toolchain Pinning in AIOS.

## 2. Telemetry and Event Schemas

### A. Audit Event Schema (Happy Path)
- **Actor**: `caller` identity (e.g. `agent:default` or `operator:cli`).
- **Tool**: `toolchain.check` | `toolchain.show` | `aios.toolchain.check` | `aios.toolchain.config.get`.
- **Outcome**: `"success"`.
- **Outcome Detail**: Formatted summary containing active toolchain manifest details (e.g., `"Toolchain check passed: rust=1.99.0, python=3.14, node=v24.18"`).

### B. Audit Event Schema (Failure Path)
- **Actor**: `caller` identity.
- **Tool**: `toolchain.check` | `aios.toolchain.check` | `aios.toolchain.set`.
- **Outcome**: `"error"` | `"refused"`.
- **Outcome Detail**: Detailed error diagnostics, including:
  - Binary missing from PATH: `"rustc: binary not found in PATH"`.
  - Version mismatch: `"toolchain mismatch: rustc version mismatch (detected 'rustc 1.80.0', expected '1.99.0')"`.
  - Refusal: `"Action 'aios.toolchain.set' is irreversible and requires an active PEP grant"`.

### C. Provenance Telemetry Output
- The `aios.toolchain.config.get` and `aiosh toolchain show` APIs emit structured JSON containing source attribution per key:
  ```json
  {
    "rust_version": { "value": "1.99.0", "source": "file" },
    "python_version": { "value": "3.14", "source": "file" },
    "node_version": { "value": "v24.18", "source": "file" },
    "enforce_hashes": { "value": false, "source": "file" }
  }
  ```

## 3. Reused vs. New Interfaces
- **Reused**:
  - `aiosh-core::audit::AuditRing` and `AuditRowInput` for immutable event persistence.
  - `aiosh-core::toolchain_config::ToolchainManifest` for provenance extraction.
- **AIOS Specific**:
  - Structured provenance attribution mapping in JSON envelope.
