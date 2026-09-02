# T-00376 — Dependency & Toolchain Pinning / security policy: Integration

## 1. Integration Scope
This task integrates the Dependency & Toolchain Pinning security policy with the broader AIOS PEP gating mechanism, audit logging ring, and root security policy documentation.

## 2. Integration Mechanics
1. **PEP Subsystem (`code/aiosh-rust/aiosh-core/src/pep.rs`)**:
   - `aios.toolchain.set` and `toolchain.set` wired into `is_irreversible` matcher.
   - Any state-mutating toolchain command is immediately blocked unless accompanied by a verified grant token.
2. **Audit Ring Logging**:
   - All toolchain operations (read-only and mutating) route through audit context logging, ensuring immutable audit trail records in SQLite WAL database.
3. **Repository Security Policy (`SECURITY.md`)**:
   - Updated `SECURITY.md` with explicit toolchain integrity references in the Security Knowledge Index (`T-00367-security.md`, `T-00377-security.md`).

## 3. Verification
- `cargo test -p aiosh-core test_check_toolchain_policy_enforcement` -> PASS
- `python tools/check_security_policy.py` -> PASS (S1..S5)
- All integrated policy enforcement paths verified.
