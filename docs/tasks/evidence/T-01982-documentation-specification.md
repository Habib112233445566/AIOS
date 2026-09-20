# Task Evidence: T-01982 - System Update / documentation: Specification (Sub-Epic 9)

## 1. Overview
- **Task ID**: `T-01982`
- **Sub-Epic**: 9 (System Update Documentation Subsystem)
- **Goal**: Specify the data models, query interfaces, dynamic rendering templates, and persistence contracts for `system_update_doc.rs`.

---

## 2. Data Models & Interface Contracts

### 2.1 `SystemUpdateDocCategory`
Enumeration of documentation categories:
- `Architecture`: Core system update subsystem concepts, design rationale, and invariants.
- `ABPartitioning`: Dual-slot partition layout, bootloader interaction, slot selection, and switching.
- `Security`: Cryptographic signature verification, anti-rollback protection, partition allowlisting, and revocation denylists.
- `Observability`: Telemetry reporting, dual-slot state monitoring, sanitization, and health checks.
- `Configuration`: `SystemUpdateConfig` options, environment overrides, and quotas.
- `Troubleshooting`: Diagnostics, rollback recovery, digest error resolution, and manual slot rescue.

Methods:
- `as_str(&self) -> &'static str`
- `from_str_loose(s: &str) -> Option<Self>`

### 2.2 `SystemUpdateDocTopic`
Structured documentation entity:
```rust
pub struct SystemUpdateDocTopic {
    pub id: String,
    pub title: String,
    pub category: SystemUpdateDocCategory,
    pub tags: Vec<String>,
    pub content: String,
    pub see_also: Vec<String>,
}
```
Methods:
- `to_markdown(&self) -> String`: Formats topic into Markdown with metadata table, content body, and cross-references.

### 2.3 `SystemUpdateDocSearchResult`
Search result with relevance scoring:
```rust
pub struct SystemUpdateDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub category: SystemUpdateDocCategory,
    pub score: u32,
    pub snippet: String,
}
```

### 2.4 `SystemUpdateDocIndex`
In-memory documentation repository and rendering engine:
- `new() -> Self`: Initializes repository pre-populated with canonical technical topics.
- `get_by_id(&self, id: &str) -> Option<&SystemUpdateDocTopic>`: Exact topic retrieval by identifier.
- `list_by_category(&self, category: SystemUpdateDocCategory) -> Vec<&SystemUpdateDocTopic>`: Categorical filtering.
- `search(&self, query: &str) -> Vec<SystemUpdateDocSearchResult>`: Tokenized, ranked full-text search.
- `render_full_index_markdown(&self) -> String`: Formats complete documentation catalog into Markdown.
- `render_status_markdown(service: &SystemUpdateService) -> String`: Formats live slot and update status into Markdown with ASCII slot diagram.
- `render_observability_markdown(report: &SystemUpdateObservabilityReport) -> String`: Formats live observability telemetry report into a Markdown executive summary.
- `export_to_file(&self, path: &Path) -> Result<(), String>`: Atomic file export with symlink rejection and 1 MB size limit.

---

## 3. Operational Invariants (`UDOC1..UDOC6`)
- `UDOC1` (Canonical Offline Index): Pre-populated repository covering all 6 functional domains.
- `UDOC2` (Deterministic Category Navigation): Standard categories with loose case-insensitive parsing.
- `UDOC3` (Ranked Full-Text Search): Relevance scoring: ID match (+100), title match (+50), tag matches (+20), body matches (+5).
- `UDOC4` (Markdown Export): Formats topics and catalogs into GitHub Flavored Markdown.
- `UDOC5` (Dynamic Status & Diagram Rendering): Dynamic Markdown generation from live service and report structs.
- `UDOC6` (Bounded I/O & Path Hygiene): Max 1 MB export size, symlink defense, atomic `.tmp.<pid>` rename.
