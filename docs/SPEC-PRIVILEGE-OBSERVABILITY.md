# Specification: Privilege Escalation Prevention Observability (SPEC-PRIVILEGE-OBSERVABILITY)

- **Status**: APPROVED
- **Date**: 2026-10-02
- **Scope**: Observability, telemetry aggregation, and health reporting for Privilege Escalation Prevention.

## 1. Data Contract: `PrivilegeObservabilityReport`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrivilegeObservabilityReport {
    pub generated_at_utc: String,
    pub total_registered_actors: usize,
    pub active_contexts_count: usize,
    pub actors_by_tier: HashMap<String, usize>,
    pub total_transitions_recorded: usize,
    pub transitions_by_outcome: HashMap<String, usize>,
    pub policy_mode: String,
    pub is_healthy: bool,
}
```

## 2. Invariants
- `PRIVESCOBS1`: Generated timestamps must follow RFC 3339 format and cannot be blank.
- `PRIVESCOBS2`: Textual telemetry inputs must pass through `sanitize_telemetry_text()`, stripping control characters, bell, backspace, ANSI escapes, and truncating at `MAX_TELEMETRY_TEXT_LEN` (256 bytes).
- `PRIVESCOBS3`: Cardinality of distribution maps (`actors_by_tier`, `transitions_by_outcome`) is capped at `MAX_OUTCOME_DISTRIBUTION_ENTRIES` (128).
- `PRIVESCOBS4`: Consistency check: `active_contexts_count` cannot exceed `total_registered_actors`.
- `PRIVESCOBS5`: Query window bounded to `MAX_AUDIT_LOG_TAIL_ITEMS` (1,000 items) to prevent ring query degradation.
- `PRIVESCOBS6`: System health evaluates true when active contexts are within capacity limits (`<= max_contexts`) and validation succeeds.

## 3. Operator & Agent Invocation

### 3.1 CLI Commands
Generate human-readable observability metrics:
```bash
aiosh privilege stats
```

Generate machine-readable JSON metrics:
```bash
aiosh privilege stats --json
```

Sample JSON output:
```json
{
  "active_contexts_count": 2,
  "actors_by_tier": {
    "guest": 1,
    "user": 1
  },
  "generated_at_utc": "2026-10-02T06:14:15.123456Z",
  "is_healthy": true,
  "policy_mode": "enforcing",
  "total_registered_actors": 2,
  "total_transitions_recorded": 5,
  "transitions_by_outcome": {
    "allowed": 4,
    "denied": 1
  }
}
```

### 3.2 MCP Tool Calls
Agents can query observability using the MCP tools:
- `aios.privilege.stats`
- `aios.privilege.observability`

Example MCP Call:
```json
{
  "name": "aios.privilege.stats",
  "arguments": {}
}
```

## 4. Constraints and Limitations
1. **Audit Tail Window**: Tail queries are bounded to `MAX_AUDIT_LOG_TAIL_ITEMS = 1000`. Historical data older than the 1,000 most recent transitions must be queried via long-term data warehouse exports.
2. **Cardinality Caps**: Telemetry buckets are clamped to 128 categories to prevent memory exhaustion under high cardinalities.
3. **In-Memory Volatility**: Live actor contexts reflect current process lifecycle state; persistent transitions are drawn from the AuditRing database.

## 5. Sub-Epic Task Evidence
- [T-02571 Observability Research](tasks/evidence/T-02571-observability-research.md)
- [T-02572 Observability Specification](tasks/evidence/T-02572-observability-specification.md)
- [T-02573 Observability Scaffold](tasks/evidence/T-02573-observability-scaffold.md)
- [T-02574 Observability Implementation](tasks/evidence/T-02574-observability-implementation.md)
- [T-02575 Observability Unit Tests](tasks/evidence/T-02575-observability-unit-test.md)
- [T-02576 Observability Integration](tasks/evidence/T-02576-observability-integration.md)
- [T-02577 Observability Security Review](tasks/evidence/T-02577-observability-security-review.md)
- [T-02578 Observability Hardening](tasks/evidence/T-02578-observability-hardening.md)
- [T-02579 Observability Documentation](tasks/evidence/T-02579-observability-documentation.md)
- [T-02580 Observability Verification](tasks/evidence/T-02580-observability-verification-evidenc.md)
