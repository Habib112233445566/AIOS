# T-00393 — Dependency & Toolchain Pinning / documentation: Scaffold

## 1. Scaffold Scope
This task establishes the structural framework and section hierarchy for the Dependency & Toolchain Pinning documentation in `docs/README.md`.

## 2. Structural Hierarchy
The following section layout is established and verified in `docs/README.md`:
- `## Dependency & Toolchain Pinning (T-00311..T-00400)`
  - `Usage Example (CLI)`: `aiosh toolchain show` and `aiosh toolchain check`
  - `Usage Example (MCP)`: `aios.toolchain.config.get` and `aios.toolchain.check`
  - `Configuration`: `$AIOSH_TOOLCHAIN_CONFIG` environment overrides & 64KB limits
  - `Security Policy (PEP Gating & Audit)`: PEP grant enforcement & audit ring emission
  - `Observability & Telemetry Diagnostics`: `ToolchainTelemetry`, provenance tagging, and string clamping
  - `Automated Tests`: CLI/MCP smoke test invocations & CI suites
  - `Known Limitations (Toolchain Pinning)`: Subprocess timeouts, optional Node runtime, and size caps
  - `Evidence`: Task ledger evidence links

## 3. Invariant Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
