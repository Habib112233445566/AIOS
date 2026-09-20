# Evidence: T-02068 - security policy: Hardening

## Task Overview
- **Task ID**: `T-02068`
- **Sub-Epic**: Sub-Epic 7: Security Policy (`T-02061`..`T-02070`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Harden Capability Security Policy against path traversal evasion, host obfuscation, and derivation tree cycles.

## Hardening Implemented
1. **Path Normalization & Traversal Detection (`code/aiosh-rust/aiosh-core/src/capability_policy.rs`)**:
   - Implemented `normalize_path()` to convert backslashes to forward slashes, collapse redundant slashes (e.g. `//etc///shadow` $\rightarrow$ `/etc/shadow`), and strip trailing slashes.
   - Added traversal detection (`CAPSEC_PATH_TRAVERSAL`): explicitly checks if any path component equals `..` and blocks issuance/attenuation.
   - Added ASCII control character detection (`CAPSEC_MALFORMED_PATH`).
2. **Network Host Sanitization (`code/aiosh-rust/aiosh-core/src/capability_policy.rs`)**:
   - Implemented `sanitize_host()` to strip accidental `:port` suffixes, strip IPv4/IPv6 square brackets (`[...]`), and strip trailing dots (`metadata.google.internal.`).
   - Hardened `CAPSEC_PROHIBITED_HOST` evaluation to compare sanitized representations.
3. **Cycle Prevention in Derivation Depth (`code/aiosh-rust/aiosh-core/src/capability_service.rs`)**:
   - Hardened `CapabilityService::get_derivation_depth()` with a `HashSet<String>` visited set to detect cycles in parent links.
   - Bounded traversal loop to a maximum of 256 iterations to prevent infinite loop denial of service.
4. **Automated Verification**:
   - Added `test_policy_hardening_and_cycle_prevention` in `test_capability_policy.rs`.
   - All 9 unit tests passed in 0.01s.
