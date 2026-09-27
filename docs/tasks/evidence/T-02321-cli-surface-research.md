# T-02321: Audit Chain Extensions CLI Surface Research

## Objective
Establish facts, constraints, and prior art for the CLI surface of Audit Chain Extensions in `aiosh-cli`.

## Facts vs Assumptions

### Facts
1. **Existing CLI Structure**: `aiosh-cli/src/main.rs` defines `cmd_audit` matching subcommands `tail`, `verify`, `rotate`, `segments`, and `seen`.
2. **Standard Output Envelope**: All `aiosh` CLI commands emit JSON to stdout matching `{"ok": bool, "subcommand": string, "outcome": string, "audit_id": i64, "data": ...}`.
3. **Audit Row Emission Invariant**: Consequential CLI commands write an audit row recording the command, actor, tool, and outcome via `emit(&mut ctx, ...)`.
4. **Underlying Service**: `AuditChainService` in `aiosh-core` already provides the necessary backend operations: `query_events`, `trace_ancestry`, `verify_event_signature`, and `get_row_by_hash`.

### Assumptions
1. Operators and security auditing agents require CLI subcommands to query provenance by session/trace, trace causal lineage graphs, and verify cryptographic signatures.
2. The CLI should support both flag parsing (e.g. `--session`, `--trace`, `--depth`, `--limit`) and positional arguments where intuitive.

## Prior Art & Command Line Conventions
- `git log --graph`, `git rev-list --ancestry-path`: Prior art for traversing commit DAGs and displaying ancestry chains.
- `openssl dgst -verify`: Prior art for cryptographic signature verification from CLI.
- POSIX exit codes: `0` for success, `2` for usage refusal / invalid arguments, `1` for execution failure.

## Decisions Needed Prior to Implementation
1. **Subcommand Names**:
   - `aiosh audit query`: multi-criteria search (`--session`, `--trace`, `--actor`, `--tool`, `--parent`, `--limit`).
   - `aiosh audit ancestry <hash> [--depth N]`: causal DAG traversal upwards.
   - `aiosh audit sign-verify <hash>`: signature verification for a specific event hash.
   - `aiosh audit inspect <hash>`: detailed JSON inspection of an extended audit row.
2. **Context Management**: Subcommands will instantiate `AuditChainService::new(ctx.ring)` to execute logic and output the standard JSON envelope.
