# T-00917 — Agent Handoff Protocol / Data Model: Security Review

## 1. Threat Model & Abuse Scenarios

| ID | Abuse Scenario | Mitigation | Status |
|---|---|---|---|
| AS-1 | Memory exhaustion via massive prompt payload | `MAX_PAYLOAD_BYTES` (64 KiB) and `MAX_CONTEXT_SUMMARY_BYTES` (4 KiB) bound record memory footprint | Mitigated |
| AS-2 | Signature spoofing / payload tampering | Normalized SHA-256 signature guarantees tamper evidence; any modification breaks hash check | Mitigated |
| AS-3 | Anonymous or blank agent impersonation | `validate_handoff_record` strictly forbids blank or whitespace sender/receiver IDs | Mitigated |

## 2. Invariant Verification
- Input bounds enforced on construction.
- Strict prefix `HND-` required on record identifiers.
- Zero open policy bypasses remain.
