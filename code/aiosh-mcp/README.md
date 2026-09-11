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
| `aios.service.config` | ✓     |        | Inspects Init & Service Supervision configuration parameters (SC1..SC7) |
| `aios.service.policy` | ✓     |        | Evaluates or inspects service security policies against specifications (SP1..SP6) |
| `aios.service.stats`  | ✓     |        | Inspects service observability metrics, inventory distributions, and health telemetry (SO1..SO6) |
| `aios.service.check`  | ✓     |        | Validates on-disk service store integrity and optionally performs non-destructive recovery (SR1..SR5) |
| `aios.session.validate` | ✓   |        | Validates session ID, username syntax (SB1, SB2) or full `UserSessionSpec` against invariants (SB1..SB5) |
| `aios.session.list`   | ✓     |        | Lists active/tracked user sessions with optional user, state, type, seat, limit filtering |
| `aios.session.get`    | ✓     |        | Retrieves runtime status and specification of a session by ID |
| `aios.session.action` |       | ✓     | Executes lifecycle action (`authenticate`, `activate`, `lock`, `unlock`, `terminate`) with seat arbitration |
| `aios.session.create` |       | ✓     | Atomically provisions a new user or agent session with validation and capacity enforcement |
| `aios.session.config` | ✓     |        | Inspects User Session Bootstrap configuration parameters and limits (SC1..SC7) |

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

#### 6. Inspect Service Observability Metrics (`aios.service.stats`)
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "aios.service.stats",
    "arguments": {}
  }
}
```

#### 7. Validate and Recover Service Store (`aios.service.check`)
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "aios.service.check",
    "arguments": {
      "store_path": "/var/lib/aios/services.json",
      "auto_recover": true
    }
  }
}
```

### User Session Bootstrap Tools

#### Validate Session Identifier, Username, or Specification (`aios.session.validate`)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.session.validate",
    "arguments": {
      "session_id": "sess-01"
    }
  }
}
```
Validate complete session specification against formal invariants (`SB1..SB5`):
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.session.validate",
    "arguments": {
      "spec": {
        "session_id": "sess-01",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "x11",
        "session_class": "user",
        "seat": "seat0",
        "vtnr": 7,
        "display": ":0",
        "remote_host": null,
        "environment": {
          "XDG_RUNTIME_DIR": "/run/user/1000",
          "DISPLAY": ":0"
        }
      }
    }
  }
}
```

#### Query Tracked Sessions (`aios.session.list`)
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.session.list",
    "arguments": {
      "state": "active",
      "seat": "seat0"
    }
  }
}
```

#### Inspect Session Specification & Status (`aios.session.get`)
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "aios.session.get",
    "arguments": {
      "session_id": "greeter-seat0"
    }
  }
}
```

#### Execute Lifecycle Action (`aios.session.action`)
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "aios.session.action",
    "arguments": {
      "session_id": "sess-01",
      "action": "activate"
    }
  }
}
```

#### Provision Session (`aios.session.create`)
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "aios.session.create",
    "arguments": {
      "spec": {
        "session_id": "sess-agent-01",
        "username": "aios-agent",
        "uid": 1001,
        "gid": 1001,
        "session_type": "ai_agent",
        "session_class": "agent",
        "seat": "seat0",
        "vtnr": null,
        "display": null,
        "remote_host": null,
        "environment": {
          "AIOS_AGENT_MODE": "autonomous",
          "AIOS_AGENT_ID": "agent-copilot"
        }
      },
      "store_path": "/tmp/.aios/sessions.json"
    }
  }
}
```

### CLI Command Examples (`aiosh session`)
- Validate session spec: `aiosh session validate --spec /path/to/spec.json --json`
- List active sessions: `aiosh session list --state active --seat seat0 --limit 50 --json`
- Show session status: `aiosh session show greeter-seat0 --json`
- Lock / Unlock session: `aiosh session action greeter-seat0 lock --json`
- Create session: `aiosh session create --id sess-agent-01 --user aios-agent --type agent --seat seat0 --json`

