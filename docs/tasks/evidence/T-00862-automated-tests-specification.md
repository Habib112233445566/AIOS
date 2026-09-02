# T-00862 — Regression Triage / Automated Tests: Specification

## 1. Test Criteria Specification (T1..T6 Matrix)

| Criterion | Target | Verification Method | Assertions |
|---|---|---|---|
| `T1` | Data Model | `cargo test --lib triage::tests` | Fingerprint determinism, bounds, report invariants |
| `T2` | Core Service | `cargo test --lib triage_service::tests` | Ingestion, deduplication, store roundtrip, reopen lifecycle |
| `T3` | CLI Surface | `cargo test -p aiosh-cli --bin aiosh -- test_cmd_triage_flow` | Command dispatch, JSON outputs, error codes |
| `T4` | MCP Server | `cargo test -p aiosh-mcp --bin aiosh-mcp -- test_mcp_triage_tools` | JSON-RPC tool schemas, audit ring emission |
| `T5` | Configuration | `cargo test --lib triage_config::tests` | Schema validation, boundary checks, suite filters |
| `T6` | End-to-End Flow | `test_t6_e2e_lifecycle_suite()` in `tools/test_triage_suites.py` | Ingest -> Check (fail) -> Resolve -> Check (pass) -> Reopen on recurrence |

## 2. Invariant & Isolation Guarantees
- All tests execute against ephemeral temp directories (`std::env::temp_dir()`).
- All process invocations execute with a hard 120s timeout limit.
- Zero leftover disk state upon test completion.
