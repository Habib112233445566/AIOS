# Task Evidence: T-02228 (Grant Lifecycle / CLI surface: Hardening)

## 1. Metadata
- **Task ID:** `T-02228`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle CLI Surface Hardening (`code/aiosh-rust/aiosh-cli`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic: Grant Lifecycle (3/10) — CLI Surface Hardening

---

## 2. Hardening Measures Implemented

1. **Storage Path Validation & Defense-in-Depth**:
   - Added rigorous path hygiene checks via `aiosh_core::pep_decision_service::validate_pep_service_path` directly to the `g_store_path` resolution branch.
   - Enforces `.json` extension requirement, path length constraint ($\le 1,024$ bytes), control character rejection, and complete prohibition of directory traversal (`..`).
   - Rejection outputs structured error code `INVALID_STORE_PATH` with status code 2, logging failure to the audit trail.
2. **Resource Size Caps & Payload Limits**:
   - Implemented 16 MiB size cap guard against oversized, malicious, or circular grant storage files prior to parsing.
   - Prevents memory exhaustion / DoS vulnerabilities from maliciously inflated JSON payloads (`STORE_TOO_LARGE`).
3. **Structured Failure Envelopes**:
   - Guaranteed standard result envelopes (`code`, `data`, `error`) across all failure paths.
   - Disallowed silent failures or unhandled panics.
4. **Leak-Free Resource Cleanup & Audit Invariants**:
   - File handles and SQLite connections close deterministically via RAII guards.
   - Every failure path emits an honest, classified audit row to the SQLite audit database via `classify_and_emit` conforming to ADR-0035 §F-2.

---

## 3. Verification

### 3.1 Unit & Integration Suite
```text
> python -m pytest code/aiosh-cli/tests/test_pep_grant_cli.py
============================= test session starts =============================
platform win32 -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
rootdir: C:\Users\OBSESSION\Desktop\AIOS_MERGED
plugins: anyio-4.14.2
collected 5 items

code\aiosh-cli\tests\test_pep_grant_cli.py .....                         [100%]

============================== 5 passed in 5.28s ==============================
```

### 3.2 CLI Smoke Invariant Suite
```text
> python code/aiosh-cli/tests/test_pep_cli_smoke.py
PASS: aiosh pep --help
PASS: aiosh pep unknown_cmd returns 2
PASS: aiosh pep path hygiene enforcement
PASS: aiosh pep lifecycle and evaluation
PASS: aiosh pep security policy privilege boundary
PASS: aiosh pep report CLI integration
PASS: aiosh pep doc CLI integration
PASS: aiosh pep recovery & validation CLI integration
PASS: aiosh pep grant CLI integration
=== All PEP CLI tests passed ===
```

---

## 4. Acceptance Confirmation
- [x] Explicit failure modes return standard error envelope and non-zero exit codes.
- [x] Defense-in-depth path validation and 16 MiB size ceiling enforced.
- [x] Zero temp or connection leaks on all error paths.
- [x] Honest audit row emission on all failure and mutation paths.
