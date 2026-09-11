# aiosh-cli

AIOS shell CLI (Sprint 0). Linux-substrate userspace command for the AIOS
subsystem surface. Every subcommand emits exactly one append-only,
hash-chained audit row to a SQLite WAL ring.

Implements:
- **ADR-0035 §D-2** (single audit substrate) — for CLI surface only; MCP is the AI tool surface.
- **AI_CONSTITUTION §1.4 C-1..C-3** mechanically — via PEP grant tokens.

## Install / build

```bash
cd code/aiosh-cli
npm install
npm run build   # tsc → dist/
```

Run a subcommand:

```bash
node dist/cli.js status
node dist/cli.js run whoami
node dist/cli.js audit tail 5
```

## Subcommands

| Subcommand              | What it does                                     |
|-------------------------|--------------------------------------------------|
| `aiosh status`          | Print env, Constitution revision, ring head hash |
| `aiosh run <cmd...>`    | Run a host command; stdout/stderr + audit row    |
| `aiosh agent <prompt>`  | Invoke agent (Sprint 0: stub; Sprint 1: Ollama)  |
| `aiosh audit tail [n]`  | Tail last N rows                                 |
| `aiosh audit verify`    | Verify SHA-256 ring chain                |
| `aiosh grant create`    | Issue PEP grant (audited)                        |
| `aiosh grant list`      | List active grants                               |
| `aiosh grant revoke <id>` | Revoke grant (audited)                         |
| `aiosh service <subcmd>`| Init & Service Supervision commands (`validate`, `list`, `get`, `action`, etc.) |
| `aiosh session <subcmd>`| User Session Bootstrap lifecycle, seat arbitration, and session inspection |

## User Session Bootstrap CLI Surface (`aiosh session`)

The `aiosh session` surface provides operator and AI agent management over graphical, console, remote, and service sessions on Linux seats (`loginctl(1)` semantics, ADR-0035 §D-2, SB1..SB5, CS1..CS5).

### Subcommands

| Subcommand | Description | Syntax / Options |
|---|---|---|
| `aiosh session validate` | Validate session identifier, user, or specification payload | `aiosh session validate [--id <id>] [--user <name>] [--spec <file_or_json>] [--json]` |
| `aiosh session list` | List tracked sessions with optional filtering | `aiosh session list [--user <name>] [--state <state>] [--type <type>] [--seat <seat>] [--limit <n>] [--store <path>] [--json]` |
| `aiosh session show` | Inspect specification, runtime state, and audit status | `aiosh session show <session_id> [--store <path>] [--json]` |
| `aiosh session status` | Alias for `show` with identical parameters | `aiosh session status <session_id> [--store <path>] [--json]` |
| `aiosh session action` | Execute state transition action | `aiosh session action <session_id> <action> [--reason <reason>] [--store <path>] [--json]` |
| `aiosh session activate` | Shortcut to activate session onto seat | `aiosh session activate <session_id> [--store <path>] [--json]` |
| `aiosh session lock` | Shortcut to lock session display/input | `aiosh session lock <session_id> [--store <path>] [--json]` |
| `aiosh session unlock` | Shortcut to unlock session | `aiosh session unlock <session_id> [--store <path>] [--json]` |
| `aiosh session terminate` | Shortcut to terminate session | `aiosh session terminate <session_id> [--reason <reason>] [--store <path>] [--json]` |
| `aiosh session auth` | Shortcut to mark session authenticated | `aiosh session auth <session_id> [--store <path>] [--json]` |
| `aiosh session create` | Ingest specification and bootstrap new session | `aiosh session create <spec_file_or_json> [--store <path>] [--json]` |
| `aiosh session config` | Inspect resolved session configuration and limits | `aiosh session config [--config <path>] [--json]` |

### Copy-Pasteable Invocations

#### 1. Inspect Tracked Sessions
```bash
aiosh session list --state active --seat seat0 --json
```

#### 2. Query Session Status
```bash
aiosh session status greeter-seat0 --json
```

#### 3. Create Session from Inline JSON Spec
```bash
aiosh session create '{"session_id":"sess-user1","username":"user1","uid":1000,"gid":1000,"session_type":"wayland","session_class":"user","seat":"seat0","vtnr":1,"display":":0","remote_host":null,"environment":{"XDG_RUNTIME_DIR":"/run/user/1000"}}' --json
```

#### 4. Activate Session with Seat Arbitration
```bash
aiosh session activate sess-user1 --json
```

