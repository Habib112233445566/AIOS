# T-00748 — Secrets & Access Hygiene / MCP/API surface: Hardening

## 1. Hardening Deliverables
- **Input Sanitization & Schema Bounds**:
  - `additionalProperties: false` enforced in MCP JSON-RPC schemas preventing unexpected argument pollution.
  - Defaults `repo_path` to `.` when omitted.
  - Scans bounded by `max_bytes` (16 MiB default) with option to override safely.
- **Auditable Error Envelopes**:
  - All execution exceptions bubble up into `{ "ok": false, "error": "<detailed_message>" }` envelopes.
  - Failures generate an honest audit row via `dispatch::recorded_call` (`outcome: "error"`).
- **Resource Management**:
  - Rust RAII handles clean tear-down of file descriptors and buffered readers.
  - Zero persistent state leaks across stdio JSON-RPC sessions.
