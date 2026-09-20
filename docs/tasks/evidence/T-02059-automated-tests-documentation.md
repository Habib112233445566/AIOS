# Evidence: T-02059 - automated tests: Documentation

## Task Overview
- **Task ID**: `T-02059`
- **Sub-Epic**: Sub-Epic 6: Automated Tests (`T-02051`..`T-02060`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Formally document the automated test harness, coverage strategy, verification invariants (`CAPTEST1..6`), test case inventories (Rust and Python), and execution procedures.

## Documentation Additions
Authored Section 11 in `docs/capability_model.md`:
- **11. Automated Test Framework**:
  - Verification Invariants:
    - `CAPTEST1` (Determinism): Execution produces deterministic outcomes across identical seeds.
    - `CAPTEST2` (Complete Coverage): All capability lifecycle states (Issued, Active, Revoked, Expired, Depleted) exercised.
    - `CAPTEST3` (Attenuation Monotonicity): Attenuated capabilities strictly preserve monotonicity.
    - `CAPTEST4` (Cascade Revocation Completeness): Revoking an ancestor invalidates all descendant tokens.
    - `CAPTEST5` (Concurrency Safety): Multi-threaded token verification and state transitions remain race-free under high contention.
    - `CAPTEST6` (Interprocess Isolation): MCP daemon and client interact cleanly without leaking processes or capabilities.
  - Test Suite Inventory:
    - Rust Unit & Automated Suite (`code/aiosh-rust/aiosh-core/tests/test_capability_automated.rs`): 8 tests covering issuance, temporal expiration, invocation limits, scope restriction, attenuation monotonicity, cascade revocation, concurrency stress (8 threads x 100 iterations), and deep hierarchy stress (50 levels).
    - Python MCP Smoke Suite (`code/aiosh-mcp/tests/test_capability_automated_smoke.py`): 3 tests covering lifecycle via MCP protocol, attenuation hierarchy via MCP protocol, and process isolation / cleanup.

## Verification
- Documentation cross-referenced with test suite implementations.
- All code paths and invariant references verified against codebase.
