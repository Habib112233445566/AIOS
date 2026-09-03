# T-01028 — Distro Selection & Justification / CLI Surface: Hardening

## 1. Hardening Deliverables
- **Structured Error Reporting**:
  - Non-zero exit code mapping strictly follows convention: `1` for missing/invalid profiles or unparseable store files, `2` for syntax errors or missing required arguments.
  - Informative stderr diagnostics on all error paths (no silent exits).
- **Audit Row Invariant for Failures**:
  - When operations fail, an honest audit row is persisted logging the error context and non-zero outcome.
- **Resource & Bounded Execution**:
  - Store paths supplied via `--store` are bounded by `MAX_STORE_BYTES` (10 MiB) before allocation.
  - Process argument parsing operates in bounded heap memory with non-UTF8 replacement.

## 2. Test Verification
All positive and negative cases verified via `code/aiosh-cli/tests/test_distro_cli_smoke.py`.
```
PASS: aiosh distro show missing id returns 2
PASS: aiosh distro show nonexistent returns 1
ALL DISTRO CLI SMOKE TESTS PASSED!
```
