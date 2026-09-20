# Unit Test Evidence - T-01865: Network Bootstrap Security Policy Unit Test

- File: `code/aiosh-rust/aiosh-core/tests/test_network_policy.rs`
- Invariants `NPOL1..NPOL6` verified: 14 test cases testing default validation, disallowed interface types, prohibited names, allowlists, promiscuous detection, missing MAC, orphan routes, DNS rules, capacity limits, mode switches (Enforcing/Audit/Permissive), address redaction, atomic persistence, and 1 MB size limits.
