# T-00481 — Documentation Index Control / observability: Research

## 1. Goal
Establish facts, constraints, metrics schemas, operational diagnostics, and prior art for the observability of Documentation Index Control in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical Codebase Context):
1. **Audit Ring Event Sourcing**: Every Documentation Index invocation (`aiosh doc show`, `aiosh doc check`, `aiosh doc search`, `aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`) writes structured, hash-chained audit records to the SQLite WAL database.
2. **Link Health Diagnostics**: `DocLinkValidationReport` exposes granular diagnostic data including `total_links_checked`, `is_valid`, and a collection of `BrokenDocLink` items with `source_path`, `target_link`, and `reason`.
3. **Catalog Metadata**: `DocIndexManifest` records document count, document paths, H1 titles, section groupings, outbound link lists, and line counts per indexed file.
4. **Structured JSON Output**: All CLI commands (`--json`) and MCP JSON-RPC tool calls return standardized structured envelopes with status codes, payload data, and error details.

### Assumptions:
1. Operators and automated orchestrators benefit from a consolidated `DocIndexTelemetry` summary reporting aggregate document metrics (`total_docs_indexed`, `total_links_checked`, `broken_links_count`, `is_healthy`).
2. Diagnostic link verification details should be easily inspectable through both CLI reports and `aiosh audit tail` query logs.

## 3. Prior Art & Authoritative Sources
- **ADR-0035 §F-2 (Audit Invariants & Observability)**: Systems rely on immutable append-only event logs for all operational telemetry.
- **OpenTelemetry Semantic Conventions for Content Indexers**: Metric definitions for document catalogs, graph traversal counts, and validation error counts.
- **Twelve-Factor App §XI (Logs as Event Streams)**: Emitting structured telemetry events to stdout/stderr and audit rings.

## 4. Decisions Needed
1. **Telemetry Data Model**: Define `DocIndexTelemetry` struct in `aiosh-core::doc_index_service` capturing aggregate statistics.
2. **Audit Detail Clamping**: Ensure diagnostic strings emitted to audit logs clamp long paths or broken link lists to 512 bytes (`clamp_str`) to avoid audit database bloat.
3. **CLI/MCP Observability Surface**: Verify `aiosh doc check` and `aios.doc.check` return the telemetry summary alongside detailed broken link items.

## 5. Next Steps
Advance to Specification (T-00482) to define the `DocIndexTelemetry` schema and observability contracts.
