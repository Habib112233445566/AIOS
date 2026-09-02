# T-00581 — Evidence & Audit Trail / observability: Research

## 1. Goal
Establish facts, constraints, metrics schemas, operational diagnostics, and prior art for the observability of Evidence & Audit Trail in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical Codebase Context):
1. **Audit Ring Event Sourcing**: Every Evidence & Audit Trail invocation (`aiosh evidence hash`, `aiosh evidence verify`, `aiosh evidence scan`, `aios.evidence.hash`, `aios.evidence.verify`, `aios.evidence.scan`) writes structured, hash-chained audit records to the SQLite WAL database.
2. **Verification Diagnostics**: `EvidenceVerificationReport` exposes granular diagnostic data including `total_records`, `valid_records`, `missing_files`, `hash_mismatches`, and `is_valid`.
3. **Manifest Metadata**: `TaskEvidenceManifest` records epic name, task range, ISO 8601 generation timestamp, and an array of individual `EvidenceRecord` entries.
4. **Structured JSON Output**: All CLI commands (`--json`) and MCP JSON-RPC tool calls return standardized structured envelopes with status codes, payload data, and error details.

### Assumptions:
1. Operators and automated orchestrators benefit from a consolidated `EvidenceTelemetry` summary reporting aggregate artifact metrics (`total_records`, `valid_records`, `missing_files_count`, `hash_mismatches_count`, `is_healthy`).
2. Diagnostic verification details should be easily inspectable through both CLI reports and `aiosh audit tail` query logs without exceeding WAL record limits.

## 3. Prior Art & Authoritative Sources
- **ADR-0035 §F-2 (Audit Invariants & Observability)**: Systems rely on immutable append-only event logs for all operational telemetry.
- **OpenTelemetry Semantic Conventions for Artifact & CI/CD Integrity**: Metric definitions and attributes for tracking provenance attestations (`artifact.attestation.filename`), verification durations, and verification outcomes (`cicd.artifact.verification.status`).
- **Twelve-Factor App §XI (Logs as Event Streams)**: Emitting structured telemetry events to stdout/stderr and audit rings.

## 4. Decisions Needed
1. **Telemetry Data Model**: Define `EvidenceTelemetry` struct in `aiosh-core::evidence_service` capturing aggregate verification statistics.
2. **Audit Detail Clamping**: Ensure diagnostic strings emitted to audit logs clamp long missing/mismatch lists to 512 bytes (`clamp_str`) to avoid audit database bloat.
3. **CLI/MCP Observability Surface**: Verify `aiosh evidence verify` and `aios.evidence.verify` return the telemetry summary alongside detailed failure items.

## 5. Next Steps
Advance to Specification (**T-00582**) to define the `EvidenceTelemetry` schema and observability contracts.
