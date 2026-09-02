# T-00411 — Documentation Index Control / data model: Research

## 1. Goal
Establish facts, constraints, structural requirements, and prior art for the data model of Documentation Index Control in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical Repository Context):
1. **Existing Documentation Artifacts**:
   - `docs/README.md`: Central operator and developer manual with sub-sections for every epic.
   - `docs/tasks/MASTER_TASK_LEDGER.md` & `docs/tasks/MASTER_TASK_LEDGER.jsonl`: Complete task index.
   - `docs/SPEC-TASK-LEDGER.md`: Invariant specification.
   - `docs/tasks/GOALS.md`: Mission and sequential execution laws.
2. **Existing Validation Tools**:
   - `tools/check_task_docs.py` enforces Python-level invariants (C1..C6) including spec health, component sections, referenced paths, phase maps, and link resolution.
3. **Native Rust Integration Gap**:
   - While Python tools enforce static repo invariants, the native Rust runtime (`aiosh-core`), CLI (`aiosh`), and MCP agent server currently lack a structured native data model for querying, inspecting, validating, and searching documentation indexes programmatically.

### Assumptions:
1. Documentation Index Control requires a deterministic, serializable data model representing document nodes, section hierarchies, task ranges, and link edges.
2. The data model should support pure in-memory deserialization, validation, and JSON-RPC transport for agent workflows.

## 3. Prior Art & Industry Standards
- **Diátaxis Documentation Framework**: Standardizes technical documentation categorization into Tutorials, How-To Guides, Reference, and Explanation.
- **Markdown AST / Link Resolvers (CommonMark / pulldown-cmark)**: Models documents as hierarchical trees with URI link references that resolve relative to repository roots.
- **Structured Knowledge Graph Models**: Index manifests storing document paths, headings, checksums, and cross-references.

## 4. Proposed Data Model
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexEntry {
    pub path: String,
    pub title: String,
    pub section: String,
    pub task_range: Option<String>,
    pub backlinked_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexManifest {
    pub version: String,
    pub entries: Vec<DocIndexEntry>,
}
```

## 5. Decisions Needed
1. **Data Model Representation**: Should `DocIndexManifest` be defined in `code/aiosh-rust/aiosh-core/src/doc_index.rs`?
   - *Decision*: Yes, following the modular pattern of `release.rs` and `toolchain_config.rs`.
2. **Serialization Format**: Should the manifest serialize to canonical JSON for MCP/CLI parity?
   - *Decision*: Yes, using `serde` and standard JSON representations.

## 6. Next Steps
Advance to Specification (T-00412) to formalize the schema, struct fields, and serialization contracts.
