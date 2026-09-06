# aiosh-mcp

AIOS Model Context Protocol server (Sprint 0). Five tools, stdio
transport, single source of audit truth shared with `aiosh-cli` via the
same SQLite WAL ring.

Implements **ADR-0035 §D-2** (MCP as the only tool-call protocol).

## Tools exposed

| Tool                  | Reads | Writes | Notes |
|-----------------------|-------|--------|-------|
| `aios.fs.read`        | ✓     |        | Refused without grant; path restricted to `/tmp` and `$HOME/.aios` |
| `aios.process.list`   | ✓     |        | /proc on Linux; `ps` fallback |
| `aios.audit.tail`     | ✓     |        | Tail N rows of the hash chain |
| `aios.audit.verify`   | ✓     |        | Walk chain, confirm hashes; `full=True` replays archived segments (Sprint 3) |
| `aios.audit.rotate`   |       | ✓     | Sprint 3: seal live rows into an archived checkpoint segment; **requires grant** (`audit.rotate` / `audit.*`) |
| `aios.audit.segments` | ✓     |        | Sprint 3: list archived rotation checkpoints |
| `aios.audit.seen`     | ✓     |        | Sprint 3: bloom-backed "was this row hash ever logged?" (`exact=True` confirms via archive scan) |
| `aios.pentest.nmap`   |       | ✓¹    | Real `nmap` if on PATH; otherwise "would-run" stub |
| `aios.service.validate` | ✓   |        | Validates service name syntax (SS1) or full `ServiceSpec` against invariants (SS1..SS5) |
| `aios.service.list`   | ✓     |        | Lists registered services with optional pattern, state, and startup mode filtering |
| `aios.service.get`    | ✓     |        | Retrieves detailed specification and runtime status of a service by canonical name |
| `aios.service.action` |       | ✓     | Executes lifecycle action (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`) |
| `aios.service.order`  | ✓     |        | Computes topological startup sequence for a service and its dependency graph |

¹ The pentest and service action tools write an audit row through the same recorded dispatch helper; in Sprint 0 the row is written synchronously.

## Service Supervision Tools (`aios.service.*`)

The Init & Service Supervision tools provide programmatic control over platform services via standard MCP JSON-RPC 2.0:

### Copy-Pasteable Tool Call Examples

#### 1. Validate Service Name or Specification (`aios.service.validate`)
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

#### 2. Query Services with Filters (`aios.service.list`)
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

#### 3. Inspect Service Metadata & Status (`aios.service.get`)
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

#### 4. Execute Service Lifecycle Action (`aios.service.action`)
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

#### 5. Compute Startup Dependency Sequence (`aios.service.order`)
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

### Constraints & Known Limitations
- **Name Format**: Service names are limited to 128 ASCII characters matching `^[a-zA-Z0-9][a-zA-Z0-9_\-\.]{0,126}\.service$`. Control characters are strictly rejected.
- **Masking Invariant**: A service in `masked` mode cannot be started or enabled. Active services must be stopped before they can be masked.
- **Persistence Boundary**: By default, tools operate on the active runtime store; passing `store_path` reads and atomically persists changes to the designated JSON file using PID-isolated temp files.

## Running

```bash
cd code/aiosh-mcp
python -m aiosh_mcp.server
```

The server speaks MCP over stdio, so any MCP client (e.g. the
`mcp` CLI, Claude Desktop with this stdio server registered, an Ollama
tool bridge, or our own `aiosh-cli agent` once Sprint-1 lands) can call
the tools.

## Architecture references

- **ADR-0035 §D-2** — MCP is the only tool-call protocol.
- **ADR-0035 §D-4** — Constitution C-1..C-3 enforced at the tool gate.
- **AI_CONSTITUTION.md §1.4 C-1** — pentest tools require explicit grant.
- **AI_CONSTITUTION.md §1.4 C-3** — non-reversible effects (fs.write,
  pentest, shutdown) require granular consent.

## Sampling primitive

**Not exposed.** Per ADR-0035 §5, the deprecated MCP Sampling primitive
is hard-removed from our MCP manifest to eliminate the prompt-injection
vector that asks a server to insert reasoning into a tool response.

## Tests

```bash
python tests/test_smoke.py
```
