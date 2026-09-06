# T-01339: Init & Service Supervision - MCP/API Surface: Documentation

## Metadata
- **Task ID:** `T-01339`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface Documentation
- **Status:** Complete (Documentation)

---

## 1. Documentation Summary
Documented the complete MCP service supervision API surface (`aios.service.*`) in `code/aiosh-mcp/README.md`:
1. Updated the Tools Exposed table with the 5 service supervision tools:
   - `aios.service.validate`
   - `aios.service.list`
   - `aios.service.get`
   - `aios.service.action`
   - `aios.service.order`
2. Provided copy-pasteable JSON-RPC 2.0 requests for all 5 tools over the stdio transport.
3. Explicitly recorded security constraints, character length limits, and state machine invariants (e.g. masking rules and atomic persistence).

---

## 2. Copy-Pasteable Tool Call Examples

### 2.1 Validate Service Name
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.service.validate",
    "arguments": {
      "name": "auditd.service"
    }
  }
}
```

### 2.2 List Active Services with Pattern Match
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.service.list",
    "arguments": {
      "state": "active",
      "pattern": "audit"
    }
  }
}
```

### 2.3 Retrieve Service Specification & Runtime Status
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.service.get",
    "arguments": {
      "name": "auditd.service"
    }
  }
}
```

### 2.4 Execute Service Lifecycle Action
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "aios.service.action",
    "arguments": {
      "name": "auditd.service",
      "action": "restart"
    }
  }
}
```

### 2.5 Compute Topological Startup Plan
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "aios.service.order",
    "arguments": {
      "name": "aios-securityd.service"
    }
  }
}
```

---

## 3. Constraints & Operational Limitations
- **Syntax Boundaries**: Service names are strictly checked against `^[a-zA-Z0-9][a-zA-Z0-9_\-\.]{0,126}\.service$`. No control characters are permitted in any string argument.
- **State Machine Rules**: Masked services reject activation attempts (`start`, `restart`, `reload`, `enable`). Active services cannot be masked without first being stopped.
- **Persistence Guarantees**: Mutations write to a PID-tagged temp file with flush/fsync and atomic rename, eliminating partial-write corruptions.

---

## 4. Evidence Links
- Research: [T-01331-mcp-api-surface-research.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01331-mcp-api-surface-research.md)
- Specification: [T-01332-mcp-api-surface-specification.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01332-mcp-api-surface-specification.md)
- Scaffold: [T-01333-mcp-api-surface-scaffold.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01333-mcp-api-surface-scaffold.md)
- Implementation: [T-01334-mcp-api-surface-implementation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01334-mcp-api-surface-implementation.md)
- Unit Test: [T-01335-mcp-api-surface-unit-test.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01335-mcp-api-surface-unit-test.md)
- Integration: [T-01336-mcp-api-surface-integration.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01336-mcp-api-surface-integration.md)
- Security Review: [T-01337-mcp-api-surface-security-review.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01337-mcp-api-surface-security-review.md)
- Hardening: [T-01338-mcp-api-surface-hardening.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01338-mcp-api-surface-hardening.md)
