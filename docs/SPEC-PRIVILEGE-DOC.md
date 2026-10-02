# Specification: Privilege Escalation Prevention Documentation (SPEC-PRIVILEGE-DOC)

- **Status**: APPROVED
- **Date**: 2026-10-02
- **Scope**: Canonical offline technical documentation, keyword search index, and topic rendering for Privilege Escalation Prevention.

## 1. Data Contracts

### 1.1 Topic Categories
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeDocCategory {
    Architecture,
    Lifecycle,
    Policy,
    Observability,
    Security,
    Recovery,
}
```

### 1.2 Topic & Section Models
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeDocSection {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeDocTopic {
    pub id: String,
    pub title: String,
    pub category: PrivilegeDocCategory,
    pub summary: String,
    pub sections: Vec<PrivilegeDocSection>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeDocSearchResult {
    pub topic_id: String,
    pub title: String,
    pub score: usize,
    pub snippet: String,
}
```

## 2. Invariants
- `PRIVDOC1`: Index pre-populates at least 6 canonical topics covering Architecture, Lifecycle, Policy, Observability, SystemKernel Immutability, and Recovery.
- `PRIVDOC2`: Search query must be bounded: $1 \le \text{len} \le 128$ bytes, returning at most 10 results. Queries with control characters are rejected.
- `PRIVDOC3`: Relevance scoring weights:
  - Exact or partial Tag match: +10 points
  - Title match: +5 points
  - Summary match: +3 points
  - Section title match: +2 points
  - Section body match: +1 point
- `PRIVDOC4`: Rendering to Markdown includes title, category, tags, summary, formatted sections, and copy-pasteable CLI/MCP examples.
- `PRIVDOC5`: Error codes:
  - `PRIVDOC_ERR_NOT_FOUND`: Topic ID not present in index.
  - `PRIVDOC_ERR_QUERY_BOUNDS`: Empty query, control characters, or query exceeds 128 characters.

## 3. Operator & Agent Invocation

### 3.1 CLI Commands
List available privilege topics:
```bash
aiosh privilege doc list
aiosh privilege doc list --json
```

Retrieve a specific documentation topic:
```bash
aiosh privilege doc get priv-arch
aiosh privilege doc get priv-arch --json
```

Search topics with keyword scoring:
```bash
aiosh privilege doc search "kernel lockout"
aiosh privilege doc search "kernel lockout" --json
```

### 3.2 MCP Tool Calls
Agents can use the `aios.privilege.doc` tool:
```json
{
  "name": "aios.privilege.doc",
  "arguments": {
    "action": "search",
    "query": "elevation policy"
  }
}
```

## 4. Constraints and Known Limitations
1. **In-Memory Storage**: Topics are compiled into the core library binary, ensuring zero disk dependency and zero I/O latency.
2. **Result Limits**: Search responses are capped at 10 items (`MAX_PRIVILEGE_DOC_SEARCH_RESULTS`).
3. **Query Length**: Queries are bounded to 128 characters (`MAX_PRIVILEGE_DOC_QUERY_LEN`).

## 5. Sub-Epic Task Evidence
- [T-02581 Documentation Research](tasks/evidence/T-02581-documentation-research.md)
- [T-02582 Documentation Specification](tasks/evidence/T-02582-documentation-specification.md)
- [T-02583 Documentation Scaffold](tasks/evidence/T-02583-documentation-scaffold.md)
- [T-02584 Documentation Implementation](tasks/evidence/T-02584-documentation-implementation.md)
- [T-02585 Documentation Unit Test](tasks/evidence/T-02585-documentation-unit-test.md)
- [T-02586 Documentation Integration](tasks/evidence/T-02586-documentation-integration.md)
- [T-02587 Documentation Security Review](tasks/evidence/T-02587-documentation-security-review.md)
- [T-02588 Documentation Hardening](tasks/evidence/T-02588-documentation-hardening.md)
- [T-02589 Documentation Documentation](tasks/evidence/T-02589-documentation-documentation.md)
- [T-02590 Documentation Verification](tasks/evidence/T-02590-documentation-verification-evidenc.md)
