# T-00907 — Regression Triage / Recovery & Validation: Security Review

## 1. Threat Model & Abuse Scenarios

| ID | Abuse Scenario | Mitigation | Status |
|---|---|---|---|
| AS-1 | Parser crash / DoS on corrupted store JSON | `load_or_recover` catches parse and IO errors, returning a clean store and diagnostic error string | Mitigated |
| AS-2 | Malformed signature / injection into ID | `validate_triage_record` validates 64-char SHA-256 signature and `TRG-` ID prefix | Mitigated |
| AS-3 | Silent data loss on store reset | Recovery explicitly returns `Some(err_msg)` warning message so callers log the event | Mitigated |

## 2. Invariant Verification
- Input validation: All records validated prior to report compilation.
- Audit emission: All state transitions log to SQLite WAL audit ring.
- Zero open policy bypasses remain.
