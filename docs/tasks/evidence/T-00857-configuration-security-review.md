# T-00857 — Regression Triage / Configuration: Security Review

## 1. Threat Model & Abuse Scenarios

| ID | Abuse Scenario | Mitigation | Status |
|---|---|---|---|
| AS-1 | Denial of Service via oversized config files | Enforced `MAX_CONFIG_FILE_BYTES = 65536` (64 KiB) before reading | Mitigated |
| AS-2 | Store file inflation causing memory exhaustion | Enforced bounded `max_store_bytes` ($16\text{ KiB} \le S \le 64\text{ MiB}$) | Mitigated |
| AS-3 | Malformed or malicious JSON injection | Strict `serde_json` strongly-typed deserialization rejecting extra/invalid fields | Mitigated |
| AS-4 | Silent suppression of critical regressions | Ingestion adheres to deterministic pattern matching; unhandled suites are explicit | Mitigated |

## 2. Invariant Verification
- Input validation: All config parameters validated via `TriageConfig::validate()`.
- Audit trail: State-changing triage commands emit structured audit events through `classify_and_emit`.
- Zero open policy bypasses detected.
