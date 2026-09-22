# Task Evidence: T-02189 (documentation: Documentation)

## 1. Documentation Updates
Section 13 has been added to `docs/pep_decision_engine.md`:
- Documented `PepDocIndex` subsystem architecture and role as the authoritative zero-dependency offline knowledge base for PEP.
- Formally recorded invariants `PEPDOC1..PEPDOC6`.
- Provided copy-pasteable invocation examples for CLI commands (`aiosh pep doc list`, `show`, `search`) and MCP tool invocations (`aios.pep.doc`).
- Honestly stated constraints: in-memory static index, 256-character query limits, 50-item search truncation, UTF-8 snippet truncation at 160 characters.
- Cross-linked evidence artifacts for tasks `T-02184` through `T-02190`.

## 2. Example Usage
```bash
# Search for PEP combining algorithm documentation
aiosh pep doc search DenyOverrides

# Retrieve complete Markdown topic
aiosh pep doc show pep-arch

# JSON envelope for programmatic consumption
aiosh pep doc list --category architecture --json
```

## 3. Acceptance Verification
- [x] Documentation updated with working examples.
- [x] Constraints and limits stated transparently.
- [x] Subsystem reference integrated into `docs/pep_decision_engine.md`.
