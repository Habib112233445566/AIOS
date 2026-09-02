# T-00389 — Dependency & Toolchain Pinning / observability: Documentation

## 1. Documentation Scope
This task documents the observability features, runtime telemetry structure, and diagnostic capabilities of Dependency & Toolchain Pinning for operators and AI agents.

## 2. Documentation Updates
- **File Modified**: `docs/README.md`
- **Section Added**: `Observability & Telemetry Diagnostics` under `## Dependency & Toolchain Pinning (T-00311..T-00390)`

### Summary of Documented Capabilities:
1. **Provenance Metadata**: `ToolchainManifest::to_json_with_sources()` attaches source attribution (`source: "default" | "file" | "env"`) to version fields in CLI and MCP responses.
2. **Audit Telemetry & Troubleshooting**: All probe results and mismatch diagnostics are logged to `outcome_detail` in the SQLite WAL audit ring, accessible via `aiosh audit tail`.
3. **Log Inflation Hardening**: Host binary version probe outputs are clamped to 512 bytes with explicit `[TRUNCATED]` markers.

## 3. Acceptance Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
- Example commands and operational guidance verified.
