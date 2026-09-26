# T-02282 Specification: Grant Lifecycle Documentation

**Task:** Specify the exact contract for the documentation of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Documentation  

---

## 1. Scope & Objectives

The Grant Lifecycle Documentation Subsystem (`PepGrantDocIndex`) provides an embedded, self-contained reference repository and search index for PEP authorization grants, rights attenuation, revocation cascading, security policies, and MCP interfaces.

---

## 2. Data Types & Interface Specification

### 2.1 Categories
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepGrantDocCategory {
    Architecture,
    Lifecycle,
    Attenuation,
    Revocation,
    Policy,
    Observability,
    Reference,
}
```

### 2.2 Topic and Section Structures
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantDocSection {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantDocTopic {
    pub id: String,
    pub title: String,
    pub category: PepGrantDocCategory,
    pub summary: String,
    pub sections: Vec<PepGrantDocSection>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
}
```

### 2.3 Constants & Error Codes
| Identifier | Type | Value | Purpose |
|---|---|---|---|
| `MAX_GRANT_DOC_QUERY_LEN` | `usize` | `128` | Maximum query length |
| `MAX_GRANT_DOC_SEARCH_RESULTS` | `usize` | `10` | Maximum results returned |
| `MAX_GRANT_DOC_SNIPPET_LEN` | `usize` | `200` | Snippet character limit |
| `GRANTDOC_ERR_NOT_FOUND` | `&str` | `"GRANTDOC_ERR_NOT_FOUND"` | Topic ID not found |
| `GRANTDOC_ERR_QUERY_BOUNDS` | `&str` | `"GRANTDOC_ERR_QUERY_BOUNDS"` | Query empty or > 128 chars |

---

## 3. Evaluation Contract & Search Logic

```rust
impl PepGrantDocIndex {
    /// Constructs default index populated with canonical grant topics.
    pub fn new() -> Self;

    /// Lists all available documentation topics.
    pub fn list_topics(&self) -> &[PepGrantDocTopic];

    /// Retrieves a specific topic by its unique ID.
    pub fn get_topic(&self, id: &str) -> Option<&PepGrantDocTopic>;

    /// Searches topics with relevance scoring.
    pub fn search(&self, query: &str) -> Result<Vec<PepGrantDocSearchResult>, String>;

    /// Formats a topic into GitHub-flavored Markdown.
    pub fn render_markdown(&self, topic_id: &str) -> Result<String, String>;
}
```

### 3.1 Scoring Algorithm
- **Exact Tag Match:** +10 points
- **Title Match:** +5 points
- **Summary Match:** +3 points
- **Section Match:** +1 point per hit
- Results sorted in descending score order and truncated to `MAX_GRANT_DOC_SEARCH_RESULTS`.

---

## 4. Canonical Topic Catalog
1. `grant-arch`: Data model, capability scope, constraints, and validation rules.
2. `grant-lifecycle`: State machine (Requested, Active, Suspended, Revoked, Expired).
3. `grant-attenuation`: Monotonicity invariants, delegation depth decrement.
4. `grant-revocation`: Direct revocation, cascade tree revocation, and expiration sweeping.
5. `grant-policy`: `PepGrantSecurityPolicy`, duration limits, prohibited delegation rights.
6. `grant-observability`: Telemetry aggregation and health thresholds.
7. `grant-mcp`: Tool definitions, parameters, and invocation examples.

---

## 5. Acceptance Verification
- ✅ Complete contract defined for topics, search, and Markdown rendering.
- ✅ Reuses existing patterns from `pep_doc.rs`.
- ✅ Standalone reviewable specification.
