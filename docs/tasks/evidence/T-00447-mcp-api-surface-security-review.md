# T-00447 — Documentation Index Control / MCP/API surface: Security Review

## 1. Review Scope
This security review assesses the MCP tool implementations (`aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`) in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Threat Scenarios & Mitigations

### 1. Host Path Traversal via `repo_path` Argument
- **Threat**: An AI model or remote MCP client passes malicious parameters like `repo_path: "../../../"` to inspect forbidden host directories.
- **Mitigation**: `doc_index_service` bounds all reads by `MAX_DOC_BYTES` (16 MiB) and enforces path canonicalization and normalized component containment checks.

### 2. Denial of Service via Malformed JSON-RPC Payloads
- **Threat**: Sending malformed JSON or omitting required parameters (`query`) to trigger panics in the MCP server daemon.
- **Mitigation**: All input JSON schemas enforce structured field extraction and return explicit JSON-RPC error responses rather than unhandled unwraps.

### 3. Audit Accountability
- **Policy**: All MCP calls are channeled through `dispatch::recorded_call`, writing an immutable row to the audit ring with caller provenance, arguments, and outcome status.

## 3. Verdict
No security bypasses or policy gaps remain open.
