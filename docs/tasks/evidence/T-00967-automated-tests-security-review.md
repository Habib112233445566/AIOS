# T-00967 — Agent Handoff Protocol / Automated Tests: Security Review

## 1. Threat Modeling & Test Isolation Analysis

### AS-1: Shared Environment Test Leakage / Race Conditions
- **Threat**: Tests writing to static production paths causing inter-test race conditions or data pollution.
- **Mitigation**: All disk tests use isolated temp directories (`tempfile.TemporaryDirectory` / `tempdir()`) that are cleaned up upon exit.

### AS-2: Test Execution Timeout Hang
- **Threat**: Hanging sub-processes or infinite loops blocking CI runners indefinitely.
- **Mitigation**: All test subprocess calls specify explicit 120-second timeouts.

### AS-3: Non-deterministic Signature Validation Flakiness
- **Threat**: Hash mismatch due to dictionary key re-ordering across runs.
- **Mitigation**: Canonical JSON formatting (lexicographical sorting, no spaces) guarantees deterministic hashing.
