# T-02307: Audit Chain Extensions Data Model Security Review

## Threat Model & Security Posture
This security review evaluates the data model additions introduced for Audit Chain Extensions (`ExtendedAuditRow`, `AuditProvenance`, `AuditCausalLink`, `AuditSignature`, and `ExtendedAuditRowInput`).

## Abuse Scenarios & Mitigations

### 1. AS-EXT-01: Memory & SQLite Storage Exhaustion via Unbounded Extensions
- **Attack Vector**: A malicious agent or subtask injects massive JSON dictionaries into `extensions` or dozens of causal links to cause memory bloat, SQLite WAL write latency, or disk exhaustion.
- **Mitigation & Verification**:
  - `MAX_CAUSAL_LINKS` is capped at 16 entries.
  - `MAX_EXTENSION_ENTRIES` is capped at 32 keys.
  - `MAX_EXTENSION_PAYLOAD_BYTES` is strictly enforced at 64 KB (65,536 bytes).
  - Validation fails fast with `AUDIT_EXT_ERR_BOUNDS` prior to disk commitment.

### 2. AS-EXT-02: Causal Link Spoofing & Malformed Parent Hashes
- **Attack Vector**: Submitting non-hex, oversized, or control-character-laden parent event hashes in `AuditCausalLink` to trigger SQL injection or bypass DAG reconstruction.
- **Mitigation & Verification**:
  - `AuditCausalLink::validate()` asserts `parent_event_hash.len() == 64` and validates all characters match `is_ascii_hexdigit()`.
  - Causality types are length-capped and stripped of invalid whitespace.

### 3. AS-EXT-03: Asymmetric Signature Stripping and Forgery
- **Attack Vector**: Submitting counterfeit signature blocks or tampering with row attributes after signing.
- **Mitigation & Verification**:
  - Digital signatures sign the definitive SHA-256 row hash (`row.hash`), which itself binds all core fields, provenance metadata, causal links, and extension payloads in canonical JSON form.
  - Any alteration of covered attributes alters the hash, immediately failing signature verification and chain verification.

### 4. AS-EXT-04: Cross-Substrate Proto Desynchronization
- **Attack Vector**: Inconsistent key ordering or whitespace insertion in JSON fields causing distinct hash representations across substrates.
- **Mitigation & Verification**:
  - All extended proto maps utilize strict canonical JSON formatting with sorted keys and compact delimiters.
  - Omitted or empty fields serialize consistently, guaranteeing byte-identical hashes.

## Security Verdict
No policy bypasses, injection vectors, or unmitigated vulnerabilities identified. The data model is safe to proceed to hardening.
