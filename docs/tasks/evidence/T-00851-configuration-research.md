# T-00851 — Regression Triage / Configuration: Research

## 1. Objectives & Context
- **Context**: `Regression Triage / configuration` (T-00851..T-00860) defines configuration parameters and schemas for regression triage behavior across CLI, MCP, and core services.
- **Configurable Dimensions**:
  - `max_store_bytes`: Maximum allowed size of the serialized triage JSON file (default: 1,048,576 bytes / 1 MiB).
  - `default_severity`: Fallback severity when none is specified (`Blocker`, `Critical`, `Major`, `Minor`; default: `Critical`).
  - `auto_ingest_suites`: List of test suite identifiers or globs allowed for automatic CI summary ingestion.
  - `retention_days`: Maximum age for resolved regression records before rotation/archiving (default: 90 days).
  - `notify_blockers`: Boolean flag controlling immediate notification emission on blocker triage discovery.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Existing Configuration Pattern | Fact | `LedgerConfig` (`aiosh-core::ledger_config`) and `ToolchainManifest` provide serde-based deterministic JSON schemas with default fallback. |
| Invariant Bounds | Fact | Config files must enforce bounded sizes (<64 KiB) and valid JSON serialization. |
| Environment Override | Fact | Environment variable `AIOS_TRIAGE_CONFIG` allows overriding the configuration file path. |

## 3. Decisions Needed
- Create `code/aiosh-rust/aiosh-core/src/triage_config.rs` with `TriageConfig`.
- Support `--config <path>` in `aiosh triage` CLI and optional `config_path` in MCP tools.
- Add criterion `T5` to `tools/test_triage_suites.py`.
