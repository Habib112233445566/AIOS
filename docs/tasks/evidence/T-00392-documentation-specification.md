# T-00392 — Dependency & Toolchain Pinning / documentation: Specification

## 1. Specification Overview
This specification defines the documentation architecture, content structure, and example specifications for Dependency & Toolchain Pinning in AIOS.

## 2. Documentation Architecture (`docs/README.md`)

### Target Section: `## Dependency & Toolchain Pinning (T-00311..T-00400)`
1. **Overview & Pin Manifests**:
   - Explanation of pinned toolchains (`config/toolchain.json`, `rust-toolchain.toml`, `.python-version`).
2. **CLI Surface Reference**:
   - Syntax and options for `aiosh toolchain show` and `aiosh toolchain check`.
3. **MCP Tool Reference & JSON Examples**:
   - `aios.toolchain.config.get`: Returns active manifest with provenance.
   - `aios.toolchain.check`: Executes host binary version enforcement.
4. **Configuration Overrides**:
   - Use of `$AIOSH_TOOLCHAIN_CONFIG` environment variable and 64KB size cap.
5. **Security Policy & PEP Gating**:
   - Read-only vs. mutating operations (`aios.toolchain.set`), active PEP grant requirements, and fail-closed behavior.
6. **Observability & Telemetry**:
   - `ToolchainTelemetry` structure, 512-byte string clamping, and `aiosh audit tail` diagnostic access.
7. **Automated Testing**:
   - Invocations for `test_toolchain_cli_smoke.py`, `test_toolchain_mcp_smoke.py`, and CI suites.
8. **Known Limitations**:
   - Subprocess timeouts (15s), optional Node runtime, and size caps.

## 3. Structural Validation
- Documentation must conform to all C1..C6 invariants checked by `tools/check_task_docs.py`.
