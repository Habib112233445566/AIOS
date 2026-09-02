# T-00978 — Agent Handoff Protocol / Security Policy: Hardening

## 1. Hardening Defenses Implemented
- **Actor ID Sanitization**: Whitespace trimmed and case normalized during authorization evaluations.
- **Fail-Closed Default**: Unknown actions or actors default strictly to `false` and return an informative `Err` message.
- **Immutable Status Defense**: Terminal states (`Completed`, `Rejected`, `Cancelled`) cannot be modified regardless of actor privileges.
