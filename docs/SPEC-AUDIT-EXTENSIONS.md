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

---

## 6. Audit Chain Security Policy Subsystem (Sub-Epic 7)

### 6.1 Policy Modes & Rules
The security policy engine governs event ingestion, causal parent linking, and provenance integrity:
- **`enforcing`**: Fail-closed rejection of policy violations with structured error codes (`AUDITPOL_ERR_*`).
- **`permissive`**: Permits operations with a structured audit warning (`PermitWithWarning`).
- **`disabled`**: Bypasses security policy evaluation.

### 6.2 Key Constraints
- **Anonymous Provenance**: Prohibits empty or `"anonymous"` actor and tool strings when `disallow_anonymous` is true.
- **Sensitive Tool Signatures**: Tools matching `signature_required_prefixes` (`["kernel:", "sec:", "admin:", "pep:"]`) strictly require valid Ed25519 digital signatures.
- **Causal Fan-out Bounding**: Maximum allowed causal links per event bounded to $\le 32$ (hard cap: 64).
- **Temporal Validity**: Clock skew restricted to $\le 300\text{s}$ future skew; optional active window (`valid_from_epoch_secs`, `valid_until_epoch_secs`).

### 6.3 Example Commands
```bash
# Inspect active audit chain security policy via CLI
aiosh audit policy --json

# Inspect custom policy file
aiosh audit policy --path config/custom_audit_policy.json

# Call via MCP JSON-RPC
{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "aios.audit.policy", "arguments": {}}}
```

---

## 7. Audit Chain Observability Subsystem (Sub-Epic 8)

### 7.1 Overview & Metrics
The observability subsystem (`AuditChainObservabilityReport`) provides point-in-time telemetry snapshot generation for audit chain storage, cryptographic posture, and causal graph density:
- **Event Volume**: Total rows and extended structured rows count.
- **Lineage Density**: Total causal parent-child link edges registered across the chain.
- **Cryptographic Coverage**: Total verified Ed25519 digitally signed event records.
- **Cardinality Metrics**: Distinct actor, tool, session, and distributed trace counts.
- **Outcome Distribution**: Bounded frequency histogram of event outcomes (`success`, `denied`, `error`, etc.).
- **Physical Health**: Database file size in bytes, policy enforcement mode, and cryptographic SHA-256 chain integrity verification.

### 7.2 Safety & Hardening Constraints
- **Text Sanitization**: Control characters and ANSI escape codes stripped from string telemetry; capped to 256 characters (`MAX_TELEMETRY_TEXT_LEN`).
- **Cardinality Caps**: Outcome distribution bounded to top 128 classes (`MAX_OUTCOME_DISTRIBUTION_ENTRIES = 128`).
- **Memory Bounding**: In-memory deduplication sets for sessions/traces capped at 100,000 items (`MAX_TRACKED_CARDINALITY_ITEMS`).
- **Audit-on-Read**: Observability queries executed via MCP tools are intercepted and logged through PEP and audit rings (`dispatch::recorded_call`).

### 7.3 Usage & Examples
```bash
# Print formatted observability summary to terminal
aiosh audit stats

# Retrieve structured JSON report for ingestion into monitoring/SIEM
aiosh audit stats --json

# Query observability telemetry via MCP tool call
{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "aios.audit.stats", "arguments": {}}}
```

### 7.4 Evidence & Cross-References
- Research: [T-02371 Evidence](docs/tasks/evidence/T-02371-observability-research.md)
- Specification: [T-02372 Evidence](docs/tasks/evidence/T-02372-observability-specification.md)
- Scaffold: [T-02373 Evidence](docs/tasks/evidence/T-02373-observability-scaffold.md)
- Implementation: [T-02374 Evidence](docs/tasks/evidence/T-02374-observability-implementation.md)
- Unit Tests: [T-02375 Evidence](docs/tasks/evidence/T-02375-observability-unit-test.md)
- Integration: [T-02376 Evidence](docs/tasks/evidence/T-02376-observability-integration.md)
- Security Review: [T-02377 Evidence](docs/tasks/evidence/T-02377-observability-security-review.md)
- Hardening: [T-02378 Evidence](docs/tasks/evidence/T-02378-observability-hardening.md)
- Verification: [T-02380 Evidence](docs/tasks/evidence/T-02380-observability-verification-evidenc.md)

---

## 8. Audit Chain Documentation Subsystem (Sub-Epic 9)

### 8.1 Overview & Canonical Index
The documentation subsystem (`AuditChainDocIndex`) provides an in-memory, zero-dependency, self-contained reference repository and lexical keyword search engine for all components of Audit Chain Extensions:
- **Canonical Topics**:
  - `audit-arch`: Architecture, SQLite ring tables, SHA-256 rolling chain.
  - `audit-lineage`: Causal parent links, DAG topological traversal, cycle immunity.
  - `audit-crypto`: Ed25519 digital signatures and tamper detection.
  - `audit-policy`: Ingestion gating rules, modes (`enforcing`, `permissive`, `disabled`), actor bounds.
  - `audit-observability`: Point-in-time telemetry reports, histograms, cardinality stats.
  - `audit-recovery`: Integrity checking, verification, and repair workflows.
  - `audit-reference`: Comprehensive CLI and MCP tool reference.

