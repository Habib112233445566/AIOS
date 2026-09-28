# Task T-02489 Evidence: Sandbox Documentation Documentation

## Goal
Document the Sandbox Enforcement Documentation Subsystem for operators and agents.

## Delivered Documentation
- Created `docs/SPEC-SANDBOX-DOC.md` containing:
  - Six formal invariants (`SANDBOXDOC1` through `SANDBOXDOC6`).
  - CLI usage examples (`aiosh sandbox doc`, `aiosh sandbox doc <topic>`, `aiosh sandbox doc --search <query>`).
  - Model Context Protocol (MCP) tool schema and example request payload for `aios.sandbox.doc`.
  - Operational bounds, result limits, and zero-network design constraints.
  - Linked references to evidence files T-02485 through T-02488.

## Copy-Pasteable Usage Examples
```bash
# Browse topics
aiosh sandbox doc

# Query topic
aiosh sandbox doc isolation

# Search keyword
aiosh sandbox doc --search seccomp
```
