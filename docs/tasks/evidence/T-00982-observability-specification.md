# T-00982 — Agent Handoff Protocol / Observability: Specification

## 1. Specification of HandoffReport

```json
{
  "timestamp_utc": "2026-08-31T17:15:00Z",
  "total_handoffs": 10,
  "active_handoffs": 3,
  "completed_handoffs": 7,
  "records": [...]
}
```

## 2. Invariants & Calculation Rules
- `total_handoffs == records.len() as u32`.
- `active_handoffs + completed_handoffs == total_handoffs`.
- Non-terminal statuses (`Pending`, `Accepted`) map to `active_handoffs`.
- Terminal statuses (`Completed`, `Rejected`, `Cancelled`, `Expired`) map to `completed_handoffs`.
- Validated via `validate_handoff_report(&report)`.
