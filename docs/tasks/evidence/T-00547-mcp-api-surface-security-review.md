# T-00547 — Evidence & Audit Trail / MCP/API surface: Security Review

## 1. Overview
This security review evaluates the Model Context Protocol (MCP) and JSON-RPC 2.0 API surface for Evidence & Audit Trail (`aios.evidence.verify`, `aios.evidence.hash`, `aios.evidence.scan`).

## 2. Threat Scenarios & Mitigations

### A. JSON-RPC Payload Smuggling & Deserialization Panics
- **Threat**: Malformed JSON types (e.g. non-integer task IDs, deeply nested arrays) causing unhandled runtime crashes or server termination.
- **Evaluation**: Input parameters are parsed with safe combinators (`as_str()`, `as_u64()`, `unwrap_or(...)`) and return structured error envelopes without panics.

### B. Unbounded Directory Traversal via `repo_path`
- **Threat**: Passing relative `../` or system root paths into `repo_path` to enumerate private directory hierarchies.
- **Evaluation**: Directory lookups resolve only the `docs/tasks/evidence` subpath under the repo root, and individual evidence records are checked for valid markdown extensions and task numbering formats.

### C. Resource Exhaustion during Scan / Hash
- **Threat**: Repeated scanning of thousands of files or large payload generation.
- **Evaluation**: `compute_file_sha256` enforces a 16 MiB size cap per file, and directory scanning ignores non-markdown files.

### D. Audit Ring Immutability
- **Threat**: Executing MCP tools without generating audit records.
- **Evaluation**: All tool calls execute through `dispatch::recorded_call`, which records user/agent identity, parameters, tool name, and exit status into the SQLite WAL ring.

## 3. Findings & Verdict
The MCP/API surface enforces robust type handling, boundary validation, size caps, and complete audit trail persistence. No policy bypasses remain open.
