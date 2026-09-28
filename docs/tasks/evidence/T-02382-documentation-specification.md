# Task Evidence: T-02382 - Audit Chain Extensions: Documentation Specification

## Goal
Specify the contract, data structures, and query protocols for the Audit Chain Extensions documentation subsystem (`audit_chain_doc.rs`).

## 1. Domain Types and Data Contracts

### 1.1 Categories (`AuditChainDocCategory`)
- `architecture`: Hash ring chaining, SQLite table schema, and event persistence.
- `lineage`: Causal graph topology, parent-child links, diamond DAG resolution, and cycle prevention.
- `signatures`: Ed25519 cryptographic signing, verification, and tamper detection.
- `policy`: Ingestion enforcement modes (`enforcing`, `permissive`, `disabled`), actor bounds, and rules.
- `observability`: Telemetry aggregation, cardinality reporting, and sanitization.
- `recovery`: Chain validation, corruption isolation, and forward repair.
- `reference`: CLI command syntax and MCP JSON-RPC tool contracts.

### 1.2 Topic Specification (`AuditChainDocTopic`)
```rust
pub struct AuditChainDocTopic {
    pub id: String,
    pub title: String,
    pub category: AuditChainDocCategory,
    pub summary: String,
    pub sections: Vec<AuditChainDocSection>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}
```

### 1.3 Search Result (`AuditChainDocSearchResult`)
```rust
pub struct AuditChainDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
}
```

## 2. Invariants & Bounds
- Query length strictly limited to `MAX_AUDIT_DOC_QUERY_LEN = 128` UTF-8 characters. Empty queries or oversized queries return `AUDITDOC_ERR_QUERY_BOUNDS`.
- Maximum search results returned clamped to `MAX_AUDIT_DOC_SEARCH_RESULTS = 10`.
- Snippets clamped to `MAX_AUDIT_DOC_SNIPPET_LEN = 200` characters.
- Unknown topic lookups return `AUDITDOC_ERR_NOT_FOUND`.

## 3. Surface Specifications
### 3.1 CLI Contract
- Command: `aiosh audit doc [query-or-topic] [--json]`
- Behavior:
  - If no argument: lists all topics with summaries.
  - If argument matches a topic ID: prints full topic sections and examples.
  - Otherwise: executes search and outputs ranked matching snippets.

### 3.2 MCP Tool Contract (`aios.audit.doc`)
- Parameters:
  - `action`: `"list" | "get" | "search"` (optional, inferred from arguments)
  - `topic_id`: `String` (optional)
  - `query`: `String` (optional)
- Standard envelope response: `{"ok": true, "tool": "aios.audit.doc", "data": ...}`.
