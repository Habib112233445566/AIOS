# T-00937 — Agent Handoff Protocol / CLI Surface: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-1: Command Line Argument / Payload Injection
- **Threat**: Malicious actor supplies malformed JSON or binary control chars in `--payload` or `--summary`.
- **Mitigation**: Serde JSON parsing ensures payload validity, and string lengths are bounded and escaped in outputs.

### AS-2: State Machine Bypass via CLI Flags
- **Threat**: User attempts to force transition of an already completed/rejected handoff via CLI subcommand.
- **Mitigation**: `HandoffStore` validates current status before allowing transition; invalid transitions return exit code 1 with explicit error.

### AS-3: Audit Evasion
- **Threat**: Executing state changes without leaving an audit trail.
- **Mitigation**: Every state-changing CLI subcommand (`initiate`, `accept`, `reject`, `complete`, `cancel`) invokes `classify_and_emit` synchronously prior to returning.
