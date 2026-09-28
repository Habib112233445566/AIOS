# Task Evidence: T-02389 - Audit Chain Extensions: Documentation Documentation

## Goal
Document the documentation subsystem of Audit Chain Extensions for operators, agents, and client integrations.

## Documentation Summary
1. **Specification Update**:
   - Updated `docs/SPEC-AUDIT-EXTENSIONS.md` with Section 8 ("Audit Chain Documentation Subsystem").
   - Detailed topic catalogue (`audit-arch`, `audit-lineage`, `audit-crypto`, `audit-policy`, `audit-observability`, `audit-recovery`, `audit-reference`).
2. **Commands & Invocations**:
   - Documented CLI invocations: `aiosh audit doc`, `aiosh audit doc <topic>`, `aiosh audit doc <query> [--json]`.
   - Documented MCP JSON-RPC tool invocation: `{"name": "aios.audit.doc", "arguments": {"query": "signature"}}`.
3. **Constraints & Known Limitations**:
   - Query length bounded to 128 characters.
   - Max search tokens parsed is 16.
   - Snippet length truncated to 200 characters.
   - Results truncated to top 10 ranked matches.
4. **Evidence Cross-References**:
   - Linked tasks T-02381 to T-02390.
