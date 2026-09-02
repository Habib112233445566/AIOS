# T-00962 — Agent Handoff Protocol / Automated Tests: Specification

## 1. Test Matrix for Criterion H6

| Test Case | Description | Expected Outcome |
|---|---|---|
| TC-01 | Full lifecycle: Initiate $\to$ Accept $\to$ Complete | Status sequence `Pending` $\to$ `Accepted` $\to$ `Completed`. |
| TC-02 | Rejection lifecycle: Initiate $\to$ Reject | Status sequence `Pending` $\to$ `Rejected`. |
| TC-03 | Cancellation lifecycle: Initiate $\to$ Cancel | Status sequence `Pending` $\to$ `Cancelled`. |
| TC-04 | Terminal state immutability: Cancel/Accept after Completed/Rejected | Returns explicit `Err` on illegal transition. |
| TC-05 | In-flight deduplication: Double initiate same parameters | Returns identical existing `HND-*` record without duplicating. |
| TC-06 | Size cap boundary: Store exceeding `max_store_bytes` | Returns explicit size limit `Err`. |
| TC-07 | Corrupted file recovery: Malformed JSON store loading | Automatically initializes fresh store with warning. |

## 2. Invariants Asserted
- Standardized exit code 0 for all green suite runs.
- Deterministic SHA-256 signatures for identical handoff payloads.
