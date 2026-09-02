# T-00421 — Documentation Index Control / core service: Research

## 1. Goal
Establish facts, constraints, parsing rules, and prior art for the core service of Documentation Index Control in `aiosh-core`.

## 2. Facts vs. Assumptions

### Facts (Empirical Repository Context):
1. **Existing Document Topology**:
   - Documents reside primarily under `docs/` (`docs/README.md`, `docs/SPEC-TASK-LEDGER.md`, `docs/tasks/GOALS.md`, `docs/tasks/MASTER_TASK_LEDGER.md`, and evidence records under `docs/tasks/evidence/`).
2. **Link Patterns**:
   - CommonMark markdown links: `[label](relative_path.md)`.
   - In-tree backticked references: `` `docs/tasks/GOALS.md` ``.
3. **Python Checker Invariants**:
   - `tools/check_task_docs.py` verifies C3 (referenced paths in backticks) and C5 (link targets resolve inside checkout).
4. **Native Rust Service Requirements**:
   - The native Rust core service needs functions to:
     - Scan markdown documents for headings and links.
     - Validate physical file existence for all internal links.
     - Generate formatted index representations for CLI/MCP queries.

### Assumptions:
1. Scanning operations should be non-destructive (read-only) and bound file reads to prevent denial-of-service memory spikes on corrupted trees.
2. Relative paths should be canonicalized and checked for root containment (`Path::canonicalize` or normalized component checking).

## 3. Prior Art & Authoritative Specifications
- **CommonMark Spec (RFC 7763 / RFC 3986)**: Standardizes inline and reference link syntax `[text](target)`.
- **Cargo and Rustdoc Search Indexes**: Deterministic JSON-serialized catalog of module paths and document anchors.
- **Diátaxis Hierarchy**: Organization of docs by functional scope (Overview, Architecture, Task Ledger, Security).

## 4. Decisions Needed
1. **File Scanning Boundaries**: Should document scanning enforce a 16MB file read cap per document matching `tools/check_task_docs.py`?
   - *Decision*: Yes, enforce `MAX_DOC_BYTES = 16 * 1024 * 1024` with `Read::take`.
2. **Path Sanitization**: How to prevent path traversal when checking links?
   - *Decision*: Verify that resolved paths are strictly children of `repo_root`.

## 5. Next Steps
Advance to Specification (T-00422) to define the typed function signatures and report schemas for the core service.
