# Task Evidence: T-02381 - Audit Chain Extensions: Documentation Research

## Goal
Establish facts, constraints, and design patterns for the Audit Chain Extensions documentation subsystem.

## Research & Prior Art

### 1. Existing Documentation Architectures in AIOS
- Examination of existing `aiosh-core` documentation modules:
  - `pep_grant_doc.rs`, `pep_doc.rs`, `capability_doc.rs`, `kernel_module_doc.rs`, `network_doc.rs`.
- Standard architecture pattern:
  - Pure in-memory canonical documentation index (`DocIndex`) with zero external filesystem/network dependencies during search.
  - Categories enum (`enum AuditChainDocCategory`).
  - Structured topics (`AuditChainDocTopic`) with sections, tags, examples.
  - Lexical token-based search with keyword scoring and bounded snippet generation (`AuditChainDocSearchResult`).
  - Strict input bounding: max query length (128 chars), max search results (10), max snippet length (200 chars).
  - Explicit error codes: `AUDITDOC_ERR_NOT_FOUND`, `AUDITDOC_ERR_QUERY_BOUNDS`.

### 2. Subsystem Scope for Audit Chain Extensions Documentation
The documentation subsystem must cover:
1. **Architecture & Ledger**: SQLite audit ring schema, provenance, extensions JSON, and hash chain.
2. **Causal Lineage & DAG**: Parent-child links, ancestry queries, diamond DAG resolution, cycle resistance.
3. **Cryptographic Signatures**: Ed25519 signatures, payload hashing, key verification.
4. **Security Policy**: Enforcing vs permissive vs disabled modes, rules, and constraints.
5. **Observability & Telemetry**: Health reports, cardinality stats, outcomes histograms.
6. **Recovery & Repair**: Detection of chain corruption, tamper evidence, repair mechanisms.
7. **Tool Reference**: CLI subcommands (`aiosh audit ...`) and MCP tools (`aios.audit.*`).

### 3. Decisions & Invariants
- **No external I/O required**: Canonical doc index initialized in memory at compile-time/runtime without requiring external markdown reading on every tool call.
- **MCP & CLI exposure**: CLI will expose `aiosh audit doc [query]` or `aiosh audit doc <topic>`, and MCP will expose `aios.audit.doc` with `list`, `get`, or `search` actions.
- **Fail-safe query bounds**: Excessively long queries rejected with `AUDITDOC_ERR_QUERY_BOUNDS`; control chars stripped before evaluation.

## Citations
- `code/aiosh-rust/aiosh-core/src/pep_grant_doc.rs`
- `docs/SPEC-AUDIT-EXTENSIONS.md`
