# Unit Test Evidence - T-01845: Network Bootstrap Configuration Unit Test

- Test File: `code/aiosh-rust/aiosh-core/tests/test_network_config.rs`
- Invariants Tested: `NCONF1..NCONF6`
- 19 test cases covering default validity, path hygiene (empty, control chars, max length, path traversal), capacity limits (interfaces, routes, DNS), payload/timeout bounds, DNS syntax parsing, JSON roundtrip, atomic save/load, 1MB size limit rejection, and environment variable ingestion/fallback.
- Result: 19 passed; 0 failed.
