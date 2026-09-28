# T-02439: Sandbox Enforcement MCP/API Surface Documentation

## 1. Documentation Overview
This document records verification evidence for task T-02439: Sandbox Enforcement MCP/API Surface Documentation.
The complete MCP/API surface was documented in [`docs/SPEC-SANDBOX-MCP.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/SPEC-SANDBOX-MCP.md).

## 2. Tools & Invocations
The documentation provides working JSON-RPC 2.0 payloads for:
1. `aios.sandbox.profiles`: Profile listing and policy inspection.
2. `aios.sandbox.probe`: Host platform kernel sandboxing discovery.
3. `aios.sandbox.exec`: Sandboxed command dispatch with profile, argument, and working directory control.

## 3. Constraints & Limitations
- Commands must not contain null bytes.
- Working directory must not contain directory traversal sequences (`..`).
- Buffer captures are bounded at 10 MiB.
- Execution requires PEP authorization when enforcement is active.

## 4. Evidence References
- Research: [`T-02431-mcp-api-surface-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02431-mcp-api-surface-research.md)
- Specification: [`T-02432-mcp-api-surface-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02432-mcp-api-surface-specification.md)
- Scaffold: [`T-02433-mcp-api-surface-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02433-mcp-api-surface-scaffold.md)
- Implementation: [`T-02434-mcp-api-surface-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02434-mcp-api-surface-implementation.md)
- Unit Testing: [`T-02435-mcp-api-surface-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02435-mcp-api-surface-unit-test.md)
- Integration: [`T-02436-mcp-api-surface-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02436-mcp-api-surface-integration.md)
- Security Review: [`T-02437-mcp-api-surface-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02437-mcp-api-surface-security-review.md)
- Hardening: [`T-02438-mcp-api-surface-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02438-mcp-api-surface-hardening.md)
