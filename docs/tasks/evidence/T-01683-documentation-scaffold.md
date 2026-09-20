# T-01683: Kernel Module Management Documentation Scaffold

## Sub-Epic
Kernel Module Management / Documentation (T-01683)

## Objective
Scaffold the Kernel Module Management Documentation subsystem under `code/aiosh-rust/aiosh-core/src/kernel_module_doc.rs` and register it within `code/aiosh-rust/aiosh-core/src/lib.rs`.

## Scaffold Summary
1. **Source File Created**:
   - `code/aiosh-rust/aiosh-core/src/kernel_module_doc.rs`
2. **Types & Traits Defined**:
   - `DocCategory`: `Directive`, `Lifecycle`, `Security`, `Observability`, `Baseline`.
   - `DocSection`: Structured documentation section (`title`, `content`).
   - `DocTopic`: Full topic record (`id`, `title`, `category`, `summary`, `sections`, `tags`, `references`, `examples`).
   - `DocSearchResult`: Relevance-scored search match (`topic_id`, `title`, `score`, `snippet`, `matched_tags`).
   - `KernelModuleDocIndex`: In-memory registry with `new`, `get_topic`, `list_topics`, `list_by_category`, `search`, and `format_topic_markdown`.
3. **Crate Registration**:
   - Added `pub mod kernel_module_doc;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
4. **Build Verification**:
   - Successfully verified compilation via `cargo check -p aiosh-core`.
