# T-00727 — Secrets & Access Hygiene / core service: Security Review

## 1. Security Review Scope
This task evaluates the security posture, threat vectors, and abuse scenarios for the secrets scanning service in `code/aiosh-rust/aiosh-core/src/secrets_service.rs`.

## 2. Threat Analysis & Abuse Scenarios

### Scenario 1: Resource Exhaustion via Pathological Files
- **Threat**: An attacker plants multi-gigabyte files or minified JavaScript lines spanning millions of characters to cause Out-Of-Memory (OOM) conditions or ReDoS latency spikes during scanning.
- **Mitigation**: `scan_file_for_secrets` enforces a strict file size cap (`max_file_bytes`, default 16 MiB) and clamps line buffer scanning to `MAX_LINE_SCAN_LENGTH` (4096 bytes).
- **Verdict**: PASS.

### Scenario 2: Binary File Ingestion Spikes
- **Threat**: Scanning compiled binaries or media files containing random byte sequences that trigger false positives and excessive memory allocations.
- **Mitigation**: The scanner checks the first 512 bytes of every file for null bytes (`\0`) and skips binary files immediately.
- **Verdict**: PASS.

### Scenario 3: Secret Leakage in Finding Data & Error Envelopes
- **Threat**: Discovered secret payloads are written to memory buffers or error messages in plaintext.
- **Mitigation**: Discovered values are immediately processed through `redact_secret_value` before `SecretFinding` instantiation; unredacted secrets are discarded immediately.
- **Verdict**: PASS.

## 3. Compliance Verification
- `python tools/check_security_policy.py`: S1..S5 PASS.
