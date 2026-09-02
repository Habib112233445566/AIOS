# T-00737 — Secrets & Access Hygiene / CLI surface: Security Review

## 1. Threat Model & Abuse Analysis

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **A-1** | Raw Secret Leakage via CLI / Stdout | `redact_secret_value` transforms all candidate values prior to building `SecretFinding`. Raw secrets are never returned in stdout, JSON envelopes, or SQLite audit records. | Mitigated |
| **A-2** | Large File / Memory Exhaustion DoS | `max_bytes` bounds (default 16 MiB) and line length caps (`MAX_LINE_SCAN_LENGTH` = 4096 bytes) prevent memory spiking on minified bundles or enormous artifacts. | Mitigated |
| **A-3** | Binary / Null-Byte Parsing Freeze | 512-byte header inspection detects and skips binary assets immediately without regex/string parsing. | Mitigated |
| **A-4** | Stealth Workspace Inspection | All command executions emit an immutable audit row (`AuditRing::write`) recording actor, timestamps, path, and scan counts. | Mitigated |

## 2. Policy Compliance
- **Read-Only Guarantees**: `aiosh secrets <scan|check>` performs no mutating operations on disk.
- **Fail-Closed Security**: Errors during file opening or workspace walking yield explicit auditable error rows.