### User Session Constraints & Invariants (CS1..CS5, SB1..SB5)
- **Seat Mutual Exclusion (CS2)**: At most one session can hold `SessionScope::Foreground` on any physical/virtual seat (e.g. `seat0`). Activating a session automatically demotes prior foreground sessions on that seat to `SessionScope::Background`.
- **State Monotonicity & Two-Stage Teardown (CS1)**: Terminated sessions (`state == Terminated`) are immutable and permanent; any subsequent action fails with an explicit error envelope. Active and Locked sessions follow a two-stage termination path: `terminate` transitions to `Terminating` (process cleanup), and a subsequent `terminate` finalizes transition to `Terminated`.
- **Store & User Capacities (CS3, SB5)**: Maximum 32 active sessions per user (`MAX_SESSIONS_PER_USER`) and maximum 1,024 total tracked sessions in store (`MAX_TOTAL_SESSIONS`).
- **Lock State Invariant (CS5)**: `state == Locked` strictly matches `locked == true`; `state == Active` strictly matches `locked == false`. Touching activity resets `idle_seconds` to 0 but never unlocks a session.
- **Hardening Ceilings & Bounds**:
  - Request line ceiling: 1 MiB (`1,048,576` bytes) transport-level limit.
  - Specification payload ceiling: 1 MiB limit on inline JSON objects and strings in `validate` and `create`.
  - Query bounds: `limit` parameter in `aios.session.list` strictly enforced to $[1 \dots 10,000]$.
  - Path sanitization: `store_path` parameter strictly bounded to $\le 1024$ characters with zero control characters.
  - Session identifier: strictly validated against `SB1` ($[1 \dots 64]$ chars, alphanumeric start, no slashes or traversal tokens).
  - Atomic persistence: 10 MiB store limit (`MAX_SESSION_STORE_SIZE`), atomic rename via PID-isolated temporary files (`.tmp.<pid>.<nanos>`), and guaranteed tempfile cleanup on write/rename failure.

### Evidence & Implementation Artifacts
- **Research:** [`T-01431-mcp-api-surface-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01431-mcp-api-surface-research.md)
- **Specification:** [`T-01432-mcp-api-surface-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01432-mcp-api-surface-specification.md)
- **Scaffold:** [`T-01433-mcp-api-surface-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01433-mcp-api-surface-scaffold.md)
- **Implementation:** [`T-01434-mcp-api-surface-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01434-mcp-api-surface-implementation.md)
- **Unit Testing:** [`T-01435-mcp-api-surface-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01435-mcp-api-surface-unit-test.md)
- **Integration:** [`T-01436-mcp-api-surface-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01436-mcp-api-surface-integration.md)
- **Security Review:** [`T-01437-mcp-api-surface-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01437-mcp-api-surface-security-review.md)
- **Hardening:** [`T-01438-mcp-api-surface-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01438-mcp-api-surface-hardening.md)
- **Documentation:** [`T-01439-mcp-api-surface-documentation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01439-mcp-api-surface-documentation.md)

### Constraints & Known Limitations
- **Name Format**: Service names are limited to 128 ASCII characters matching `^[a-zA-Z0-9][a-zA-Z0-9_\-\.]{0,126}\.service$`. Control characters are strictly rejected.
- **Masking Invariant**: A service in `masked` mode cannot be started or enabled. Active services must be stopped before they can be masked.
- **Persistence Boundary**: By default, tools operate on the active runtime store; passing `store_path` reads and atomically persists changes to the designated JSON file using PID-isolated temp files.
### User Session Bootstrap Tools

The User Session Bootstrap tools provide programmatic management, inspection, and security policy evaluation for interactive and autonomous AI agent sessions:

#### `aios.session.validate`
Validates session identifiers, user identity bounds, or full session specifications against invariants.

#### `aios.session.list`
Queries tracked user and agent sessions with optional filters (`session_type`, `state`, `username`, `seat`, `limit`).

#### `aios.session.get`
Retrieves detailed session status and specification by `session_id`.

#### `aios.session.action`
Executes lifecycle state transitions (`authenticate`, `activate`, `lock`, `unlock`, `terminate`) on a session.

#### `aios.session.create`
Bootstraps and initializes a new user or agent session specification.

#### `aios.session.config`
Inspects runtime session parameters, timeouts, and capacity ceilings.

#### `aios.session.policy`
Evaluates user session specifications or entire session stores against `UserSessionSecurityPolicy` (`SSP1..SSP7`):

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "aios.session.policy",
    "arguments": {
      "spec": {
        "session_id": "agent-eval-01",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "ai_agent",
        "session_class": "agent",
        "seat": "seat0"
      }
    }
  }
}
```

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
