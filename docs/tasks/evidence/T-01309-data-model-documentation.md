# T-01309: Init & Service Supervision - Data Model: Documentation

## Metadata
- **Task ID:** `T-01309`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service`
- **Component:** Init & Service Supervision Data Model Documentation
- **Status:** Complete

## 1. Summary of Delivered Capabilities
The Init & Service Supervision Data Model provides a unified, cross-distribution service specification, state tracking, and validation layer supporting both systemd and OpenRC service architectures:

1. **Core Data Structures (`code/aiosh-rust/aiosh-core/src/service.rs`)**:
   - `ServiceType`: Service execution model (`Simple`, `Exec`, `Forking`, `Oneshot`, `Notify`, `Idle`).
   - `ServiceState`: Execution state (`Active`, `Inactive`, `Activating`, `Deactivating`, `Failed`, `Reloading`, `Unknown`).
   - `ServiceRestartPolicy`: Process restart policy on exit (`No`, `Always`, `OnSuccess`, `OnFailure`, `OnAbnormal`, `OnWatchdog`, `OnAbort`).
   - `ServiceStartupMode`: Boot and target enablement (`Enabled`, `Disabled`, `Masked`, `Static`).
   - `ServiceDependencyType`: Inter-service relationship type (`Requires`, `Wants`, `After`, `Before`, `Conflicts`).
   - `ServiceDependency`: Named directed dependency with optional flag.
   - `ServiceHealth`: Health metrics, exit code accounting, PID, uptime, restarts counter, and error reporting.
   - `ServiceSpec`: Full service unit definition including command paths, user/group credentials, working directory, environment variables, dependencies, and start/stop timeouts.
   - `ServiceStatus`: Runtime status linking spec, state, health, substate, and ISO-8601 timestamp.
   - `ServiceAction`: Discrete management operations (`Start`, `Stop`, `Restart`, `Reload`, `Enable`, `Disable`, `Mask`, `Unmask`).
   - `ServiceQuery`: Multi-attribute filtering container for querying service registries.

2. **Validation Invariants (`SS1..SS5`)**:
   - `SS1`: Service name syntax: length $[1 \dots 128]$, alphanumeric `[a-zA-Z0-9]`, `_`, `-`, `.`, starting with alphanumeric, valid extension (`.service`, `.socket`, `.target`, etc.) if specified.
   - `SS2`: Execution commands: non-empty `exec_start` ($\le 4,096$ chars), optional `exec_stop`/`exec_reload`, absolute `working_dir` without `..` traversal.
   - `SS3`: Dependency hygiene: no self-dependencies, no duplicates, max 128 dependencies, valid target names.
   - `SS4`: Resource and field limits: timeout in $[1 \dots 86,400]$ seconds, description $\le 4,096$ bytes, environment $\le 256$ entries (no `=` or null in keys), POSIX user/group names.
   - `SS5`: Lifecycle consistency: `Failed` state incompatible with `healthy == true`, `Masked` mode incompatible with `Active` or `Activating`.

3. **Operator CLI Surface (`aiosh service`)**:
   - `aiosh service validate --name <name> [--json]`: Validate service name syntax against SS1.
   - `aiosh service validate --spec <file_or_inline_json> [--json]`: Deep-audit service specification with 1 MiB payload ceiling against SS1..SS5.

4. **Autonomous Agent MCP Surface (`aios.service.validate`)**:
   - Accepts `{ "name": string }` or `{ "spec": object }`, returns standard response envelope with PEP authorization and audit logging.

## 2. Operator CLI Usage Examples

### Validating Service Name Syntax (SS1)
```bash
aiosh service validate --name aios-securityd.service
# VALID: Service name 'aios-securityd.service' conforms to SS1 naming syntax

aiosh service validate --name "bad/service" --json
```
Output:
```json
{
  "code": 2,
  "data": {
    "name": "bad/service",
    "valid": false
  },
  "error": {
    "code": "VALIDATION_FAILED",
    "errors": [
      "service name contains invalid character '/' in 'bad/service'"
    ],
    "message": "Service name 'bad/service' is invalid: service name contains invalid character '/' in 'bad/service'"
  }
}
```

### Validating Complete Service Specification (SS1..SS5)
```bash
aiosh service validate --spec '{
  "name": "aios-securityd.service",
  "description": "AIOS Security Daemon",
  "exec_start": "/usr/bin/aios-securityd --daemon",
  "exec_stop": null,
  "exec_reload": null,
  "service_type": "simple",
  "restart_policy": "always",
  "startup_mode": "enabled",
  "user": "aios",
  "group": "aios",
  "working_dir": "/var/lib/aios",
  "environment": {},
  "dependencies": [],
  "timeout_start_secs": 30,
  "timeout_stop_secs": 30
}' --json
```

## 3. MCP Tool Invocation Example
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.service.validate",
    "arguments": {
      "name": "aios-securityd.service"
    }
  }
}
```

Response:
```json
{
  "code": 0,
  "data": {
    "name": "aios-securityd.service",
    "ok": true,
    "tool": "aios.service.validate",
    "valid": true
  },
  "error": null
}
```

## 4. Constraints & Known Limitations
- The data model defines schemas and validation for service units; execution and process management via systemd/OpenRC D-Bus or PID 1 interfaces will be implemented in subsequent sub-epics.
- Service units are capped at 128 dependencies and 256 environment variables to ensure bounded memory and acyclic topological ordering.
- Specification inputs are restricted to 1 MiB (`1,048,576` bytes) to prevent resource exhaustion attacks.
