# T-00936 — Agent Handoff Protocol / CLI Surface: Integration

## 1. Integration Scope
- Integrated `cmd_handoff` into `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Exposed subcommands:
  - `aiosh handoff list`
  - `aiosh handoff show`
  - `aiosh handoff initiate`
  - `aiosh handoff accept`
  - `aiosh handoff reject`
  - `aiosh handoff complete`
  - `aiosh handoff cancel`
- Validated end-to-end integration via `tools/test_handoff_suites.py` (H1..H3) and `tools/test_handoff_unit.py` (U01..U07).
