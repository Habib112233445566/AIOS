# T-01668: Hardening

See full hardening details in [T-01668-security-policy-hardening.md](./T-01668-security-policy-hardening.md).
Hardening measures:
- Regular file verification (`metadata.is_file()`).
- Bounded reading with `MAX_POLICY_FILE_BYTES` (64 KiB).
- Standardized error envelopes and honest failure audit rows.
