# Task Evidence: T-02086 (documentation: Integration)

## Overview
- **Task ID**: T-02086
- **Sub-Epic**: Sub-Epic 9: Capability Documentation Subsystem
- **Component**: `aiosh-mcp` (Tool `aios.capability.doc`)
- **Objective**: Integrate the Capability Documentation Subsystem into the MCP server, exposing actions `list`, `get`, and `search`, and validate via cross-surface smoke testing.

## Integration Details
1. **MCP Tool Declaration (`aios.capability.doc`)**:
   - Registered in `aiosh-mcp/src/main.rs`.
   - Input schema supports:
     - `action`: enum `["list", "get", "search"]` (required).
     - `topic_id`: string (required for `get`).
     - `query`: string (required for `search`).
     - `category`: enum `["architecture", "lifecycle", "security", "observability", "reference"]` (optional filter for `list`).
2. **MCP Dispatch Handler**:
   - Implemented in `aiosh-mcp/src/main.rs::call_tool()`.
   - Action `list`: returns array of topic summaries (ID, title, category, summary, tags).
   - Action `get`: returns full topic object and rendered GitHub-Flavored Markdown.
   - Action `search`: returns scored results with UTF-8 safe snippets and matched tags.
   - Audited via `dispatch::recorded_call` to the SQLite WAL audit ring.
3. **Cross-Surface E2E Smoke Suite (`code/aiosh-mcp/tests/test_capability_doc_smoke.py`)**:
   - Tests tool manifest registration (`tools/list`).
   - Tests full listing (8 topics) and category-filtered listing (2 architecture topics).
   - Tests topic retrieval by ID and Markdown formatting.
   - Tests scored search and relevance ranking for query `"attenuation"`.
   - Tests error handling on missing/invalid arguments.

## Verification
- Validated with `test_capability_mcp_tools` in `aiosh-mcp`.
- Validated with `python code/aiosh-mcp/tests/test_capability_doc_smoke.py`.