### 8.2 Safety & Hardening Bounds
- **Query Bounds**: Search queries constrained between 1 and 128 characters (`MAX_AUDIT_DOC_QUERY_LEN`).
- **Token Limits**: Maximum 16 search tokens evaluated per query (`MAX_DOC_SEARCH_TOKENS`).
- **Control Character Filtering**: Control characters stripped via `sanitize_doc_query` to prevent terminal injection.
- **Topic ID Validation**: Topic slug restricted to alphanumeric and hyphen/underscore characters (`[a-zA-Z0-9_-]`).
- **Result Limits**: Search results capped to top 10 matches (`MAX_AUDIT_DOC_SEARCH_RESULTS`); snippets capped to 200 chars.

### 8.3 Invocations & Usage
```bash
# List all documentation topics
aiosh audit doc

# Display formatted Markdown for a specific topic
aiosh audit doc audit-crypto

# Search topics by keyword
aiosh audit doc lineage

# Query via MCP tool call
{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "aios.audit.doc", "arguments": {"query": "signature"}}}
```

### 8.4 Evidence & Cross-References
- Research: [T-02381 Evidence](docs/tasks/evidence/T-02381-documentation-research.md)
- Specification: [T-02382 Evidence](docs/tasks/evidence/T-02382-documentation-specification.md)
- Scaffold: [T-02383 Evidence](docs/tasks/evidence/T-02383-documentation-scaffold.md)
- Implementation: [T-02384 Evidence](docs/tasks/evidence/T-02384-documentation-implementation.md)
- Unit Tests: [T-02385 Evidence](docs/tasks/evidence/T-02385-documentation-unit-test.md)
- Integration: [T-02386 Evidence](docs/tasks/evidence/T-02386-documentation-integration.md)
- Security Review: [T-02387 Evidence](docs/tasks/evidence/T-02387-documentation-security-review.md)
- Hardening: [T-02388 Evidence](docs/tasks/evidence/T-02388-documentation-hardening.md)
- Verification: [T-02390 Evidence](docs/tasks/evidence/T-02390-documentation-verification-evidenc.md)

---

## 9. Audit Chain Recovery & Invariant Validation Subsystem (Sub-Epic 10)

### 9.1 Overview & Invariant Rules
The recovery and validation subsystem (`AuditChainRecoveryManager`) provides two-tier structural verification and non-destructive forward repair:
- **Structural Invariants**:
  - `HashDiscontinuity`: Sequential `prev_hash` links verified from `GENESIS_HASH`.
  - `InvalidJson`: Validates `provenance_json`, `causal_links_json`, `signature_json`, `extensions_json`.
  - `CausalCycleDetected`: Enforces DAG acyclicity by detecting self-referential or circular causal links.
  - `SignatureMismatch`: Detects corrupted digital signatures on events.
- **Non-Destructive Forward Repair**:
  - Pre-flight atomic database snapshots: `<db_path>.backup.<timestamp>` (or custom `--backup-dir`).
  - Strict append-only forward anchoring: Historical rows are never deleted or rewritten; instead, a cryptographically sealed `repair` anchor event (`outcome = "repaired"`, `tool = "audit.recover"`) is appended forward.

### 9.2 Safety & Hardening Bounds
- **Path Traversal Shield**: `validate_backup_dir` rejects `..` sequences, control characters, and enforces `MAX_PATH_LEN = 1024`.
- **Diagnostic Capping**: Maximum diagnostic issues capped at 1,000 (`MAX_VALIDATION_ISSUES`).
- **Audit-on-Operation**: All repair and validation actions via MCP (`aios.audit.validate`, `aios.audit.repair`) are audited via `dispatch::recorded_call`.

### 9.3 Invocations & Usage
```bash
# Validate audit chain invariants
aiosh audit validate
aiosh audit validate --json

# Execute forward recovery with automatic snapshot
aiosh audit repair
aiosh audit repair --backup-dir /var/backups/audit --json

# Query via MCP tool call
{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "aios.audit.validate", "arguments": {}}}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "aios.audit.repair", "arguments": {}}}
```

### 9.4 Evidence & Cross-References
- Research: [T-02391 Evidence](docs/tasks/evidence/T-02391-recovery-validation-research.md)
- Specification: [T-02392 Evidence](docs/tasks/evidence/T-02392-recovery-validation-specification.md)
- Scaffold: [T-02393 Evidence](docs/tasks/evidence/T-02393-recovery-validation-scaffold.md)
- Implementation: [T-02394 Evidence](docs/tasks/evidence/T-02394-recovery-validation-implementation.md)
- Unit Tests: [T-02395 Evidence](docs/tasks/evidence/T-02395-recovery-validation-unit-test.md)
- Integration: [T-02396 Evidence](docs/tasks/evidence/T-02396-recovery-validation-integration.md)
- Security Review: [T-02397 Evidence](docs/tasks/evidence/T-02397-recovery-validation-security-review.md)
- Hardening: [T-02398 Evidence](docs/tasks/evidence/T-02398-recovery-validation-hardening.md)
- Verification: [T-02400 Evidence](docs/tasks/evidence/T-02400-recovery-validation-verification-evidenc.md)





