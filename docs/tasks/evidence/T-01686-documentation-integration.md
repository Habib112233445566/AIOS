# T-01686: Kernel Module Management Documentation Integration

## Sub-Epic
Kernel Module Management / Documentation (T-01686)

## Objective
Integrate the Kernel Module Management Documentation subsystem into the CLI operator surface (`aiosh mod doc`) and agent MCP surface (`aios.kernel_module.doc`), author the integration smoke test, and verify end-to-end functionality.

## Implementation Details

1. **CLI Surface Integration (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - Added `doc` subcommand handler under `aiosh mod`.
   - Supports:
     - `aiosh mod doc [list] [--json]`: Lists all available topics.
     - `aiosh mod doc get <topic_id> [--json]`: Retrieves full topic content in Markdown or JSON.
     - `aiosh mod doc search <query> [--json]`: Performs scored full-text and tag searching.
     - Direct topic shortcut: `aiosh mod doc <topic_id>`.
   - Structured audit emission via `classify_and_emit`.

2. **MCP Agent Tool Integration (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Registered `aios.kernel_module.doc` in `tool_manifest` with JSON schema.
   - Handled `aios.kernel_module.doc` in `call_tool` dispatched through `dispatch::recorded_call`.
   - Supports `action: "list" | "get" | "search"`.

3. **Integration Smoke Test (`code/aiosh-cli/tests/test_kernel_module_doc_smoke.py`)**:
   - `test_cli_doc_list`: Validates enumeration of $\ge 7$ canonical topics.
   - `test_cli_doc_get`: Validates topic metadata and section retrieval.
   - `test_cli_doc_search`: Validates scored query matching (`cramfs` -> `cis-benchmark-hardening`).
   - `test_cli_doc_markdown_rendering`: Validates Markdown formatting and headers.
   - `test_mcp_doc_tool`: Validates `aios.kernel_module.doc` across list, get, and search actions via stdio JSON-RPC.

## Verification Execution Results
```
PASS: test_cli_doc_list
PASS: test_cli_doc_get
PASS: test_cli_doc_search
PASS: test_cli_doc_markdown_rendering
PASS: test_mcp_doc_tool
ALL DOCUMENTATION INTEGRATION TESTS PASSED.
```
