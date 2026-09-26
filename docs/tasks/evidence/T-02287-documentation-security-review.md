# T-02287: Documentation Security Review (Grant Lifecycle)

## Overview
This security review assesses the security properties, threat vectors, input validation boundaries, and audit guarantees for the PEP Grant Lifecycle documentation subsystem (`PepGrantDocIndex`, `pep_grant_doc.rs`, and the `aios.pep.grant.doc` MCP tool).

## Threat Modeling & Abuse Scenarios

### Abuse Scenario AS-DOC-01: Path Traversal via Arbitrary Topic IDs
- **Vector**: An unprivileged agent or external client calls `aios.pep.grant.doc` with `action: "get"` and `topic_id: "../../../etc/shadow"` or `topic_id: "..\\Windows\\System32\\config\\SAM"`.
- **Analysis**: `PepGrantDocIndex` is an offline, in-memory repository compiled into the static binary with canonical pre-seeded data structures. `get_topic` performs a linear key equality match against internal `topic.id` strings. It does not open files, call `std::fs`, or interact with OS path resolvers.
- **Finding**: **MITIGATED**. Traversal attempts fail with standard `GRANTDOC_ERR_NOT_FOUND` / `topic not found: <id>`.

### Abuse Scenario AS-DOC-02: ReDoS and Algorithmic Complexity Attacks
- **Vector**: Adversary supplies an exponential backtracking regex or million-character string to `query` in `action: "search"`.
- **Analysis**: The search mechanism in `PepGrantDocIndex::search` utilizes linear substring matching (`to_lowercase().contains(&lower_q)`), which has \(O(N \times M)\) worst-case time complexity. Furthermore, `MAX_GRANT_DOC_QUERY_LEN` strictly caps incoming query length at 128 characters. Queries longer than 128 characters or empty whitespace strings return `Err(GRANTDOC_ERR_QUERY_BOUNDS)`.
- **Finding**: **MITIGATED**. CPU consumption is bounded to microsecond execution per search.

### Abuse Scenario AS-DOC-03: Memory Exhaustion via Unbounded Result Payloads
- **Vector**: Adversary crafts a query matching thousands of substrings, causing massive allocation and client OOM.
- **Analysis**: `PepGrantDocIndex::search` caps the returned topic matches to `MAX_GRANT_DOC_SEARCH_RESULTS = 10`. Each matching snippet is clamped to `MAX_GRANT_DOC_SNIPPET_LEN = 200` characters. Total payload per response is bounded under 8 KB.
- **Finding**: **MITIGATED**. Memory footprint is deterministic and bounded.

### Abuse Scenario AS-DOC-04: Sensitive Credential & Key Disclosure
- **Vector**: Documentation topics expose production private keys, authorization tokens, or environment secrets.
- **Analysis**: Review of all seven canonical topics (`grant-arch`, `grant-lifecycle`, `grant-attenuation`, `grant-revocation`, `grant-policy`, `grant-observability`, `grant-mcp`) confirmed only architectural descriptions, state machines, specification schemas, and mock examples are embedded. No secrets, credentials, or host-specific paths exist.
- **Finding**: **MITIGATED**. Zero credential leakage.

### Abuse Scenario AS-DOC-05: Unaudited Information Probing
- **Vector**: Adversary queries documentation tool repeatedly without generating an audit trail.
- **Analysis**: All invocations in `aiosh-mcp/src/main.rs` route through `dispatch::recorded_call`. Each call appends an immutable record with event sequence, timestamp, tool name, actor, and result status to the PEP audit ring buffer.
- **Finding**: **MITIGATED**. Complete audit coverage guaranteed.

## Verification Checklist
- [x] Input validation enforced for `action`, `topic_id`, and `query`.
- [x] Bounds enforcement: query length $\le 128$, results $\le 10$, snippet $\le 200$.
- [x] No path traversal or arbitrary filesystem access.
- [x] PEP audit trail recorded for every tool dispatch.
- [x] No sensitive credentials or secrets stored in canonical topics.
- [x] Zero open policy bypasses.
