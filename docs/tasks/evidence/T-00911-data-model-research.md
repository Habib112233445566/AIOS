# T-00911 — Agent Handoff Protocol / Data Model: Research

## 1. Prior Art & Architecture
- **Agent Handoff Protocol Overview**:
  - Provides formal, auditable, and authenticated transfer of task execution context and authority between agents or human supervisors.
  - Prevents context truncation, state drift, or unauthorized capability delegation during multi-agent workflows.
- **Core Data Types**:
  - `HandoffStatus`: `Pending`, `Accepted`, `Rejected`, `Completed`, `Cancelled`, `Expired`.
  - `HandoffPriority`: `Low`, `Normal`, `High`, `Urgent`.
  - `HandoffRecord`: Core struct capturing sender, recipient, task association, state payload, cryptographic signature, timestamps, and expiration limits.
  - `HandoffReport`: Quantitative summary of queue health, active handoffs, and completion rates.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Module Location | Fact | Canonical Rust userspace library `code/aiosh-rust/aiosh-core/src/handoff.rs`. |
| Audit Trail | Fact | All handoff creations, acceptances, and completions must write immutable SQLite WAL audit rows. |
| Non-Replayability | Fact | Each handoff carries a unique deterministic signature and optional expiration window to prevent replay attacks. |

## 3. Decisions & Actions
- Implement `HandoffStatus`, `HandoffPriority`, `HandoffRecord`, and `HandoffReport` in `aiosh-core::handoff`.
- Implement failure-resistant validation engine and SHA-256 fingerprinting.
