# Task Evidence: T-01937 - System Update / MCP/API surface: Security Review

- **Task**: `T-01937`
- **Sub-Epic**: `Sub-Epic 4: Model Context Protocol (MCP) & API Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Security Review
Conducted comprehensive threat modeling and security review of the System Update MCP/API surface exposed by `code/aiosh-rust/aiosh-mcp/src/main.rs`:

### Threat Vectors Analyzed:
1. **`THREAT-UMCP-01` (Path Traversal via `state_dir` / `manifest_path`)**:
   - *Attack*: Providing `../../etc/shadow` or directory escape vectors in `state_dir` or `manifest_path`.
   - *Mitigation*: Sanitized via `resolve_update_service()`: enforces maximum length $\le 1024$ bytes and strictly forbids control characters (`\n`, `\t`, `\r`, `\0`, `\x07`). `state_dir` only reads/writes strictly named files (`slot_status.json`, `update_status.json`).
   - *Hardening Action for T-01938*: Ensure `manifest_path` parameter in `aios.update.check` and `aios.update.apply` enforces identical path length and control character sanitization before opening files.

2. **`THREAT-UMCP-02` (Payload Bomb / Malformed Manifest Denial of Service)**:
   - *Attack*: Sending deeply nested or oversized JSON objects or pointing `manifest_path` to `/dev/urandom` to induce memory exhaustion.
   - *Mitigation*: Deserialization to strongly-typed `UpdateManifest` with bounded fields.
   - *Hardening Action for T-01938*: When reading `manifest_path` in MCP handlers, enforce 1MB file size limits prior to reading into memory (matching CLI invariant `UCLI3`).

3. **`THREAT-UMCP-03` (Unbounded Version Strings / State Poisoning)**:
   - *Attack*: Passing megabyte-long version strings in `aios.update.confirm`.
   - *Mitigation*: Enforced strict length cap: `if ver.len() > 64 { return Err("version length cannot exceed 64 characters"); }`.

4. **`THREAT-UMCP-04` (State Machine Evasion / Premature Confirmation)**:
   - *Attack*: Calling `aios.update.confirm` while state is `idle`, `downloading`, or `verifying` to prematurely flip active boot slot.
   - *Mitigation*: Service-level state machine strictly enforces that confirmation is only permitted from `ReadyToReboot`. Any call from other states fails with `ok: false`.

5. **`THREAT-UMCP-05` (PEP Policy Bypass / Unauthorized State Mutation)**:
   - *Attack*: Unprivileged agent invoking `aios.update.apply` or `aios.update.confirm` without authorization.
   - *Mitigation*: All 6 tools are dispatched through `dispatch::recorded_call`, passing `grant_id`. Policy evaluation determines if invocation is permitted or refused.

6. **`THREAT-UMCP-06` (Covert State Changes / Missing Audit Trail)**:
   - *Attack*: Executing update commands without leaving an audit record in the WAL ring.
   - *Mitigation*: `dispatch::recorded_call` guarantees that every invocation (whether success or failure) writes an immutable audit record containing caller ID, tool name, arguments, and outcome.

## Conclusion & Next Steps
Zero open policy bypasses found. Hardening recommendations (enforcing `manifest_path` validation and 1MB size bounds in `aios.update.check` and `apply` MCP handlers) are queued for implementation in `T-01938`.
