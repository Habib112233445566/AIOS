# T-00772 — Secrets & Access Hygiene / security policy: Specification

## 1. Security Policy Specification
The AIOS security policy (`SECURITY.md`) specifies operational and reporting standards for vulnerability disclosure and secret management:

### What Counts as a Vulnerability (Secrets & Access Hygiene additions)
- Exposing plaintext API tokens, private keys, AWS credentials, or configuration passwords through any supported interface.
- Bypassing or disabling `redact_secret_value` or omitting cryptographic fingerprints on secret findings.
- Bypassing workspace or file secret scan gates in CI workflows.

### Redaction Invariants
- All candidate secret strings $\ge 12$ characters must be masked preserving only 4 prefix and 4 suffix characters (`XXXX****YYYY`).
- Shorter candidate strings must be completely masked (`****`).

### Security Knowledge Index Update
- `docs/tasks/evidence/T-00777-security.md` (Secrets & Access Hygiene security review).
