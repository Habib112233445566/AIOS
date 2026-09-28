# Task Evidence: T-02386 - Audit Chain Extensions: Documentation Integration

## Goal
Integrate the documentation subsystem of Audit Chain Extensions into the production CLI and MCP server surfaces.

## Integration Details
1. **CLI Command (`aiosh audit doc`)**:
   - Added subcommand `aiosh audit doc [query-or-topic] [--json]` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Wired `cmd_audit_doc` to inspect the canonical index:
     - When called without arguments: lists all topics with id, category, title, and summary.
     - When called with a topic ID: renders complete markdown or structured JSON.
     - When called with search keywords: performs relevance-ranked search and prints matches with snippets.
2. **MCP Tool (`aios.audit.doc`)**:
   - Registered `aios.audit.doc` in the MCP tool catalog in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Handled `list`, `get`, and `search` actions under audited PEP dispatch (`dispatch::recorded_call`), guaranteeing complete non-repudiation on read.
3. **Workspace Check**:
   - Full workspace verification: `cargo check --workspace` succeeded with 0 errors and 0 warnings.
