# T-02322: Audit Chain Extensions CLI Surface Specification

## Subcommands Overview
Audit Chain Extensions introduces four subcommands under `aiosh audit`:

### 1. `aiosh audit query`
Multi-field query filtering over the append-only audit trail.
- **Flags**:
  - `--session <id>`: Filter by session identifier.
  - `--trace <id>`: Filter by distributed trace ID.
  - `--actor <name>`: Filter by actor string.
  - `--tool <name>`: Filter by tool string.
  - `--parent <hash>`: Filter by direct causal parent event hash.
  - `--limit <n>`: Pagination limit (1..1000, default 50).
- **Exit Codes**: `0` on success, `2` on invalid argument syntax.

### 2. `aiosh audit ancestry <hash> [--depth <n>]`
Traverses the DAG of causal parent links backwards to reconstruct execution lineage.
- **Arguments**:
  - `<hash>`: 64-character SHA-256 event hash (required).
  - `--depth <n>`: Max traversal depth (1..64, default 16).
- **Output Data**: `{"target_hash": string, "ancestors": [{"row": ..., "depth": int, "relationship": string}], "max_depth_reached": bool}`.
- **Exit Codes**: `0` on success, `2` on missing or invalid hash.

### 3. `aiosh audit sign-verify <hash>`
Verifies the cryptographic digital signature attached to an audit event.
- **Arguments**:
  - `<hash>`: SHA-256 event hash (required).
- **Output Data**: `{"target_hash": string, "has_signature": bool, "algorithm": string | null, "is_valid": bool, "error": string | null}`.
- **Exit Codes**: `0` on success, `1` if row not found, `2` on argument syntax error.

### 4. `aiosh audit inspect <hash>`
Inspects an extended audit record including base fields, provenance, causal links, signatures, and extensions.
- **Arguments**:
  - `<hash>`: SHA-256 event hash (required).
- **Exit Codes**: `0` on success, `1` if row not found, `2` on argument syntax error.

## Standard JSON Output Envelope
Every command complies with ADR-0035 standard envelope formatting:
```json
{
  "ok": true,
  "subcommand": "audit query",
  "outcome": "ok",
  "audit_id": 42,
  "data": { ... }
}
```
Consequential CLI actions write an audit row to the SQLite WAL table.
