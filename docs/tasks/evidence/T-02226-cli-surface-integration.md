# Task Evidence: T-02226 (Grant Lifecycle / CLI surface: Integration)

## 1. Metadata
- **Task ID:** `T-02226`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle CLI Surface Integration (`code/aiosh-rust/aiosh-cli`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic: Grant Lifecycle (3/10) — CLI Surface Integration

---

## 2. Integration Summary
1. **Production Surface Wiring**:
   - Integrated the `grant` subcommand into `aiosh pep` dispatch path (`cmd_pep` in `code/aiosh-rust/aiosh-cli/src/main.rs`).
   - Exposed all 7 lifecycle subcommands: `issue`, `attenuate`, `list`, `inspect`, `validate`, `revoke`, and `sweep`.
   - Wired discovery into `aiosh pep --help` command catalog, ensuring operators and orchestrators discover grant capabilities.
2. **Cross-Substrate Parity & Storage Invariants**:
   - Storage persistence operates on atomic canonical JSON store via `PepGrantStore` and `PepGrantService`.
   - Audit trail emission integrated via `classify_and_emit` / `dispatch::recorded_call`, appending tamper-evident cryptographic hash chain records into SQLite `$AIOSH_HOME/audit.db`.
   - Validated standard result envelopes (`code`, `data`, `error`) for both JSON and terminal human-readable output modes.
3. **Integration Smoke Verification**:
   - Extended `code/aiosh-cli/tests/test_pep_cli_smoke.py` to verify help discovery and end-to-end grant lifecycle operations through `aiosh` binary.

---

## 3. Verification Output

### 3.1 CLI Smoke Test Execution
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

### 3.2 Granular Unit & Integration Suite
```text
> python -m pytest code/aiosh-cli/tests/test_pep_grant_cli.py
============================= test session starts =============================
platform win32 -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
rootdir: C:\Users\OBSESSION\Desktop\AIOS_MERGED
plugins: anyio-4.14.2
collected 5 items

code\aiosh-cli\tests\test_pep_grant_cli.py .....                         [100%]

============================== 5 passed in 5.90s ==============================
```

---

## 4. Acceptance Confirmation
- [x] Feature reachable through production CLI surface (`aiosh pep grant`).
- [x] Integration smoke passes end-to-end with zero regressions.
- [x] Help catalog discoverability confirmed.
- [x] Cross-substrate audit row emission verified.
