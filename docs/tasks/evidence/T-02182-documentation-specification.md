# Task Evidence: T-02182 - PEP Decision Engine: Documentation: Specification

## Task Metadata
- **Task ID**: `T-02182`
- **Sub-Epic**: Sub-Epic 9: Documentation Subsystem
- **Component**: `aiosh-core::pep_doc`
- **Date**: 2026-09-22
- **Status**: Completed

## 1. Specification Overview
Defines the functional, security, and interface specifications for the offline, self-contained documentation and help repository (`aiosh_core::pep_doc`) covering the PEP Decision Engine.

## 2. Invariant Specifications (`PEPDOC1..PEPDOC6`)

### `PEPDOC1`: Documentation Taxonomy & Categorization
- Enum `PepDocCategory` defines 6 standard categories:
  - `Architecture`: Structural model, PEP/PDP/PAP separation, memory layout.
  - `Evaluation`: Target matching, combine algorithms, default-deny evaluation semantics.
  - `Policy`: Administrative authoring boundaries, restricted resources, temporal windows.
  - `Observability`: Metrics aggregation, capacity bounds, health degradation.
  - `Security`: Fail-closed invariants, path traversal rejection, audit ring integrity.
  - `Reference`: Copy-pasteable CLI commands, MCP tool call examples, schema definitions.

### `PEPDOC2`: Structured Topic Schema
- Data models:
  - `PepDocSection`: `title: String`, `content: String`.
  - `PepDocTopic`:
    - `id: String` (kebab-case identifier, max 64 chars)
    - `title: String`
    - `category: PepDocCategory`
    - `summary: String`
    - `sections: Vec<PepDocSection>`
    - `tags: Vec<String>`
    - `references: Vec<String>`
    - `examples: Vec<String>`
  - `PepDocSearchResult`:
    - `topic_id: String`
    - `title: String`
    - `score: usize`
    - `snippet: String`
    - `matched_tags: Vec<String>`

### `PEPDOC3`: Embedded Documentation Registry
- `PepDocIndex::new()` builds a zero-allocation, pre-seeded catalog with 6 canonical topics:
  1. `pep-arch`: Architecture, components, evaluation flow.
  2. `pep-algorithms`: Combining algorithms (`DenyOverrides`, `PermitOverrides`, `FirstApplicable`).
  3. `pep-obligations`: Post-decision obligations (`AuditLog`, `RateLimit`, `RedactFields`, `Custom`).
  4. `pep-secpolicy`: Security governance, administrative boundaries, restricted resources.
  5. `pep-observability`: Observability report, capacity metrics, health thresholds.
  6. `pep-cli-mcp`: CLI commands and MCP JSON-RPC tool calling conventions.

### `PEPDOC4`: Scored Deterministic Search & Snippet Safety
- Function `extract_utf8_snippet(content: &str, byte_idx: usize, query_char_len: usize) -> String`:
  - Never slices across UTF-8 code point boundaries.
  - Truncates context window to `MAX_SNIPPET_LEN = 160` characters.
- Method `PepDocIndex::search(&self, query: &str) -> Vec<PepDocSearchResult>`:
  - Query sanitized: stripped of control characters, length capped to `MAX_DOC_QUERY_LEN = 256`.
  - Blank queries return empty result list immediately.
  - Scoring: Title match = +10, Tag match = +5, Section content occurrence = +1.
  - Deterministic sort: descending by score, then lexicographically by `topic_id`.
  - Result count capped at `MAX_DOC_SEARCH_RESULTS = 50`.

### `PEPDOC5`: Navigation & Filtering API
- `get_topic(&self, id: &str) -> Option<&PepDocTopic>`
- `list_topics(&self) -> Vec<&PepDocTopic>`
- `list_by_category(&self, cat: PepDocCategory) -> Vec<&PepDocTopic>`

### `PEPDOC6`: Production Surface Contracts & Audit Transparency
- **CLI Syntax**:
  - `aiosh pep doc list [--json]`
  - `aiosh pep doc show <id> [--json]`
  - `aiosh pep doc search <query> [--json]`
- **MCP Tool**:
  - Name: `aios.pep.doc`
  - Input Schema: `{ "action": "list" | "get" | "search", "topic_id": string, "query": string, "category": string }`
- **Audit Emission**:
  - All invocations emit audit records through `classify_and_emit` or `dispatch::recorded_call`.
