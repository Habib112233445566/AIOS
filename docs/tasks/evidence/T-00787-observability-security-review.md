# T-00787 — Secrets & Access Hygiene / observability: Security Review

## 1. Threat Model & Observability Security Analysis

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **OBS-1** | Credential Exposure via Telemetry Strings | `SecretScanReport::summary_line` only exposes quantitative metrics (scanned files, total findings count, severity bucket breakdowns, and boolean cleanliness status). No raw secret tokens or unmasked strings are serialized into telemetry streams. | Mitigated |
| **OBS-2** | Log Injection via Malformed File Paths | File path identifiers inside telemetry are validated UTF-8 and sanitized during formatting to prevent ANSI escape sequence injection into terminal logs. | Mitigated |
| **OBS-3** | Audit Ring Tampering | Consequential actions record immutable audit rows in SQLite WAL with deterministic cryptographic hashing. | Mitigated |

## 2. Policy Invariants
- Zero raw secrets in telemetry metrics.
- All telemetry formatting is read-only and non-repudiable.
