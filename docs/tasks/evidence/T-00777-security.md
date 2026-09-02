# T-00777 — Secrets & Access Hygiene: Security Review

## 1. Threat Model & Policy Invariants

| Scenario ID | Threat Vector | Policy & Technical Mitigation | Status |
|---|---|---|---|
| **SEC-POL-1** | Accidental Secret Commit | Secrets scanning engine (`aiosh-core::secrets_service`) integrated into CLI and automated CI checks to block commits containing private keys (`SEC-001`), AWS tokens (`SEC-002`), GitHub PATs (`SEC-003`), generic keys (`SEC-004`), or plaintext passwords (`SEC-005`). | Mitigated |
| **SEC-POL-2** | Unredacted Secret Leakage in Logs | `redact_secret_value` enforces mandatory boundary-character masking (`XXXX****YYYY`) and SHA-256 fingerprinting before any finding is serialized or logged. | Mitigated |
| **SEC-POL-3** | Non-Compliant Policy References | `tools/check_security_policy.py` validates all relative link targets and OpenSSF scorecard criteria (S1..S5). | Mitigated |
| **SEC-POL-4** | Scanner DoS via Binary Ingestion | Header null-byte detection (first 512 bytes) and 16 MiB size cap prevent ReDoS or buffer spikes. | Mitigated |

## 2. Policy Governance
- Vulnerability reporting is channeled privately via GitHub Security Advisories.
- Policy amendments require sequential task execution under master ledger law.
