# T-01568 — Filesystem Layout security policy: Hardening

## Metadata
- **Task ID:** `T-01568`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** Complete — hardened security policy execution against timeouts, resource leaks, and unhandled errors.
- **Date:** 2026-09-19
- **Depends on:** `T-01567` (Security Review)
- **Feeds:** `T-01569` (Documentation)
- **Artifacts:** `docs/tasks/evidence/T-01568-security-policy-hardening.md`, `docs/tasks/evidence/T-01568-hardening.md`

---

## 1. Hardening Measures Implemented

1. **Bounded Subprocess Timeouts & Process Cleanup**:
   - Stdio interactions with `aiosh-mcp` enforce a 30-second timeout.
   - On `subprocess.TimeoutExpired`, processes are explicitly killed (`p.kill()`) and reaped (`p.wait()`) to prevent zombie processes.
   - CLI invocations (`aiosh grant`, `aiosh audit tail`, `aiosh layout`) enforce a 60-second timeout cap.

2. **Resource Containment & Ephemeral State**:
   - All tests use context-managed `tempfile.TemporaryDirectory()`, guaranteeing zero disk leakage or leftover temporary files across pass and fail paths.
   - Pre-gate refusals are strictly verified to produce zero side-effects on disk (`assert not Path(store).exists()`).

3. **Standard Result Envelope & Failure Mode Explicit Errors**:
   - Negative policy decisions return structured envelopes:
     - `ok: false`
     - `gate: "pep"`
     - `error`: Explanatory message detailing the scope or grant mismatch.
   - Never fails silently or with unhandled panics.

4. **Audit Chain Invariant Enforcement (ADR-0035 §F-2)**:
   - Refusals emit honest audit rows with `outcome="refused"` and `outcome_detail` recording the specific path/tool mismatch.
   - Hash chain continuity is preserved across allowed and refused actions.

---

## 2. Acceptance Confirmation

- [x] Failure modes produce explicit, auditable errors in the standard result envelope.
- [x] No temp file, socket, or child process leaks on error paths.
