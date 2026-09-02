# T-00987 — Agent Handoff Protocol / Observability: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-1: Sensitive Payload Leakage via Stats / Reports
- **Threat**: Aggregated reporting exposes private payload data across tenant boundaries.
- **Mitigation**: Summary counts are purely numerical; individual record payloads are bounded and protected by access controls.

### AS-2: In-Memory Report Allocation Exhaustion
- **Threat**: Report generation cloning millions of records into a single huge vector.
- **Mitigation**: Store size caps (`MAX_STORE_BYTES`) strictly limit the total memory size of any `HandoffReport`.

### AS-3: Non-Idempotent Stats Side-Effects
- **Threat**: Reading stats causes state modifications or audit ring pollution.
- **Mitigation**: `to_report()` is an immutable borrow (`&self`) with zero audit row emission.
