# T-00921 — Agent Handoff Protocol / Core Service: Research

## 1. Prior Art & Architecture
- **Service Store Design (`aiosh-core::handoff_service`)**:
  - `HandoffStore` acts as the in-memory state manager with atomic JSON persistence and SQLite WAL audit logging.
  - State machine transitions:
    - `Pending` $\to$ `Accepted` (Receiver acknowledges receipt and claims task execution).
    - `Pending` $\to$ `Rejected` (Receiver declines capability or workload constraint).
    - `Accepted` $\to$ `Completed` (Receiver finishes task and returns resolution notes).
    - `Pending` | `Accepted` $\to$ `Cancelled` (Sender or supervisor revokes handoff).
  - Persistence: Atomic rename (`.tmp` write followed by `fs::rename`) with bounded file sizes.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Atomic File Persistence | Fact | Store files must use `.tmp` atomic write to prevent partial file corruption on power loss. |
| Non-Replayable IDs | Fact | Lookups by deterministic SHA-256 signature allow deduplicating identical in-flight handoff attempts. |
| Audit Compliance | Fact | Every state transition logs immutable event records. |

## 3. Decisions & Actions
- Implement `HandoffStore` in `code/aiosh-rust/aiosh-core/src/handoff_service.rs`.
- Add criterion `H2` to `tools/test_handoff_suites.py`.
