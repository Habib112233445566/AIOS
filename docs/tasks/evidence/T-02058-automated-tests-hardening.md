# Task Evidence: T-02058 (Capability Model / automated tests: Hardening)

## Task Information
- **Task ID**: T-02058
- **Title**: Capability Model / automated tests: Hardening
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 6: Automated Tests
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Hardening Actions
1. **Deep Hierarchy Stress Testing**:
   - Added `test_automated_capability_deep_hierarchy_stress` in `test_capability_automated.rs`.
   - Generates a 50-level deep attenuated capability tree (Root $\rightarrow$ Level 1 $\rightarrow$ ... $\rightarrow$ Level 50).
   - Validates access at level 50.
   - Triggers cascade revocation at Level 1, verifying that all 50 descendant capabilities are revoked transitively without stack overflow or performance degradation.
   - Verifies that Root remains active and unaffected.

2. **Leak-Proof Subprocess Management in Python E2E Smoke Harness**:
   - Hardened `run_mcp` in `test_capability_automated_smoke.py` with a `finally` block ensuring that any child process whose poll status is `None` is immediately killed with `p.kill()` and reaped with `p.wait()`.
   - Eliminates the possibility of orphaned zombie processes during test failure or timeout.
