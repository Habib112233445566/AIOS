# T-00848 — Regression Triage / MCP/API: Hardening

## 1. Hardening Deliverables
- **Request Framing Bounds**: `read_line_capped` enforces 1 MiB framing cap per MCP request.
- **Envelope Consistency**: Failures produce explicit structured envelopes (`ok: false`, `error: "..."`) and never panic.
- **Resource Discipline**: Connections and file handles are scoped and released promptly.
