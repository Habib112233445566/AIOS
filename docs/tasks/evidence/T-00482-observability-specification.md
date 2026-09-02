# T-00482 — Documentation Index Control / observability: Specification

## 1. Specification Overview
This specification formalizes the telemetry schemas, audit logging events, diagnostic reporters, and error observability for Documentation Index Control in AIOS.

## 2. Telemetry and Event Schemas

### A. Telemetry Data Model (`DocIndexTelemetry`)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexTelemetry {
    pub total_docs_indexed: usize,
    pub total_links_checked: usize,
    pub broken_links_count: usize,
    pub is_healthy: bool,
}
```

### B. Audit Ring Event Schema (Happy Path)
- **Tool**: `doc.show` | `doc.check` | `doc.search` | `aios.doc.index.get` | `aios.doc.check` | `aios.doc.search`.
- **Outcome**: `"ok"` / `"success"`.
- **Outcome Detail**: 512-byte clamped telemetry summary:
  - `doc.show`: `"Indexed 3 documentation files, 0 errors"`.
  - `doc.check`: `"Checked 45 documentation links, 0 broken links, status=healthy"`.
  - `doc.search`: `"Search query 'task': matched 4 entries"`.

### C. Audit Ring Event Schema (Failure Path)
- **Tool**: `doc.show` | `doc.check` | `doc.search` | `aios.doc.*` | `aios.doc.set`.
- **Outcome**: `"error"` | `"refused"`.
- **Outcome Detail**:
  - Missing file / bad path: `"Document not found at <path>"`.
  - Link verification failure: `"Link validation failed: <count> broken links detected"`.
  - Oversized config: `"Config exceeds maximum allowed size of 64 KiB"`.
  - Policy refusal: `"Security policy violation: 'aios.doc.set' is an irreversible mutating action and requires an active PEP grant"`.

## 3. Reused vs. New Interfaces
- **Reused**:
  - `aiosh-core::audit::AuditRing` and `aiosh-cli::emit` for WAL persistence.
  - `aiosh-core::doc_index::{DocIndexManifest, DocIndexEntry}`.
  - `aiosh-core::doc_index_service::{DocLinkValidationReport, BrokenDocLink}`.
- **New (AIOS-Specific)**:
  - `DocIndexTelemetry` struct.
  - `collect_doc_index_telemetry(&DocIndexManifest, Option<&DocLinkValidationReport>) -> DocIndexTelemetry`.
