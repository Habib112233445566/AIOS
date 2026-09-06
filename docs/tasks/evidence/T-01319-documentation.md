# T-01319 — Init & Service Supervision / Core Service: Documentation

## 1. Documentation Updates
- Updated `docs/README.md` §8.13 with operator CLI and autonomous agent MCP usage guidelines for the Init & Service Supervision core service (`aiosh-core::service_service`, `ServiceStore`).
- Documented CLI commands (`aiosh service list`, `aiosh service show`, `aiosh service action`, `aiosh service order`, `aiosh service validate`).
- Documented MCP tools (`aios.service.list`, `aios.service.get`, `aios.service.action`, `aios.service.order`, `aios.service.validate`).
- Linked evidence chain from `T-01301` through `T-01319`.

## 2. Copy-Pasteable Usage Examples
```bash
# 1. List registered services with state or mode filter
aiosh service list --mode enabled

# 2. Show detailed specification and runtime status of a service
aiosh service show aios-securityd.service --json

# 3. Trigger state machine lifecycle transition
aiosh service action aios-securityd.service restart

# 4. Plan topological startup sequence for a service and its dependencies
aiosh service order aios-securityd.service

# 5. Validate a unit specification file
aiosh service validate --spec tests/fixtures/services/valid_service.json
```

## 3. Autonomous Agent MCP Tool Calls
```json
// Example: Requesting topological execution sequence via MCP
{
  "tool": "aios.service.order",
  "arguments": {
    "name": "aios-securityd.service"
  }
}
```

## 4. Honest Limitations & Constraints
- In-memory service store persists atomically to JSON; physical process lifecycle supervision (`fork`, `execve`, cgroup management) is orchestrated by downstream init runner components.
- Store file loading strictly enforces a 10 MiB ceiling (`MAX_STORE_BYTES`) and 10,000 maximum service entries.
- Timeouts are bounded between 1 second and 86,400 seconds (24 hours).
