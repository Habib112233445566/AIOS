# T-00464 — Documentation Index Control / automated tests: Implementation

## 1. Implementation Scope
This task implements the test criteria checks D1..D7 in `tools/test_doc_index_suites.py` validating the Documentation Index Control subsystem across data model, configuration hierarchy, Markdown title and link extraction, link integrity verification, CLI subcommands, MCP tool execution, and hardening limits.

## 2. Implementation Details
- **D1 (Manifest Data Model)**: Validates `DocIndexManifest` JSON serialization, query filtering, and structure.
- **D2 (Configuration Layer)**: Tests config file loading, validation of fields, and environment override via `$AIOS_DOC_INDEX_CONFIG`.
- **D3 (Title & Link Extraction)**: Verifies H1 title parsing and markdown inline link extraction excluding external schemes.
- **D4 (Link Integrity & Traversal)**: Executes `doc check --json` asserting link validity and traversal prevention.
- **D5 (CLI Surface)**: Tests `aiosh doc show`, `aiosh doc check`, and `aiosh doc search` in both prose and JSON output modes.
- **D6 (MCP Surface)**: Validates JSON-RPC stdio protocol execution for `aios.doc.index.get`, `aios.doc.check`, and `aios.doc.search`.
- **D7 (Hardening Limits)**: Asserts rejection of oversized configuration payloads (>64 KiB) and non-existent configuration files with clean error handling.

## 3. Test Verification
```text
[+] D1 manifest model & query helpers
[+] D2 configuration hierarchy & limits
[+] D3 title parsing & link extraction
[+] D4 link integrity & traversal detection
[+] D5 CLI subcommand execution & json mode
[+] D6 MCP tool execution & protocol schemas
[+] D7 hardening limits & negative error bounds

PASS: doc_index test criteria (D1..D7)
```
