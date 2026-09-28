# Evidence: T-02518 Privilege Escalation Prevention Core Service Hardening

- **Task**: `T-02518`
- **Sub-Epic**: Privilege Escalation Prevention / core service
- **Date**: 2026-09-29
- **Status**: PASSED

## Summary of Hardening Actions
1. **Capacity Clamping & Boundary Protections**:
   - Added `MIN_MAX_ACTIVE_CONTEXTS = 1` and `MAX_MAX_ACTIVE_CONTEXTS = 16384`.
   - Clamped capacity parameters in `with_capacity` to prevent unbounded memory allocation or zero-capacity deadlocks.
2. **Context & Actor Validation Pre-Flight**:
   - Added mandatory `context.validate()?` on `register_context`.
   - Enforced control character validation and empty-string checks across `unregister_context`, `drop_privilege`, and `revoke_elevation`, returning canonical error code `PRIVESC_ERR_INVALID_ACTOR`.
3. **Public Re-Exports**:
   - Re-exported `PRIVESC_MIN_MAX_ACTIVE_CONTEXTS` and `PRIVESC_MAX_MAX_ACTIVE_CONTEXTS` in `aiosh_core::lib`.
4. **Verification**:
   - Ran `cargo test -p aiosh-core --test test_privilege_service --test test_privilege_service_integration`: all 7 tests passed, 0 warnings, 0 errors.
