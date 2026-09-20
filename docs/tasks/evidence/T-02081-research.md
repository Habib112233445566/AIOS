# Research Summary: T-02081 (documentation: Research)

- **Sub-Epic**: Sub-Epic 9: Capability Documentation Subsystem
- **Scope**: In-memory documentation engine for the AIOS Capability Model.
- **Key Decisions**:
  - Model after `kernel_module_doc.rs` for consistency and reliability.
  - 8 canonical topics covering `CAP1..CAP6`, `CAPSEC1..CAPSEC6`, and `CAPOBS1..CAPOBS6`.
  - UTF-8 safe snippet extraction and bounded query constraints.
  - Multi-action MCP tool `aios.capability.doc` supporting `list`, `get`, and `search`.
