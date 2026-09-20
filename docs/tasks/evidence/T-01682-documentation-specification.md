# T-01682: Kernel Module Management Documentation Specification

## Sub-Epic
Kernel Module Management / Documentation (T-01682)

## Objective
Formally specify the data model, invariants, search semantics, canonical topics, and interface contracts for the Kernel Module Management Documentation subsystem.

## Formal Specification

### 1. Data Structures & Types
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocCategory {
    Directive,
    Lifecycle,
    Security,
    Observability,
    Baseline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocSection {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocTopic {
    pub id: String,
    pub title: String,
    pub category: DocCategory,
    pub summary: String,
    pub sections: Vec<DocSection>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
    pub matched_tags: Vec<String>,
}

pub struct KernelModuleDocIndex {
    pub topics: Vec<DocTopic>,
}
```

### 2. Invariant Contracts (KD1..KD6)

| Invariant | Name | Guarantee & Acceptance Criterion |
|---|---|---|
| **KD1** | Canonical Registry Breadth | Pre-populates $\ge 7$ distinct canonical topics covering directives, CIS hardening, lifecycles, observability, security policy, container isolation, and wireless pentest drivers. |
| **KD2** | Case-Insensitive ID Lookup | `get_topic` matches exact ID case-insensitively and returns `None` safely on missing IDs. |
| **KD3** | Deterministic Search Scoring | Matches scored by: Exact ID match (+100), Tag match (+50), Title match (+25), Content snippet match (+10). Sorted by score descending. |
| **KD4** | Markdown Rendering Quality | `format_topic_markdown` produces structured GitHub-compatible Markdown with headers, bullet points, code blocks, and citation sections. |
| **KD5** | Deterministic JSON Serialization | `DocTopic`, `DocSearchResult`, and topic lists serialize losslessly to JSON. |
| **KD6** | Operational Interface Equivalence | CLI (`aiosh mod doc list/get/search`) and MCP (`aios.kernel_module.doc`) produce identical documentation payloads. |

### 3. Canonical Built-In Topics
1. `modprobe-directives`: Syntax and semantics of `blacklist`, `alias`, `options`, `install`, `remove`, `softdep`.
2. `cis-benchmark-hardening`: Disabling obsolete filesystems (`cramfs`, `freevxfs`, etc.) and protocols (`dccp`, `sctp`, etc.).
3. `lifecycle-workflows`: Module load, unload, dependency resolution, refcounts, and state transitions.
4. `observability-and-procfs`: Introspecting `/proc/modules`, `/sys/module/*`, memory footprints, and state aggregation.
5. `security-policy-and-pep`: SP-KM1..SP-KM6 policy constraints, PEP capabilities, prohibited and protected module rules.
6. `container-isolation`: Namespace virtualization, overlayfs parameters, network bridging, and isolation presets.
7. `wireless-pentest`: Driver configurations for hardware penetration testing, monitor mode, and packet injection.

### 4. CLI & MCP Surface Interface
- **CLI**:
  - `aiosh mod doc list [--json]`
  - `aiosh mod doc get <topic_id> [--json]`
  - `aiosh mod doc search <query> [--json]`
- **MCP Tool**: `aios.kernel_module.doc`:
  - Arguments:
    - `action`: `"list"` | `"get"` | `"search"` (default: `"list"`)
    - `topic`: `Option<String>` (required if action is `"get"`)
    - `query`: `Option<String>` (required if action is `"search"`)
