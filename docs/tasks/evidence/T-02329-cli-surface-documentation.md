# T-02329: Audit Chain Extensions CLI Surface Documentation

## Overview
The `aiosh audit` CLI commands allow operators, developers, and autonomous agents to query, inspect, traverse, and verify extended audit events from the terminal.

## CLI Command Reference

### 1. `aiosh audit query`
Filter audit records by multi-attribute criteria.
```bash
# Query events by session
aiosh audit query --session session-abc-123 --limit 20

# Query events by distributed trace ID
aiosh audit query --trace trace-999

# Query by actor and tool
aiosh audit query --actor "agent:planner" --tool "aios.task.execute"

# Query direct children of a parent audit event
aiosh audit query --parent 113a2172513295bb517663045f541006c9638ad9fc012434babbd46dc10c8df8
```

### 2. `aiosh audit inspect <hash>`
Fetch the full record and metadata for a specific audit row.
```bash
aiosh audit inspect 113a2172513295bb517663045f541006c9638ad9fc012434babbd46dc10c8df8
```

### 3. `aiosh audit ancestry <hash> [--depth <n>]`
Trace the causal DAG upstream from a given event to root triggers.
```bash
aiosh audit ancestry 113a2172513295bb517663045f541006c9638ad9fc012434babbd46dc10c8df8 --depth 10
```

### 4. `aiosh audit sign-verify <hash>`
Verify the asymmetric digital signature associated with an event.
```bash
aiosh audit sign-verify 113a2172513295bb517663045f541006c9638ad9fc012434babbd46dc10c8df8
```

## JSON Output Structure
All commands return a standardized JSON response:
```json
{
  "ok": true,
  "subcommand": "audit query",
  "outcome": "ok",
  "audit_id": 19001,
  "data": {
    "count": 1,
    "rows": [ ... ]
  }
}
```

## Constraints & Limitations
- Maximum pagination limit is 1,000 rows.
- Maximum ancestry traversal depth is 64 hops.
- Invocations emit an audit row to the SQLite WAL database.

## Related Evidence Links
- CLI Research: [T-02321](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02321-cli-surface-research.md)
- CLI Spec: [T-02322](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02322-cli-surface-specification.md)
- CLI Scaffold: [T-02323](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02323-cli-surface-scaffold.md)
- CLI Implementation: [T-02324](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02324-cli-surface-implementation.md)
- CLI Unit Test: [T-02325](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02325-cli-surface-unit-test.md)
- CLI Integration: [T-02326](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02326-cli-surface-integration.md)
- CLI Security Review: [T-02327](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02327-cli-surface-security-review.md)
- CLI Hardening: [T-02328](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02328-cli-surface-hardening.md)