#### 5. Lock and Unlock Session
```bash
aiosh session lock sess-user1 --json
aiosh session unlock sess-user1 --json
```

#### 6. Terminate Session
```bash
aiosh session terminate sess-user1 --reason "user logout" --json
```

### Response Envelopes

Every CLI execution in `--json` mode returns an envelope adhering to ADR-0035 §D-2:
```json
{
  "code": 0,
  "data": { ... },
  "error": null
}
```

On validation or operational error:
```json
{
  "code": 2,
  "data": null,
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "User session validation failed: Session ID contains invalid characters"
  }
}
```

### Constraints & Known Limitations
- **1 MiB Payload Cap**: Ingested JSON specifications (`--spec` or `create <spec>`) are capped at 1,048,576 bytes. Oversized payloads return exit code 2 and `PAYLOAD_TOO_LARGE`.
- **Store Path Bounds**: `--store <path>` is limited to 1,024 bytes and must not contain ASCII control characters (`c.is_control()`).
- **Query Limit Bounds**: `--limit <n>` must be a positive integer in the range $[1 \dots 10,000]$.
- **Seat Mutual Exclusion (CS2)**: At most one session per seat may hold foreground status. Activating a session automatically demotes any prior foreground session on that seat to background.
- **State Monotonicity (CS1)**: Terminated sessions are immutable; further actions will be rejected.
- **Store Capacity (CS3, SB5)**: Maximum 32 active sessions per user, and 1,024 total tracked sessions in a single store.
- **Audit Non-Repudiation (ADR-0035 §F-2)**: All invocations (successful or failed) emit SHA-256 hash-chained audit events to `$AIOSH_HOME/audit.db`.

### Evidence References
- Research: [T-01421-cli-surface-research.md](../docs/tasks/evidence/T-01421-cli-surface-research.md)
- Specification: [T-01422-cli-surface-specification.md](../docs/tasks/evidence/T-01422-cli-surface-specification.md)
- Scaffold: [T-01423-cli-surface-scaffold.md](../docs/tasks/evidence/T-01423-cli-surface-scaffold.md)
- Implementation: [T-01424-cli-surface-implementation.md](../docs/tasks/evidence/T-01424-cli-surface-implementation.md)
- Unit Tests: [T-01425-cli-surface-unit-test.md](../docs/tasks/evidence/T-01425-cli-surface-unit-test.md)
- Integration: [T-01426-cli-surface-integration.md](../docs/tasks/evidence/T-01426-cli-surface-integration.md)
- Security Review: [T-01427-cli-surface-security-review.md](../docs/tasks/evidence/T-01427-cli-surface-security-review.md)
- Hardening: [T-01428-cli-surface-hardening.md](../docs/tasks/evidence/T-01428-cli-surface-hardening.md)

## Audit ring

SQLite WAL database at `$AIOSH_HOME/audit.db` (default `~/.aios/audit.db`).

Schema:
```sql
CREATE TABLE audit_ring (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts TEXT NOT NULL,
  actor TEXT NOT NULL,
  actor_id TEXT NOT NULL,
  tool TEXT NOT NULL,
  command TEXT NOT NULL,
  args_json TEXT NOT NULL,
  target TEXT,
  outcome TEXT NOT NULL,
  outcome_detail TEXT,
  constitution_rev TEXT,
  grant_token TEXT,
  c1 INTEGER NOT NULL DEFAULT 0,
  c2 INTEGER NOT NULL DEFAULT 0,
  c3 INTEGER NOT NULL DEFAULT 0,
  c4 INTEGER NOT NULL DEFAULT 0,
  prev_hash TEXT NOT NULL,
  hash TEXT NOT NULL UNIQUE
);
```

**Hash chain:** `row.hash = SHA-256(row.prev_hash || canonical_json(row_without_hash_field))`.
First row's prev_hash is `00...00` (genesis).

## PEP grant tokens

A grant encodes a tool/network/path scope. Tokens persist in
`pep_grants` table inside the same DB.

Create:
```bash
aiosh grant create \
  --to agent:pentest-bot \
  --tools 'pentest.nmap,network.*' \
  --networks '10.0.0.0/8,127.0.0.0/8' \
  --allow '/tmp/pentest' \
  --deny  '/etc' \
  --ttl 3600
```

Output:
```json
{
  "ok": true,
  "subcommand": "grant create",
  "outcome": "ok",
  "data": {
    "grant_id": "gr_xxxxxxxxxxxxxxxx",
    ...
  }
}
```

## Tests

```bash
npm test    # runs tests/smoke.sh
```
