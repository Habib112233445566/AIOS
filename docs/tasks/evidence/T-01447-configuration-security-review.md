# T-01447: User Session Bootstrap — Configuration: Security Review

## Metadata
- **Task ID:** `T-01447`
- **Subsystem:** `code/aiosh-rust/aiosh-core`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`
- **Component:** User Session Bootstrap Configuration Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Threat Modeling & Scope

The security review for `SessionConfig` evaluated the attack surface where untrusted user input, environment variables, or malicious configuration files could destabilize the user session subsystem, execute path traversal, or bypass Policy Enforcement Point (PEP) controls.

### Evaluated Vectors:
1. **AS-01: Path Traversal & Arbitrary File Overwrite**
   - *Threat:* Malicious `--store` or `--config` paths attempting `../../etc/shadow` or null-byte truncation.
   - *Mitigation:* `SC1` enforces $\le 1024$ bytes and rejects all control characters and `\0` null bytes. Suffix validation and canonical pathing ensure stores stay in intended boundaries.
2. **AS-02: Memory Exhaustion / Zip-Bomb / Large File Flooding**
   - *Threat:* Supplying a multi-gigabyte `/dev/zero` or huge JSON configuration file to cause Out-Of-Memory (OOM) panics.
   - *Mitigation:* `SC7` enforces strict filesystem metadata checks ($\le 64\text{ KiB}$) and uses `std::io::Read::take(65_537)` during streamed reads to prevent unbounded memory allocation.
3. **AS-03: Resource Quota Starvation via Zero or Negative Limits**
   - *Threat:* Setting `max_sessions_per_user = 0` or `max_total_sessions = 0` to permanently lock out users or crash the service.
   - *Mitigation:* `SC2` bounds `max_sessions_per_user` to $[1 \dots 128]$; `SC3` bounds `max_total_sessions` to $[10 \dots 10,000]$.
4. **AS-04: Idle Auto-Lock Bypass**
   - *Threat:* Configuring `default_idle_timeout_seconds` to 0 or negative values to prevent workstation auto-locking.
   - *Mitigation:* `SC4` strictly enforces a minimum idle timeout of 10 seconds and an upper ceiling of 86,400 seconds (24 hours).
5. **AS-05: Unaudited Administrative Mutation**
   - *Threat:* Operator or autonomous agent querying or altering configuration without audit trail.
   - *Mitigation:* Both CLI (`classify_and_emit`) and MCP (`dispatch::recorded_call`) emit non-repudiable audit events to SQLite WAL (`$AIOSH_HOME/audit.db`) with monotonic sequence numbers and hash-chaining.
6. **AS-06: PEP Capability Gating**
   - *Threat:* Unauthorized MCP tool invocation without valid `grant_id`.
   - *Mitigation:* Gated by PEP capability checks in `dispatch::recorded_call`.

---

## 2. Policy Bypass Audit Results

- **Known Policy Bypasses Remaining:** `0`.
- **Fail-Safe Envelope Parity:** Verified across all error paths; zero silent failures or unwrapped panics.
- **Verdict:** PASS.
