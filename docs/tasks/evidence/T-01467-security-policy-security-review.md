# T-01467: User Session Bootstrap — Security Policy: Security Review

## Metadata
- **Task ID:** `T-01467`
- **Subsystem:** `code/aiosh-rust/aiosh-core`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`
- **Component:** User Session Bootstrap Security Policy Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Threat Modeling & Abuse Scenarios

The security review for `UserSessionSecurityPolicy` evaluated threat vectors targeting privilege boundaries, console device isolation, runtime environment security, and autonomous AI execution controls.

### Evaluated Scenarios:
1. **AS-01: Unauthorized Root Privilege Escalation**
   - *Threat:* An adversary or rogue process creates a session with `uid == 0` to execute commands as root.
   - *Mitigation:* `SSP1` enforces `disallow_root: true` by default. Any session with `uid == 0` is rejected with fatal violation `SSP1-ROOT-DISALLOWED` unless explicitly listed in `allowed_root_users`.
2. **AS-02: Greeter Class Impersonation & Scope Hijacking**
   - *Threat:* Unprivileged user initializes a `SessionClass::Greeter` session without a physical display to intercept authentication tokens.
   - *Mitigation:* `SSP1` requires greeters to be system users (UID < 1000 or allowlisted user such as `lightdm`). `SSP2` mandates an explicit display identifier and rejects remote hosts for greeter sessions.
3. **AS-03: Console Hardware Snooping via Remote Seat0 Attachment**
   - *Threat:* Remote attacker attaches a remote session to physical `seat0`, intercepting display frames and input events of physical workstation operators.
   - *Mitigation:* `SSP3` enforces that if `remote_host.is_some()`, attachment to `seat0` is rejected with `SSP3-REMOTE-SEAT0-FORBIDDEN` unless `allow_remote_seat0` is explicitly true.
4. **AS-04: Dynamic Linker & Interpreter Hijacking via Injected Variables**
   - *Threat:* Attacker passes `LD_PRELOAD`, `LD_LIBRARY_PATH`, `PYTHONPATH`, `NODE_OPTIONS`, or `IFS` in session environment to force execution of malicious shared libraries or alter program behavior.
   - *Mitigation:* `SSP4` strictly disallows dangerous dynamic linker and interpreter hook variables, rejecting them with `SSP4-DISALLOWED-ENV-VAR`.
5. **AS-05: Denial of Service via Concurrent Session Flooding**
   - *Threat:* Malicious script spawns hundreds of sessions under one account, exhausting OS processes and PTY allocation.
   - *Mitigation:* `SSP5` enforces maximum sessions per user ($[1 \dots 128]$, default 32) and total store capacity ($[10 \dots 10,000]$, default 1,024).
6. **AS-06: Unbounded Policy Ingestion & Memory Exhaustion**
   - *Threat:* Supplying a huge policy file causing Out-Of-Memory panic.
   - *Mitigation:* `SSP7` verifies file size ($\le 64\text{ KiB}$) and stream-caps ingestion with `file.take(MAX_POLICY_FILE_BYTES + 1)`.
7. **AS-07: PEP Capability Gating & Audit Non-Repudiation**
   - *Threat:* Unauthorized invocation of MCP policy tools or unaudited policy evaluation.
   - *Mitigation:* All CLI invocations call `classify_and_emit`; MCP invocations pass through `dispatch::recorded_call`, writing monotonic sequence rows to SQLite WAL.

---

## 2. Policy Bypass Audit Results

- **Known Policy Bypasses Remaining:** `0`.
- **Fail-Safe Envelope Parity:** Verified across all error paths; zero silent failures or unwrapped panics.
- **Verdict:** PASS.
