# T-00396 — Dependency & Toolchain Pinning / documentation: Integration

## 1. Integration Scope
This task integrates the complete Dependency & Toolchain Pinning documentation into the primary developer and operator knowledge repository at `docs/README.md`.

## 2. Integration Details
- **Primary Entrypoint**: Integrated directly under `## Dependency & Toolchain Pinning (T-00311..T-00400)` in `docs/README.md`.
- **Surface Cross-Referencing**:
  - Direct shell commands for CLI interactions (`aiosh toolchain show`, `aiosh toolchain check`).
  - Copy-pasteable JSON-RPC tool definitions for MCP clients (`aios.toolchain.config.get`, `aios.toolchain.check`).
  - Configuration environment variable definitions (`AIOSH_TOOLCHAIN_CONFIG`).
  - PEP security gating boundaries and audit invariants.
  - Runtime telemetry collection and observability troubleshooting via `aiosh audit tail`.

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
- In-tree links, references, and anchors resolve cleanly.
