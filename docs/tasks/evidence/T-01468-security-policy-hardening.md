# T-01468: User Session Bootstrap — Security Policy: Hardening

## Metadata
- **Task ID:** `T-01468`
- **Subsystem:** `code/aiosh-rust/aiosh-core`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`
- **Component:** User Session Bootstrap Security Policy Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Hardening Measures Implemented

1. **Size Caps & Stream-Bounded File Ingestion**:
   - Policy file ingestion in `UserSessionSecurityPolicy::from_file` enforces a hard cap of 64 KiB (`MAX_POLICY_FILE_BYTES = 65_536`).
   - Sizing verified twice: first via filesystem metadata `metadata.len()`, second via stream bounding with `file.take(MAX_POLICY_FILE_BYTES + 1).read_to_end(&mut buffer)`, preventing memory spike attacks.

2. **String Sanitization & Argument Invariants**:
   - `policy_path` and `store_path` CLI and MCP arguments are capped at $\le 1024$ bytes and checked for ASCII control characters (`c.is_control()`).
   - Display strings are checked for $\le 32$ bytes without control characters.
   - Environment variable keys are checked for length $\le 256$, rejection of `=` characters, and absence of control characters.

3. **Strict Range Invariants**:
   - `max_sessions_per_user`: bounded to $[1 \dots 128]$.
   - `max_total_sessions`: bounded to $[10 \dots 10,000]$.
   - `max_env_vars`: bounded to $[1 \dots 1024]$.
   - `vtnr`: bounded to $[1 \dots 64]$.

4. **Zero Silent Failure & Standard Envelopes**:
   - Both CLI and MCP surfaces emit standardized JSON envelopes for all error conditions (file read error, JSON parse error, invalid argument, policy violation).
   - Under no circumstances does the engine silently fall back or drop violations.

5. **Non-Repudiation Audit Emission**:
   - Every failure path, invalid argument, and evaluation event emits an audit row to SQLite WAL with monotonic sequence ID and hash-chaining.

## 2. Verification
- Master test runner re-run: `python tools/test_session_suites.py`
- Result: Criteria `SB1..SB7` all PASS with 0 warnings and zero residual artifacts.
