# Task Evidence: T-02379 - Audit Chain Extensions: Observability Documentation

## Goal
Document the observability subsystem of Audit Chain Extensions for operators, agents, and monitoring tools.

## Documentation Summary
1. **Specification Update**:
   - Updated `docs/SPEC-AUDIT-EXTENSIONS.md` with Section 7 ("Audit Chain Observability Subsystem").
   - Detailed metric schemas, cardinality metrics, outcome distributions, text sanitization rules, and bounding parameters.
2. **Commands & Invocations**:
   - Documented CLI invocations: `aiosh audit stats` and `aiosh audit stats --json`.
   - Documented MCP JSON-RPC tool invocation: `{"name": "aios.audit.stats", "arguments": {}}`.
3. **Constraints & Known Limitations**:
   - In-memory sets bounded to 100,000 entries.
   - Outcome distribution bounded to top 128 categories.
   - String fields sanitized to 256 printable ASCII/UTF-8 characters.
4. **Evidence Cross-References**:
   - Linked all tasks from T-02371 to T-02380.
