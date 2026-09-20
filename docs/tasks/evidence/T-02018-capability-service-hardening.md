# Task Evidence: T-02018 - Capability Model / core service: Hardening (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02018`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Implement defensive hardening against threats identified in T-02017 security review.

---

## 2. Hardening Measures Implemented

1. **Registry Capacity Limiting**:
   - Added constant `MAX_CAPABILITIES_IN_REGISTRY = 10_000`.
   - In `issue_root_capability` and `attenuate_capability`, check current registry size against the limit and reject additions exceeding it.
   - In `load_from_path`, verify deserialized entries do not exceed `MAX_CAPABILITIES_IN_REGISTRY`.

2. **Root Issuer Authorization**:
   - In `issue_root_capability`, enforce that `issuer == "kernel" || issuer.starts_with("admin:")`.
   - Ambient or unprivileged callers are rejected with `CapabilityError::ValidationError`.

3. **Revocation Cycle Detection**:
   - In `revoke_capability`, added a `visited: HashSet<String>` set.
   - Guarded BFS queue traversal to prevent infinite loops in the presence of corrupted or cyclic lineage structures.

4. **Path Validation (`validate_service_path`)**:
   - Reject paths longer than 1024 characters.
   - Reject paths with control characters.
   - Reject path traversal (`..` components).
   - Require `.json` file extension.
   - Applied to both `save_to_path` and `load_from_path`.

5. **Atomic Persistence & Temp File Cleanup**:
   - Ensured atomic write via temporary file with PID suffix and cleanup on failure.
   - Symlink protection via `symlink_metadata()`.

---

## 3. Verification & Test Evidence
- Added 3 new unit tests:
  - `test_cserv7_hardening_issuer_authorization`
  - `test_cserv8_hardening_path_validation`
  - `test_cserv9_hardening_cycle_detection_in_revoke`
- Full test suite passed without errors.
