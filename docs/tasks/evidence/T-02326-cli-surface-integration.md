# T-02326: Audit Chain Extensions CLI Surface Integration

## Overview
This task documents the integration and discoverability of the Audit Chain Extensions subcommands in `aiosh-cli`.

## CLI Surface Registration & Help Verification

```text
aiosh.exe --help
aiosh — AIOS shell CLI (Rust)

Usage: aiosh <status|run|agent|audit|grant|pentest|classify|task|ci|release|backup|toolchain|doc|evidence|repo|secrets|triage|handoff|distro|image|package|service|session|layout|mod|hw|net|update|capability|pep> ...

  aiosh audit <tail|verify|rotate|segments|seen|query|ancestry|sign-verify|inspect>  Audit ring & chain extensions control
```

## Integration Highlights
1. **Registered Subcommands**:
   - `aiosh audit query`: Multi-field provenance filtering across session, trace, actor, tool, and parent links.
   - `aiosh audit inspect <hash>`: Extended row inspection.
   - `aiosh audit ancestry <hash> [--depth <n>]`: Directed acyclic graph traversal.
   - `aiosh audit sign-verify <hash>`: Public key digital signature verification.
2. **Context & Persistence Flow**:
   - Subcommands construct `AuditChainService` around the live `ctx.ring`, execute business logic, consume into `ctx.ring = service.into_ring()`, emit consequential audit events via `emit(&mut ctx, ...)`, and return the standard JSON output envelope.
3. **Cross-Substrate Durability**:
   - Full compatibility with the persistent SQLite WAL database, ensuring seamless operations between CLI and MCP surfaces.
