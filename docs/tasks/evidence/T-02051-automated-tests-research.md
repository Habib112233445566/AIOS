# Task Evidence: T-02051 (Capability Model / automated tests: Research)

## Task Information
- **Task ID**: T-02051
- **Title**: Capability Model / automated tests: Research
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 6: Automated Tests
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Research
1. **Scope & Objectives of Automated Testing Sub-Epic**:
   - Establish a comprehensive, hermetic automated test framework for the Capability Model.
   - Formulate automated test invariants `CAPTEST1..CAPTEST6`.
   - Provide end-to-end integration test harnesses across the core Rust service, CLI subcommands, and MCP JSON-RPC interface.

2. **Automated Test Matrix Requirements**:
   - **Fixture Harness (`MockCapabilityEnv`)**:
     - Hermetic temporary directory management using `tempfile::tempdir`.
     - Standard test fixtures representing diverse capability states:
       - Root admin capability (`agent:admin`)
       - Scoped filesystem worker capability (`agent:worker`)
       - Network-scoped capability (`agent:netclient`)
       - Expired capability (`agent:expired`)
       - Multi-tier attenuated lineage (Root $\rightarrow$ Child 1 $\rightarrow$ Child 2 $\rightarrow$ Child 3)
       - Quota-constrained capabilities (invocation count, byte limits)
   - **Test Scenarios**:
     - **Full Lifecycle Matrix**: Issue root $\rightarrow$ derive attenuated children $\rightarrow$ evaluate access $\rightarrow$ consume quotas $\rightarrow$ cascade revoke.
     - **Monotonic Attenuation Invariants**: Exhaustive verification that children cannot expand rights, widen scopes, relax constraints, or extend expiration.
     - **Cascade Revocation & Lineage Tracking**: Verification that revoking an ancestor capability revokes all descendants regardless of depth, while preserving unrelated branches.
     - **Quota & Temporal Invariants**: Expiration enforcement, invocation limit enforcement, byte limit enforcement, and safe pruning.
     - **Fault Injection & Hardening Verification**: Malformed JSON store files, symlink path attacks, directory traversal injection, control character scrubbing.
     - **Cross-Surface E2E Smoke**: Automated Python test suite verifying parity between CLI and MCP surfaces.

3. **Invariants Formulation (`CAPTEST1..CAPTEST6`)**:
   - `CAPTEST1`: Hermetic isolation — tests execute in temporary environments without mutating system state.
   - `CAPTEST2`: Lineage integrity — multi-level parent-child relationships remain consistent across mutations.
   - `CAPTEST3`: Monotonicity enforcement — rights/scopes can never expand during attenuation.
   - `CAPTEST4`: Cascade completeness — revoking a node revokes 100% of its descendant tree.
   - `CAPTEST5`: Quota atomicity — quota decrements/increments are strictly bounded and cannot underflow.
   - `CAPTEST6`: Fault tolerance — invalid or corrupted persistence data fails closed with clear errors.
