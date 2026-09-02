# T-00381 — Dependency & Toolchain Pinning / observability: Research

## 1. Goal
Establish facts, constraints, operational diagnostics, and prior art for the observability of Dependency & Toolchain Pinning in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Codebase):
1. **Audit Ring Event Sourcing**: Every toolchain invocation (`aiosh toolchain check`, `aiosh toolchain show`, `aios.toolchain.check`, `aios.toolchain.config.get`) writes a structured, hash-chained audit row to the SQLite WAL database.
2. **Configuration Provenance**: `ToolchainManifest::to_json_with_sources()` attaches explicit source tagging (`source: "default" | "env" | "file"`) to every configuration parameter (`rust_version`, `python_version`, `node_version`, `enforce_hashes`), exposing provenance telemetry.
3. **Subprocess Probing Metrics**: Subprocess execution for host binary verification (`rustc -V`, `python3 -V`, `node -v`) captures execution duration and exit codes with a 15-second wall-clock timeout bound.
4. **Structured JSON Output Envelopes**: Both CLI and MCP surfaces emit canonical JSON responses with explicit `ok`, `subcommand` / `tool`, `data` / `config`, and `error` fields.

### Assumptions:
1. Operators and automated agents need immediate visibility into configuration source origins (e.g. distinguishing whether a version constraint came from `config/toolchain.json` vs. `$AIOSH_TOOLCHAIN_CONFIG`).
2. Detailed version mismatch diagnostics in audit logs simplify supply chain and CI debugging.

## 3. Prior Art & Authoritative Sources
- **ADR-0035 §F-2 (Audit & Ledger Invariants)**: System observability relies on immutable, append-only ledger entries rather than unstructured log streams.
- **OpenTelemetry Semantic Conventions**: System/runtime attribute specifications (capturing compiler names, detected versions, and host runtime metadata).
- **Twelve-Factor App (Logs as Event Streams)**: Structured JSON event emission over standardized streams.

## 4. Decisions Needed
1. **Provenance Field Standardization**: Standardize the structured schema for provenance metadata in audit rows (`data.source` / `data.value`).
2. **Mismatch Diagnostics**: Ensure version mismatches clearly report detected vs expected versions in `outcome_detail` for operator debugging via `aiosh audit tail`.

## 5. Next Steps
Advance to Specification (T-00382) to define the observability schema, audit row format, and query commands.
