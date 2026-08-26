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
