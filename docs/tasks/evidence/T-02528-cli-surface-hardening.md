# Evidence: T-02528 Privilege Escalation Prevention CLI Surface Hardening

- **Task**: `T-02528`
- **Sub-Epic**: Privilege Escalation Prevention / CLI surface
- **Date**: 2026-09-29
- **Status**: PASSED

## Summary of Hardening Actions
1. **Grant Token & Capability Set Constraints**:
   - Added validation enforcing that `--grant` tokens do not contain control characters and do not exceed `MAX_GRANT_ID_LEN = 256` bytes (fails fast with code 2).
   - Added boundary checking enforcing requested capabilities do not exceed `MAX_CAPABILITIES_COUNT = 32` (fails fast with code 2).
2. **Actor Identifier Length Cap**:
   - Enforced maximum actor length `MAX_ACTOR_ID_LEN = 128` bytes across `status` and `elevate`.
3. **Store File Size Ceiling**:
   - Added 1 MiB (`1024 * 1024` bytes) hard ceiling check on `--store` files prior to deserialization to prevent memory exhaustion DoS.
4. **Verification**:
   - All CLI unit tests passed. 0 warnings, 0 errors.
