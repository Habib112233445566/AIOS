# T-02457: Sandbox Enforcement Automated Tests Security Review

## 1. Security Review Scope
This review evaluates the automated testing infrastructure of Sandbox Enforcement (`test_sandbox_automated.rs` and `test_sandbox_automated_smoke.py`) for test harness robustness, isolation, resource containment, and abuse scenarios.

---

## 2. Threat Modeling & Abuse Scenarios

### Abuse Scenario 1: Orphan Child Process Leaks (Process Table Exhaustion)
- **Threat**: Tests spawning child processes (`python`, `aiosh-mcp`) could leave detached zombie processes if assertions fail or timeouts occur.
- **Mitigation**:
  - Python tests utilize `communicate(timeout=10)` with explicit process cleanup.
  - Rust automated tests run supervised processes through `SandboxService` which reaps exit status and drops handles upon completion.

### Abuse Scenario 2: Test Artifact and DB Leaks
- **Threat**: Tests generating test databases on disk pollute the local filesystem or leak test data into production databases.
- **Mitigation**:
  - Rust automated tests utilize `tempfile::tempdir()`, guaranteeing that temporary database files (`autosandbox8.db`) are securely unlinked when the fixture goes out of scope.
  - No ambient writes to `~/.aiosh` occur during isolated automated runs.

### Abuse Scenario 3: Memory Exhaustion during Output Flooding Tests
- **Threat**: Testing high-throughput stdout capture (`print('A' * 1000000)`) could trigger Out-Of-Memory (OOM) panics.
- **Mitigation**:
  - Vector `AUTOSANDBOX6` verifies that output capture is clamped at `max_output_capture_bytes` (1024 bytes) without consuming excessive heap or buffering unbounded streams.

### Abuse Scenario 4: Factory Profile Mutation
- **Threat**: A rogue test mutating or removing default profiles compromises subsequent system operations.
- **Mitigation**:
  - Vector `AUTOSANDBOX1` strictly enforces that protected factory profiles (`standard`, `strict`, `permissive`) reject removal (`ERR_SANDBOX_CANNOT_DELETE_DEFAULT`).

### Abuse Scenario 5: SQLite WAL Lock Contention under Concurrency
- **Threat**: 4 worker threads writing simultaneous execution audit rows could encounter database busy/locked errors.
- **Mitigation**:
  - Verified under `AUTOSANDBOX8`: SQLite WAL mode with mutex serialization in `SandboxService` guarantees zero deadlocks across all concurrent threads.

---

## 3. Findings & Verdict
No security bypasses or test harness vulnerabilities were identified. Status: **PASS / CLEAN**.
