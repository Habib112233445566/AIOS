# Security Review: PEP Decision Engine Documentation Subsystem (T-02187)

## 1. Threat Model & Analysis Scope
This security review assesses the attack surface of the PEP Decision Engine documentation subsystem across `aiosh-core/src/pep_doc.rs`, `aiosh-cli/src/main.rs` (`cmd_pep doc`), and `aiosh-mcp/src/main.rs` (`aios.pep.doc`).

## 2. Abuse Scenarios & Mitigations

### Abuse Scenario 1: Path Traversal via Topic Identifiers
- **Vector**: An attacker or untrusted agent submits a `topic_id` containing path traversal sequences (e.g., `../../../../etc/passwd` or `..\\..\\Windows\\win.ini`) to read arbitrary host filesystem contents.
- **Analysis**: The documentation subsystem does not perform filesystem operations for topic resolution. All topics are managed via an in-memory index (`PepDocIndex`) populated with canonical static topics. Lookup is an in-memory equality test (`topic.id.eq_ignore_ascii_case(id)`).
- **Result**: **MITIGATED**. No file I/O occurs during topic retrieval; path traversal cannot reach or expose host files.

### Abuse Scenario 2: Denial of Service via Massive Query or Regex Bombs
- **Vector**: An attacker passes a multi-megabyte string or malicious regex/wildcard pattern into `search` to cause CPU exhaustion or OOM crash.
- **Analysis**: `PepDocIndex::search` enforces `MAX_DOC_QUERY_LEN = 256`. Queries exceeding this length or containing control characters are rejected immediately. Furthermore, the search algorithm uses simple, linear substring matching (`contains()`), not regular expressions, completely eliminating catastrophic backtracking (ReDoS). Results are capped at `MAX_DOC_SEARCH_RESULTS = 50`.
- **Result**: **MITIGATED**. O(N * M) bounded complexity where N is the fixed number of topics and M is the capped query length.

### Abuse Scenario 3: UTF-8 Slice Panic Exploitation
- **Vector**: A search query matches near multi-byte UTF-8 sequences (e.g., emojis, CJK, non-ASCII punctuation). If naive byte slicing `&text[start..end]` is performed, Rust panics with a thread panic or abort.
- **Analysis**: `extract_utf8_snippet` safely calculates start and end offsets using UTF-8 character boundaries (`char_indices()` and slicing validation). Tests in `test_pep_doc_utf8_snippet_safety` explicitly verify multi-byte characters and boundary conditions without panic.
- **Result**: **MITIGATED**. Zero panics on arbitrary UTF-8 input.

### Abuse Scenario 4: Unauthorized Policy Alteration via Documentation Interface
- **Vector**: An unprivileged agent attempts to modify policy rules or bypass PDP authorization by masquerading a rule change as a documentation request.
- **Analysis**: `PepDocIndex` exposes strictly read-only query operations (`list_topics`, `get_topic`, `search`). It contains no mutable policy storage APIs and does not interact with `PepStore` write paths.
- **Result**: **MITIGATED**. Interface separation ensures zero write capability into authorization policy tables.

### Abuse Scenario 5: Audit Bypass or Tampering
- **Vector**: An operator or agent issues queries to the documentation subsystem without triggering audit trail logging.
- **Analysis**: All MCP tool invocations route through `dispatch::recorded_call`, writing an immutable event record into the SQLite ring buffer. All CLI commands emit classification events via `classify_and_emit`.
- **Result**: **MITIGATED**. Full audit accountability maintained across both surfaces.

## 3. Findings Summary
- **Critical Issues**: 0
- **High Issues**: 0
- **Medium Issues**: 0
- **Low Issues**: 0
- **Status**: APPROVED for production integration.
