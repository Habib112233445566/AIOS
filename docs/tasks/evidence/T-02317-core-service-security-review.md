# T-02317: Audit Chain Extensions Core Service Security Review

## Threat Model & Security Evaluation
This security review evaluates the service layer (`AuditChainService`) for Audit Chain Extensions.

## Abuse Scenarios & Mitigations

### 1. AS-SVC-01: SQL Injection via Filter Parameters
- **Threat**: Malicious actors inject SQL statements into `actor`, `tool`, `session_id`, `trace_id`, or `parent_hash` filters.
- **Mitigation**: All query parameters in `AuditChainService::query_events` are bound via SQLite parameterized statements (`params_vec`), preventing SQL injection.

### 2. AS-SVC-02: Algorithmic Complexity Attack via Causal Cycles
- **Threat**: Attackers forge cyclic parent-child audit relationships (e.g. Event A -> Event B -> Event A) to cause stack overflow or infinite loops during ancestry tracing.
- **Mitigation**: `trace_ancestry` tracks visited node hashes in `HashSet<String>` and terminates branches upon encountering visited nodes. Additionally, traversal depth is hard-capped at `MAX_LINEAGE_DEPTH` (64).

### 3. AS-SVC-03: Bypass of Validation via Direct Service Calls
- **Threat**: Callers invoke `record_event` with malformed or oversized payloads bypassing model validation.
- **Mitigation**: `record_event` mandates `input.validate()?` before disk insertion, validating session IDs, causal link caps, signature attributes, and extension byte sizes.

### 4. AS-SVC-04: Silent Failure or Inconsistent Chain Mutation
- **Threat**: Partial failures during write operations leaving the chain in an unverified or detached state.
- **Mitigation**: SQLite transactions and WAL mode guarantee atomic insertion of extended rows, updating `head_hash` synchronously.

## Security Verdict
The service architecture is robust against injection, resource exhaustion, and bypass attacks. Ready for hardening.
