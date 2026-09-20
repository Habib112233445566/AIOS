# Task Evidence: T-02030 - Capability Model / CLI surface: Verification & Evidence (Sub-Epic 3 Formal Closure)

## 1. Overview
- **Task ID**: `T-02030`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Formally verify the `aiosh capability` CLI surface and close Sub-Epic 3 with full evidence.

---

## 2. Verification Summary

### 2.1 Test Verification
- **Rust Unit Tests**:
  - Module: `capability_cli_tests` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
  - Tests:
    - `test_capability_cli_help_and_unknown`
    - `test_capability_cli_path_hygiene`
    - `test_capability_cli_issue_show_attenuate_revoke_flow`
    - `test_capability_cli_hardening_validation`
  - Result: 4 passed, 0 failed, 0 ignored.
- **Python Smoke & Integration Suite**:
  - Script: `code/aiosh-cli/tests/test_capability_cli_smoke.py`.
  - Result: 4/4 checks passed (help, unknown command, path hygiene, full lifecycle).

### 2.2 Invariant Verification
- Verified zero ambient authority enforcement.
- Verified `--issuer` restriction to `kernel` and `admin:*`.
- Verified path hygiene on `--store` rejecting `..`, control characters, and non-JSON extensions.
- Verified structured `--json` envelopes and exit codes (0, 1, 2).
- Verified audit logging to `AuditRing` on every command invocation.

---

## 3. Sub-Epic 3 Formal Closure
Sub-Epic 3 (`CLI surface`, T-02021 through T-02030) is hereby formally closed.
