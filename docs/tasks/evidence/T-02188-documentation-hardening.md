# Hardening Report: PEP Decision Engine Documentation Subsystem (T-02188)

## 1. Hardening Measures Implemented

### 1.1 Bounded Operational Limits
The documentation subsystem implements compile-time and runtime hard limits to eliminate denial of service and memory growth risks:
- **Maximum Query Length**: `MAX_DOC_QUERY_LEN = 256` chars. Queries exceeding this length are safely rejected before token matching.
- **Maximum Topic ID Length**: `MAX_TOPIC_ID_LEN = 128` chars.
- **Maximum Search Results**: `MAX_DOC_SEARCH_RESULTS = 50` items. Results are sorted by relevance and truncated to prevent unbounded output serialization.
- **Maximum Snippet Length**: `MAX_SNIPPET_LEN = 160` chars. Snippets are extracted respecting UTF-8 character boundaries.

### 1.2 Explicit Result Envelopes & Error Propagation
Both production surfaces guarantee uniform, non-silent error handling:
- **MCP (`aios.pep.doc`)**:
  - Missing parameters: `{ "ok": false, "error": "missing required parameter: <param>" }`
  - Unknown topics: `{ "ok": false, "error": "topic not found: <id>" }`
  - Unknown actions: `{ "ok": false, "error": "unknown action: <action>" }`
- **CLI (`aiosh pep doc`)**:
  - Invalid arguments or unrecognized topic IDs output formatted errors to stderr and return exit code 2.
  - JSON mode (`--json`) emits `{ "ok": false, "error": "..." }` to stdout.

### 1.3 Resource Leak Prevention
- **Zero Temp Files**: No temporary files or cache artifacts are written to disk during doc indexing or querying.
- **Zero Subprocess Spawning**: Query execution is self-contained in-memory within the caller process.
- **Thread Safety**: `PepDocIndex` structures are immutable post-creation and implement `Send + Sync`.

### 1.4 Fail-Open / Fail-Closed Audit Invariant (ADR-0035 §F-2)
- All calls passing through `dispatch::recorded_call` emit audit rows regardless of whether the inner closure returns `Ok` or `Err`.
- Error messages and failure reasons are faithfully recorded in the SQLite audit ring buffer.

## 2. Hardening Verification Matrix
| Invariant | Mechanism | Status |
|-----------|-----------|--------|
| Query size cap | `query.len() <= 256` check | Verified |
| Result set cap | `.take(50)` limit | Verified |
| UTF-8 safety | `extract_utf8_snippet` boundary check | Verified |
| Non-silent error | Structured envelope on error | Verified |
| Audit trail | `dispatch::recorded_call` / SQLite ring | Verified |
| Resource cleanup | 0 temp files, 0 leaks | Verified |
