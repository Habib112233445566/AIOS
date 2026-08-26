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

¹ The pentest tool writes one audit row through the same `_mcp_dispatch`
   helper that all the others use; in Sprint 0 the row is written
   synchronously after the tool returns.

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
