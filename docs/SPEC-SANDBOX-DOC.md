# Specification: Sandbox Enforcement Documentation Subsystem

## 1. Overview
The Sandbox Enforcement Documentation Subsystem (`SandboxDocIndex`) provides an embedded, completely offline documentation repository and lexical search engine for operators, scripts, and autonomous agents within the AIOS environment.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `SANDBOXDOC1` | **Canonical Topic Set** | The default repository MUST provide 6 canonical topics: `overview`, `profiles`, `isolation`, `policy`, `observability`, `reference`. |
| `SANDBOXDOC2` | **Case-Insensitive Resolution** | Topic IDs are resolved case-insensitively with leading and trailing whitespace stripped. |
| `SANDBOXDOC3` | **Deterministic Relevance Search** | Queries are scored against topic IDs, titles, keyword tags, summaries, and section contents. |
| `SANDBOXDOC4` | **Bounded Search & Allocation** | Query strings are clamped to `MAX_DOC_QUERY_LEN` (128 chars), and results are truncated to `MAX_DOC_RESULTS` (32 items). |
| `SANDBOXDOC5` | **Control Character Sanitization** | Non-printable and control characters (< 0x20 and 0x7F) are stripped from incoming queries before evaluation. |
| `SANDBOXDOC6` | **Zero Outbound Dependencies** | All content is statically embedded into `aiosh-core`. Lookups require zero network or disk I/O. |

---

## 3. Interfaces & Usage

### CLI Commands
```bash
# List all available documentation topics
aiosh sandbox doc

# View topics in structured JSON
aiosh sandbox doc --json

# View specific documentation topic
aiosh sandbox doc overview

# Perform keyword search across topics
aiosh sandbox doc --search landlock
```

### MCP Tool Interface
- Tool: `aios.sandbox.doc`
- Parameters:
  - `topic` (string, optional): Exact topic ID to retrieve.
  - `search` (string, optional): Keyword query to match against repository.
- Example MCP Call:
```json
{
  "name": "aios.sandbox.doc",
  "arguments": {
    "search": "landlock"
  }
}
```

---

## 4. Constraints & Known Limitations
1. The documentation index is read-only and static; runtime additions require recompilation or extending the core library.
2. Search uses substring lexical matching and relevance weighting; semantic vector embeddings are not used in order to maintain zero external model dependencies in the core security kernel.

---

## 5. Linked Task Evidence
- Unit Testing: `docs/tasks/evidence/T-02485-documentation-unit-test.md`
- Integration: `docs/tasks/evidence/T-02486-documentation-integration.md`
- Security Review: `docs/tasks/evidence/T-02487-documentation-security-review.md`
- Hardening: `docs/tasks/evidence/T-02488-documentation-hardening.md`
