# T-00837 — Regression Triage / CLI: Security Review

## 1. Threat Model & CLI Security Analysis

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **TRG-CLI-1** | Command / Argument Injection | All CLI arguments (`--target`, `--error`, `--notes`, `--repro`) are received as structured typed vectors; no shell execution or string evaluation occurs. | Mitigated |
| **TRG-CLI-2** | Unlogged Mutation Bypass | Modifying operations (`record`, `resolve`, `ingest`) call `classify_and_emit`, guaranteeing append-only audit trail logging. | Mitigated |
| **TRG-CLI-3** | Insecure Exit Code Masking | `aiosh triage check` returns exit 1 upon encountering any unresolved blocker or critical triage item, preventing CI pipelines from silently passing on critical regressions. | Mitigated |

## 2. Policy Invariants
- Zero unlogged state mutations.
- Structured parameter validation and safe exit code mapping.
