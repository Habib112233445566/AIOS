# T-00717 — Secrets & Access Hygiene / data model: Security Review

## 1. Security Review Scope
This task evaluates the security invariants, threat models, and safe serialization properties of the Secrets & Access Hygiene data model in `code/aiosh-rust/aiosh-core/src/secrets.rs`.

## 2. Threat Analysis & Abuse Scenarios

### Scenario 1: Accidental Plaintext Secret Exposure
- **Threat**: Raw secret tokens or credentials discovered during scans are stored verbatim in `SecretFinding` structures and emitted to CLI outputs, MCP JSON-RPC messages, or audit ring databases.
- **Mitigation**: `redact_secret_value` enforces strict redaction on all snippet representations, preserving at most 4 prefix and 4 suffix characters with `****` masking for strings $\ge 12$ characters, and full `[REDACTED]` masking for strings $< 12$ characters. Raw unmasked strings are never stored on `SecretFinding`.
- **Verdict**: PASS.

### Scenario 2: Integrity Tampering of Scan Reports
- **Threat**: Compromised scan output falsely sets `is_clean: true` or modifies finding counts to hide active credential leaks.
- **Mitigation**: `validate_secret_report` mechanically checks arithmetic invariant consistency (`total_findings == critical + high + medium + low` and `is_clean == (total_findings == 0)`).
- **Verdict**: PASS.

### Scenario 3: Secret Fingerprint Collision
- **Threat**: Weak hashing algorithms allow collisions in finding deduplication.
- **Mitigation**: Finding fingerprints are derived from 256-bit SHA-256 digests.
- **Verdict**: PASS.

## 3. Compliance Verification
- `python tools/check_security_policy.py`: S1..S5 PASS.
