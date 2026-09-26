# T-02289: Grant Lifecycle Documentation User & Operator Guide

## Overview
The PEP Grant Lifecycle documentation subsystem provides in-system, authoritative guidance for agents, operators, and tools on capability grant semantics, delegation depth, cascade revocation, security policies, and MCP JSON-RPC tool interfaces.

## MCP Tool Surface: `aios.pep.grant.doc`

### Tool Registration
- **Name**: `aios.pep.grant.doc`
- **Description**: Query Grant Lifecycle documentation topics, guides, and MCP references.
- **Parameters**:
  - `action` (string, required): One of `"list"`, `"get"`, `"search"`.
  - `topic_id` (string, optional): Topic identifier for `"get"` action (e.g. `"grant-arch"`, `"grant-revocation"`).
  - `query` (string, optional): Search term for `"search"` action (1 to 128 characters).

### Canonical Topics Catalog
1. **`grant-arch`**: Architectural overview of PEP authorization grants, single-writer state machines, and delegation trees.
2. **`grant-lifecycle`**: State transitions (`Active` -> `Suspended` -> `Revoked` -> `Expired`).
3. **`grant-attenuation`**: Attenuation rules: monotonically decreasing validity, subsets of scopes/tools, non-increasing max depth.
4. **`grant-revocation`**: Recursive cascade revocation semantics, parent-child tombstone propagation.
5. **`grant-policy`**: Security policies, depth ceiling constraints, high-risk scope restrictions.
6. **`grant-observability`**: Observability reports, point-in-time metrics, active vs revoked ratios.
7. **`grant-mcp`**: JSON-RPC interface catalog for all `aios.pep.grant.*` tools.

## Example Invocations

### 1. Listing All Available Documentation Topics
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.grant.doc",
    "arguments": {
      "action": "list"
    }
  }
}
```

### 2. Fetching a Specific Topic in Markdown
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.grant.doc",
    "arguments": {
      "action": "get",
      "topic_id": "grant-attenuation"
    }
  }
}
```

### 3. Searching Topics with a Query
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.pep.grant.doc",
    "arguments": {
      "action": "search",
      "query": "revocation cascade"
    }
  }
}
```

## Constraints & Limitations
- **Read-Only**: The documentation subsystem does not alter grant stores or policy files.
- **Search Scope**: Search performs case-insensitive lexical matching on titles, tags, summaries, and section contents. Vector/semantic embedding search is not performed at this layer.
- **Query Length**: Maximum search query length is 128 characters; longer queries are rejected.

## Related Evidence Artifacts
- [T-02281 Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02281-documentation-research.md)
- [T-02282 Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02282-documentation-specification.md)
- [T-02284 Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02284-documentation-implementation.md)
- [T-02287 Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02287-documentation-security-review.md)
- [T-02288 Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02288-documentation-hardening.md)
