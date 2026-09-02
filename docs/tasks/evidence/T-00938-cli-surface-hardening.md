# T-00938 — Agent Handoff Protocol / CLI Surface: Hardening

## 1. Hardening Defenses Implemented
- **Explicit Exit Codes**: Returns `2` for malformed/missing CLI arguments, `1` for state machine or persistence errors, `0` for successful execution.
- **Fail-Safe Store Loading**: `load_or_recover` automatically recovers from corrupted state files and issues a warning rather than crashing.
- **Input Sanitization & Parsing**: `parse_flag` and `has_flag` safely parse tokens without out-of-bounds panics.
