# Task Evidence: T-02082 (documentation: Specification)

## Overview
- **Task ID**: T-02082
- **Sub-Epic**: Sub-Epic 9: Capability Documentation Subsystem
- **Component**: `aiosh-core::capability_doc`
- **Objective**: Formally specify invariants (`CAPDOC1..CAPDOC6`), data structures, search algorithms, snippet extraction boundaries, and MCP tool interface for capability documentation.

## Formal Invariants (`CAPDOC1..CAPDOC6`)

| Invariant | Name | Specification |
|---|---|---|
| **`CAPDOC1`** | **Canonical Coverage** | The index must contain comprehensive canonical topics covering all capability model components: Overview, Rights & Scopes, Monotonic Attenuation, Constraints & Quotas, Cascade Revocation, Security Policy, Observability, and MCP Tools. |
| **`CAPDOC2`** | **Deterministic Lookup** | Topics are retrieved by unique identifier (`get_topic`) using trimmed, case-insensitive matching in $O(N)$ or $O(1)$ time. |
| **`CAPDOC3`** | **Relevance-Scored Search** | Search queries score topics based on weighted field matches: Title = 10, Tags = 5, Summary = 3, Section Headings/Content = 1. Results are sorted descending by score. |
| **`CAPDOC4`** | **UTF-8 Snippet Safety** | Contextual snippet extraction around matching query terms must strictly align to UTF-8 character boundaries (`char_indices`), completely preventing multi-byte slicing panics (guarding against N-21 class bugs). Max snippet length is 160 characters. |
| **`CAPDOC5`** | **Defensive Bounds** | Query strings are capped at 256 characters (`MAX_DOC_QUERY_LEN`), topic IDs at 64 characters (`MAX_TOPIC_ID_LEN`), and search results at 50 (`MAX_DOC_SEARCH_RESULTS`). Control characters in inputs are rejected. |
| **`CAPDOC6`** | **Serialization Fidelity** | All topic and search result structures derive `Serialize` and `Deserialize` with deterministic snake_case JSON schemas. |

## Data Structures Specification

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDocCategory {
    Architecture,
    Lifecycle,
    Security,
    Observability,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDocSection {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDocTopic {
    pub id: String,
    pub title: String,
    pub category: CapabilityDocCategory,
    pub summary: String,
    pub sections: Vec<CapabilityDocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}

pub struct CapabilityDocIndex {
    topics: Vec<CapabilityDocTopic>,
}
```

## MCP Integration Specification
- **Tool**: `aios.capability.doc`
- **Arguments**:
  - `action`: string enum (`list`, `get`, `search`)
  - `topic_id`: optional string (required for `action: "get"`)
  - `query`: optional string (required for `action: "search"`)
  - `category`: optional string (filter for `action: "list"`)
