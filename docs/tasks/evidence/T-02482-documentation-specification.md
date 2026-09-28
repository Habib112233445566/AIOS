# Task T-02482 Evidence: Sandbox Documentation Specification

## Goal
Specify the exact contract and API data structures for the Sandbox Enforcement Documentation Subsystem.

## 1. Domain Entities & Schemas

### `SandboxDocTopic`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SandboxDocTopic {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub keywords: Vec<String>,
    pub content: String,
}
```

### `SandboxDocSummary`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SandboxDocSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub keywords: Vec<String>,
}
```

### `SandboxDocIndex`
```rust
pub struct SandboxDocIndex {
    topics: Vec<SandboxDocTopic>,
}

impl SandboxDocIndex {
    pub fn new() -> Self;
    pub fn list_topics(&self) -> Vec<SandboxDocSummary>;
    pub fn get_topic(&self, id: &str) -> Option<&SandboxDocTopic>;
    pub fn search(&self, query: &str) -> Vec<&SandboxDocTopic>;
}
```

## 2. Invariants & Error Handling
1. **SANDBOXDOC1 (Standard Topic Coverage)**: The default index MUST contain at least 6 canonical topics: `overview`, `profiles`, `policy`, `observability`, `isolation`, and `reference`.
2. **SANDBOXDOC2 (Case-Insensitive Resolution)**: Topic lookups by ID match case-insensitively and ignore leading/trailing whitespace.
3. **SANDBOXDOC3 (Deterministic Lexical Search)**: Search queries score matches across `id`, `title`, `keywords`, and `content`, returning results ordered by relevance.
4. **SANDBOXDOC4 (Zero Side-Effects)**: Documentation retrieval is strictly read-only and emits no side-effects or mutations.
5. **Standard Error Codes**:
   - `SANDBOXDOC_ERR_NOT_FOUND`: Specified topic ID does not exist.
   - `SANDBOXDOC_ERR_EMPTY_QUERY`: Search query is empty or solely whitespace.

## 3. Interfaces & Surfaces
- CLI: `aiosh sandbox doc [topic_id] [--json]`
- MCP: `aios.sandbox.doc` with optional argument `{"topic": "<id-or-query>"}`
