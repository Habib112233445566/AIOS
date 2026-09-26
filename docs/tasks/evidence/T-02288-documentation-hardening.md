# T-02288: Documentation Hardening (Grant Lifecycle)

## Overview
This task documents and verifies the defensive hardening controls implemented for the PEP Grant Lifecycle documentation subsystem (`PepGrantDocIndex`, `pep_grant_doc.rs`, and MCP tool `aios.pep.grant.doc`).

## Hardening Controls Implemented

### 1. Bounded Search Query Constraints
- **Constant**: `MAX_GRANT_DOC_QUERY_LEN = 128`
- **Validation**: Any incoming query string with character length > 128 or empty/whitespace-only is rejected immediately prior to indexing or string operations.
- **Error Code**: `GRANTDOC_ERR_QUERY_BOUNDS`
- **Result**: Guarantees bounded computation time and eliminates potential denial-of-service from abnormally large search payloads.

### 2. Search Result Truncation and Result Ceilings
- **Constant**: `MAX_GRANT_DOC_SEARCH_RESULTS = 10`
- **Constant**: `MAX_GRANT_DOC_SNIPPET_LEN = 200`
- **Behavior**:
  - The scoring algorithm sorts matched topics by relevance score descending.
  - Results are truncated to a maximum of 10 items via `.take(MAX_GRANT_DOC_SEARCH_RESULTS)`.
  - Snippets extracted from matching sections are truncated to at most 200 characters, avoiding memory spikes on client consumers.

### 3. Uniform Error Envelopes and Fail-Safe Semantics
- Tool dispatch in `aiosh-mcp/src/main.rs` captures errors as typed standard JSON error objects (`Result<Value, String>`).
- If an unknown action is supplied, the tool returns `unknown doc action '<action>'; valid actions: list, get, search`.
- If an unknown topic ID is requested, the system returns `topic not found: <topic_id>` accompanied by error constant `GRANTDOC_ERR_NOT_FOUND`.
- No silent failures; every failure path emits an honest error message.

### 4. Zero External Dependency & Safe In-Memory Store
- The documentation repository is fully compiled into the core library as immutable canonical data structures.
- No disk I/O, no network calls, and no spawned child processes occur during documentation queries.
- Zero risk of file descriptor exhaustion, thread starvation, or orphan locks.

## Verification
- Unit tests in `code/aiosh-rust/aiosh-core/tests/test_pep_grant_doc.rs` explicitly test:
  - Valid topic retrieval
  - List topics completeness
  - Case-insensitive search
  - Query bounds enforcement (> 128 characters and empty string)
  - Non-existent topic error handling
