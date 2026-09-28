# SPEC — Audit Chain Extensions & Configuration (Epic 4)

**Status:** IMPLEMENTED (2026-09-28)  
**Research:** `docs/tasks/evidence/T-02341-configuration-research.md`  
**Specification:** `docs/tasks/evidence/T-02342-configuration-specification.md`  
**Constitution:** P-2 (immutability), O-2 (one row per action), O-4 (hash chain & truncation detection), C-4 (auditability gate)  

Audit Chain Extensions expand the core SQLite audit ring with cryptographic non-repudiation, causal DAG provenance, rich query filtering, and a formalized configuration engine (`AuditChainConfig`).

---

## 1. Configuration Schema (`AuditChainConfig`)

```json
{
  "version": "1.0.0",
  "db_path": ".aios/audit.db",
  "max_query_limit": 50,
  "default_lineage_depth": 16,
  "max_causal_links": 16,
  "max_extensions_bytes": 65536,
  "verify_signatures_on_read": true,
  "strict_provenance": false
}
```

### Parameter Reference & Bounds

| Parameter | Type | Default | Bounds | Description |
| :--- | :---: | :---: | :---: | :--- |
| `version` | string | `"1.0.0"` | Must match `1.*` | Configuration schema version |
| `db_path` | path | `".aios/audit.db"` | Valid path | SQLite database path or `:memory:` |
| `max_query_limit` | integer | `50` | `1..1000` | Default query limit cap |
| `default_lineage_depth` | integer | `16` | `1..64` | Traversal depth limit for causal DAGs |
| `max_causal_links` | integer | `16` | `1..32` | Maximum causal parent links per event |
| `max_extensions_bytes` | integer | `65536` | `1024..1048576` | Maximum size of JSON extensions payload |
| `verify_signatures_on_read` | boolean | `true` | `true/false` | Validate digital signatures upon inspection |
| `strict_provenance` | boolean | `false` | `true/false` | Enforce presence of trace and session IDs |

---

## 2. Invocation Examples

### 2.1 CLI Interface
```bash
# View active audit chain configuration in human-readable table
aiosh audit config

# Export active configuration as JSON
aiosh audit config --json

# Query audit events with custom limit
aiosh audit query --limit 20 --actor operator

# Inspect event by hash
aiosh audit inspect 7e2f1a...

# Trace causal lineage upwards
aiosh audit ancestry 7e2f1a... --depth 8

# Verify digital signature
aiosh audit sign-verify 7e2f1a...
```

### 2.2 MCP Interface (JSON-RPC 2.0)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.audit.config",
    "arguments": {}
  }
}
```

---

## 3. Environment Variables
- `AIOS_AUDIT_CONFIG_PATH`: Explicit path to a JSON configuration file.
- `AIOS_AUDIT_DB_PATH`: Override SQLite database location.
- `AIOS_AUDIT_MAX_QUERY_LIMIT`: Override query pagination upper bound.
- `AIOS_AUDIT_LINEAGE_DEPTH`: Override default lineage traversal depth.

---

## 4. Constraints & Known Limitations
1. **Config File Size Limit**: Configuration files are strictly capped at `64 KiB` (`MAX_CONFIG_FILE_BYTES`). Files exceeding this size fail with `AUDITCONF_ERR_BOUNDS`.
2. **Hard Traversal Limits**: Causal ancestry depth cannot exceed 64 levels (`MAX_LINEAGE_DEPTH_BOUND = 64`) to eliminate recursion stack overflow risks.
3. **Hard Query Bounds**: Query limits cannot exceed 1,000 rows (`MAX_PERMISSIBLE_QUERY_LIMIT = 1000`).

---

## 5. Automated Test Suites & Verification

### 5.1 Formal Automated Test Vectors (`test_audit_chain_automated.rs`)
| Vector | Description | Invariant Tested |
| :--- | :--- | :--- |
| `AUTOAUDIT1` | High-Volume Scale (100+ events) | Continuous SHA-256 chain integrity under rapid sequential writes |
| `AUTOAUDIT2` | Deep Sequential DAG Lineage (20 levels) | Exact topological ancestry resolution without truncation |
| `AUTOAUDIT3` | Branching Diamond Lineage ($A \to B, C \to D$) | Multi-parent DAG deduplication and visited set tracking |
| `AUTOAUDIT4` | Cycle Detection & Loop Immunity | Termination in $O(V)$ time with loop detection |
| `AUTOAUDIT5` | Cryptographic Signatures | Ed25519 signature validation and algorithm reporting |
| `AUTOAUDIT6` | Multi-Column Query Filtering | Parameter combination filtering and limit clamping |
| `AUTOAUDIT7` | Multi-Threaded Concurrency Safety | Safe concurrent reads (4 threads) & writes (2 threads) |
| `AUTOAUDIT8` | Legacy Row Parity | Backward compatibility with unextended audit rows |

### 5.2 Test Invocation Commands
```bash
# Run Rust automated test suite
cargo test --test test_audit_chain_automated

# Run Python MCP cross-surface automated smoke tests
python code/aiosh-mcp/tests/test_audit_chain_automated_smoke.py
```

