# T-01839: Network Bootstrap / MCP/API Surface: Documentation

## 1. Overview
- **Task ID**: `T-01839`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Document the MCP/API surface of Network Bootstrap for operators and agents.

---

## 2. Documentation Updates
- Updated `docs/network_bootstrap.md` with **Section 7: Network Bootstrap MCP Tool Surface (`aios.network.*`)**.
- Documented:
  - Tool registry table with 7 tools (`aios.network.list`, `show`, `routes`, `dns`, `state`, `up`, `down`).
  - Parameter definitions, default behaviors, and grant requirements.
  - Invariants `NMCP1` through `NMCP6`.
  - JSON-RPC 2.0 tool-call examples.
  - Constraints and limitations.

## 3. Evidence References
- Research: [T-01831-mcp-api-surface-research.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01831-mcp-api-surface-research.md)
- Specification: [T-01832-mcp-api-surface-specification.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01832-mcp-api-surface-specification.md)
- Scaffold: [T-01833-mcp-api-surface-scaffold.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01833-mcp-api-surface-scaffold.md)
- Implementation: [T-01834-mcp-api-surface-implementation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01834-mcp-api-surface-implementation.md)
- Unit Tests: [T-01835-mcp-api-surface-unit-test.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01835-mcp-api-surface-unit-test.md)
- Integration Smoke: [T-01836-mcp-api-surface-integration.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01836-mcp-api-surface-integration.md)
- Security Review: [T-01837-mcp-api-surface-security-review.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01837-mcp-api-surface-security-review.md)
- Hardening: [T-01838-mcp-api-surface-hardening.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01838-mcp-api-surface-hardening.md)
