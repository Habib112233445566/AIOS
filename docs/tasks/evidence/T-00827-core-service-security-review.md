# T-00827 — Regression Triage / core service: Security Review

## 1. Threat Model & Core Service Security Analysis

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **TRG-SVC-1** | File Bloat / Memory Exhaustion via Store File | `TriageStore::load_from_path` checks file size bounds and rejects files exceeding `MAX_TRIAGE_STORE_BYTES` (1 MiB). | Mitigated |
| **TRG-SVC-2** | Arbitrary File Overwrite via Unsanitized Paths | Parent directory creation and file writing require valid explicit filesystem paths. Path resolution checks prevent directory traversal. | Mitigated |
| **TRG-SVC-3** | Ingestion Injection via Malformed CI Summaries | `ingest_ci_summary` bounds extracted strings and validates failure records before insertion into store indices. | Mitigated |

## 2. Policy Invariants
- Strict file size bounds on persisted store JSON files.
- Fail-closed error handling during disk I/O.
