# Task Evidence: T-01959 (System Update / automated tests: Documentation)

## Summary
Authored Section 9 ("Automated Testing & End-to-End Verification Subsystem") in `docs/system_update.md`.
The documentation covers:
1. **Architecture**: Rust native test harness (`test_system_update_e2e.rs`) and Python MCP smoke test (`test_system_update_e2e_smoke.py`).
2. **Invariants Enforced (UTEST1 - UTEST6)**:
   - `UTEST1`: Clean A/B update lifecycle progression with boot confirmation.
   - `UTEST2`: Cryptographic fault injection (bit-flip and truncation).
   - `UTEST3`: Boot failure simulation and rollback to active slot.
   - `UTEST4`: Quota and symlink traversal defense.
   - `UTEST5`: Out-of-order state transitions and error safety.
   - `UTEST6`: Cross-substrate JSON serialization parity.
3. **Execution Commands**: Copy-pasteable test invocation commands for Rust and Python.
4. **Constraints & Known Limitations**: User-space mock directory emulation vs physical block device partition flips.
5. **Evidence Artifacts**: Direct links to all Sub-Epic 6 evidence files.
