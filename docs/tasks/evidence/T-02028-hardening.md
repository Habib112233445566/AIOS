# Task Evidence: T-02028 - Capability Model / CLI surface: Hardening (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02028`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Implement defensive hardening against failure modes and injection attacks identified in T-02027 security review.

---

## 2. Hardening Measures Implemented

1. **Scope Target Hygiene & Absolute Path Enforcement**:
   - In `parse_cli_scope`:
     - Reject empty targets, targets $> 1024$ chars, and targets with control characters.
     - For `filesystem` scope, enforce absolute path requirement and reject `..` traversal components.

2. **Subject & Issuer Identifier Validation**:
   - In `cmd_capability` (`issue`, `attenuate`, `check`):
     - Enforce `issuer` and `subject` length $\le 256$ chars and reject any control characters.

3. **Capability ID Hygiene**:
   - In `cmd_capability` (`show`, `revoke`):
     - Enforce `id` length $\le 128$ chars and reject any control characters.

4. **Robust Integer Quota Flag Parsing**:
   - In `issue` and `attenuate`:
     - Parse `--max-invocations` and `--quota-bytes` via `s.parse::<u64>()`.
     - Invalid integers (negative numbers, non-numeric strings) now return exit code 2 and structured error `{ "code": "INVALID_FLAG_VALUE" }` instead of silently defaulting or panicking.

5. **Terminal Sanitization**:
   - All printed error messages and non-JSON output strings pass through `sanitize_terminal` to eliminate ANSI/VT100 escape sequence injection (CWE-150).

---

## 3. Verification & Test Evidence
- Added `test_capability_cli_hardening_validation` to `capability_cli_tests`:
  - Verified invalid quota values return exit code 2.
  - Verified control characters in subject or scope target return exit code 2.
  - Verified relative filesystem paths return exit code 2.
  - Verified control characters in capability ID return exit code 2.
- All 4 unit tests in `capability_cli_tests` pass.
