# T-00394 — Dependency & Toolchain Pinning / documentation: Implementation

## 1. Implementation Scope
This task implements the comprehensive operator and agent reference manual for the Dependency & Toolchain Pinning epic inside `docs/README.md`.

## 2. Documented Systems & Surfaces
1. **CLI Commands**: `aiosh toolchain show` and `aiosh toolchain check [--config <path>]`.
2. **MCP Tool Invocations**: `aios.toolchain.config.get` and `aios.toolchain.check`.
3. **Configuration Schema**: Details on `$AIOSH_TOOLCHAIN_CONFIG`, `config/toolchain.json`, `rust-toolchain.toml`, `.python-version`, and 64KB bounds.
4. **Security Policy**: PEP gating requirements for state-mutating actions (`aios.toolchain.set`), token verification, and immutable audit logs.
5. **Observability**: `ToolchainTelemetry`, provenance source tags (`source: "default" | "file" | "env"`), mismatch error logs in `outcome_detail`, and 512-byte string clamping.
6. **Automated Smoke Invocations**: CLI & MCP smoke runner commands and CI integration.
7. **Known Limitations**: Honest documentation of subprocess timeouts, 64KB payload caps, and optional Node runtime handling.

## 3. Invariant Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
