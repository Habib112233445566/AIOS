# Task Evidence: T-01882 - Network Bootstrap / documentation: Specification

## 1. Overview
- **Task ID**: `T-01882`
- **Sub-Epic**: 9 (Network Bootstrap Documentation Subsystem)
- **Goal**: Formally specify types, interfaces, search scoring, Markdown generation, and persistence for Network Bootstrap Documentation.

---

## 2. Specification

### 2.1 Types & Categories
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkDocCategory {
    Architecture,
    Discovery,
    Security,
    Observability,
    Configuration,
    Troubleshooting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkDocSection {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkDocTopic {
    pub id: String,
    pub title: String,
    pub category: NetworkDocCategory,
    pub summary: String,
    pub sections: Vec<NetworkDocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}
```

### 2.2 Index & Generator Interface: `NetworkDocIndex`
- `new() -> Self`: Initializes repository with 6+ canonical topics.
- `get_topic(&self, id: &str) -> Option<&NetworkDocTopic>`
- `list_topics(&self, category: Option<NetworkDocCategory>) -> Vec<&NetworkDocTopic>`
- `search(&self, query: &str) -> Vec<NetworkDocSearchResult>`: Ranked search scoring by ID (+100), Title (+50), Tag (+25), Content (+5).
- `render_topic_markdown(&self, id: &str) -> Option<String>`: Formats topic as clean Markdown document.
- `render_state_markdown(&self, state: &NetworkState) -> String`: Generates live system network report with interface tables, routing tables, and DNS configuration.
- `render_ascii_topology(&self, state: &NetworkState) -> String`: Renders ASCII representation of host network topology.
- `save_to_path(&self, content: &str, path: &Path) -> Result<(), String>`: Atomic persistence with hygiene and size checks.

### 2.3 Error Codes
- `NDOC_IO_ERROR`: Write or directory creation failure.
- `NDOC_PATH_ERROR`: Path length > 1024, traversal (`..`), or control characters.
- `NDOC_VALIDATION_ERROR`: Content exceeding 1 MB (`MAX_DOC_FILE_BYTES`).
