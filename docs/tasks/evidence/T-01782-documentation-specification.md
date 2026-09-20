# Specification: Hardware Detection Documentation Subsystem (T-01782)

## 1. Executive Summary
- **Subsystem**: Hardware Detection Documentation Subsystem (`aiosh-core::hardware_doc`)
- **Task**: `T-01782`
- **Scope**: Formal contract, invariants `HDOC1..HDOC6`, topic schema, search scoring, and markdown formatting.
- **Status**: **PASS (Specification Complete)**

---

## 2. Documentation Contract & Data Schema

### 2.1 Types & Enums
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareDocCategory {
    Architecture,
    Discovery,
    Security,
    Observability,
    Configuration,
    Troubleshooting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareDocSection {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareDocTopic {
    pub id: String,
    pub title: String,
    pub category: HardwareDocCategory,
    pub summary: String,
    pub sections: Vec<HardwareDocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}

pub const MAX_DOC_QUERY_LEN: usize = 256;
pub const MAX_DOC_SEARCH_RESULTS: usize = 50;
pub const MAX_TOPIC_ID_LEN: usize = 64;

pub struct HardwareDocIndex {
    pub topics: Vec<HardwareDocTopic>,
}
```

### 2.2 Methods
```rust
impl HardwareDocIndex {
    pub fn new() -> Self;
    pub fn get_topic(&self, id: &str) -> Option<&HardwareDocTopic>;
    pub fn search(&self, query: &str, category: Option<HardwareDocCategory>) -> Vec<HardwareDocSearchResult>;
    pub fn list_topics(&self, category: Option<HardwareDocCategory>) -> Vec<&HardwareDocTopic>;
    pub fn format_topic_markdown(topic: &HardwareDocTopic) -> String;
}
```

---

## 3. Documentation Invariants (HDOC1..HDOC6)

- **HDOC1 (Offline Completeness)**: The documentation index is completely self-contained in compiled memory with zero network, filesystem, or external database dependencies.
- **HDOC2 (Case-Insensitive Topic Lookup)**: `get_topic(id)` performs case-insensitive ASCII comparison. Rejects inputs $> 64$ characters or containing control characters.
- **HDOC3 (Ranked Search Scoring)**:
  - Exact ID match: $+100$ points.
  - Title match: $+50$ points.
  - Tag match: $+30$ points per tag.
  - Content / Summary match: $+10$ points.
  - Search queries capped at `MAX_DOC_QUERY_LEN = 256` chars; results capped at `MAX_DOC_SEARCH_RESULTS = 50`.
- **HDOC4 (Category Filtering)**: Both `search` and `list_topics` support optional category filtering via `HardwareDocCategory`.
- **HDOC5 (Deterministic Markdown Rendering)**: `format_topic_markdown` generates deterministic markdown with consistent section headers, code blocks, and references.
- **HDOC6 (Bounded Resource Footprint)**: Total in-memory index size $\le 500$ KB; search execution latency $\le 5$ ms.
