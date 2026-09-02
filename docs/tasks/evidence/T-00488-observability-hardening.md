# T-00488 — Documentation Index Control / observability: Hardening

## 1. Hardening Overview
This task hardens the telemetry and observability subsystem of Documentation Index Control against runtime panics, memory bloat, silent error masking, and unhandled boundary inputs.

## 2. Hardening Measures
1. **Fallback Calculation Resilience**:
   - `collect_doc_index_telemetry` operates reliably with or without a prior `DocLinkValidationReport`, computing outbound link totals directly from the manifest entries when a full link scan is omitted.
2. **Schema Invariant Bounds**:
   - Manifest entry caps (10,000 entries max) and per-entry link caps (1,000 links max) prevent memory exhaustion attacks during telemetry aggregation.
3. **Structured Error Envelopes**:
   - CLI and MCP endpoints return explicit JSON error structures (`{"ok": false, "subcommand": "doc check", "error": "..."}`) upon read failures or invalid config paths rather than crashing.
4. **Honest Audit Row Emission**:
   - Every execution path, whether successful or failed, writes an honest audit row to the SQLite WAL ring.

## 3. Verification
- `cargo test -p aiosh-core test_collect_doc_index_telemetry` -> 3/3 passed.
- `python tools/test_doc_index_suites.py` -> PASS (D1..D7).
