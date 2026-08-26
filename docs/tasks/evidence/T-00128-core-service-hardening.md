# T-00128 — CI Smoke Orchestration / core service: Hardening

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration core service

## 1. Hardening Implementations (Rust Port)

To fully align with the v2.1 Rust stack mandate and harden the CI summary service, we migrated the `ci_service` into native Rust (`aiosh-core/src/ci.rs` and `aiosh-cli/src/main.rs`), allowing us to enforce hardening via the native kernel primitives.

- **Bounded Retries:** When `aiosh ci` loads the CI summary JSON artifact, it is subject to race conditions (the CI orchestrator might still be flushing the file). A bounded retry loop (3 attempts, 500ms apart) was added to ensure we don't fail spuriously on lock contention or write latency.
- **Size Caps:** `fs::metadata` is used to check the file size before opening it. If it exceeds 1MB, the service immediately throws a parse error rather than reading it into memory and OOMing the process.
- **Explicit Errors / Audit Row Emission:** As per ADR-0035 §F-2, all failures in parsing or file-finding result in an explicit exit code (2) **and** an honest audit row emission (`aiosh ci check` -> `error` or `failure`). There are no silent failures.
- **Resource Cleanup:** Rust's ownership model automatically drops the file handle (`fs::read_to_string`), ensuring no dangling file descriptors are leaked on the error path.

## 2. Abuse Scenarios & Mitigations

1. **Large File DoS:**
   - *Attack:* A compromised CI suite pipes 5GB of data into `/tmp/aiosh-ci-results.json` to crash the orchestrator via OOM.
   - *Mitigation:* The 1MB size cap `if meta.len() > 1024 * 1024` instantly short-circuits the load, emitting a failure audit row and preventing memory exhaustion.

2. **JSON Bomb / Hash Collision DoS:**
   - *Attack:* An artifact with deeply nested structures or colliding dictionary keys is supplied.
   - *Mitigation:* The strict `RunSummary` Rust struct used by `serde_json` drops unknown fields and enforces strict depth/type mappings.

3. **Silent Failure Evasion:**
   - *Attack:* A script supplies a malformed JSON file to cause an exception, hoping the gate "fails open" without leaving a trace of the failure in the system audit ring.
   - *Mitigation:* The error path calls `emit(..., outcome="error")` containing the parse error details in the `outcome_detail` field, permanently recording the failure in the SQLite WAL.

## 3. Verdict
All hardening requirements are met in the Rust implementation. No remaining known policy bypasses.
