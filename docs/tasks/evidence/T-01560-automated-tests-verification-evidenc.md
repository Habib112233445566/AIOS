# T-01560 — Filesystem Layout automated tests: Verification & Evidence

## Metadata
- **Task ID:** `T-01560`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — Sub-Epic 6 (Filesystem Layout Automated Tests, T-01551..T-01560) verified across all criteria FL1..FL10, Cargo suites, and ledger validation; milestone closed.
- **Date:** 2026-09-19
- **Depends on:** `T-01559` (Automated Tests Documentation)
- **Feeds:** `T-01561` (Security Policy Research)
- **Artifacts:** `docs/tasks/evidence/T-01560-automated-tests-verification-evidenc.md`, `docs/tasks/evidence/T-01560-verify.md`

---

## 1. Test Execution Evidence

### 1.1 `python tools/test_fs_layout_suites.py`
```
[+] FL1 filesystem layout data model integrity & invariants (FL1..FL5)
[+] FL2 filesystem layout core service (store, probe, diff, fstab, persistence)
[+] FL3 filesystem layout CLI surface smoke & boundaries (cmd_fs_layout)
[+] FL4 filesystem layout CLI audit emission & escape-injection security proof
[+] FL5 filesystem layout CLI in-tree unit test
[+] FL5 filesystem layout MCP in-tree unit test
[+] FL6 filesystem layout cross-surface CLI <-> MCP integration parity
[+] FL7 filesystem layout CLI hardening (non-regular paths, bounded reads, atomic persistence)
[+] FL8 filesystem layout MCP contract (advertised schema, audit target, destructive verdict, grant scope.paths confinement and canonical alias matching, nested-injection refusal)
[+] FL9 filesystem layout configuration contract (CLI wire suite: T-01542 validation rules E-1..E-7, ordering, fail-closed refusals, T-01544 store parse contract)
[+] FL10 filesystem layout automated lifecycle & edge cases (A1..A8: state machine, protection, fstab, probe, diff, corruption, audit)

PASS: fs_layout_suites criteria (FL1..FL10)
```

### 1.2 `code/aiosh-cli/tests/test_fs_layout_automated_cases.py` (Standalone)
```
Running FL10 Automated Cases...
  [+] A1: Full lifecycle state machine PASS
  [+] A2: Built-in protection PASS
  [+] A3: Active layout protection PASS
  [+] A4: fstab import & generation roundtrip PASS
  [+] A5: Probe capacity feasibility PASS
  [+] A6: Differential analysis destructive detection PASS
  [+] A7: Corrupt store tamper resistance PASS
  [+] A8: SQLite WAL audit trail emission PASS
All FL10 automated cases passed successfully!
```

---

## 2. Milestone Closure Summary: Sub-Epic 6 (T-01551..T-01560)

Sub-Epic 6 concludes with all 10 tasks satisfied:
1. `T-01551`: Automated Tests Research — established prior art and gap analysis.
2. `T-01552`: Automated Tests Specification — defined test cases A1..A8 and criterion FL10.
3. `T-01553`: Automated Tests Scaffold — scaffolded `test_fs_layout_automated_cases.py`.
4. `T-01554`: Automated Tests Implementation — delivered all 8 automated lifecycle test cases.
5. `T-01555`: Automated Tests Unit Test — verified standalone execution and negative/boundary cases.
6. `T-01556`: Automated Tests Integration — wired FL10 into `test_fs_layout_suites.py`.
7. `T-01557`: Automated Tests Security Review — audited subprocess safety and path confinement.
8. `T-01558`: Automated Tests Hardening — bounded timeouts, temporary directory isolation.
9. `T-01559`: Automated Tests Documentation — updated `docs/filesystem_layout.md`.
10. `T-01560`: Automated Tests Verification & Evidence — full battery green; milestone closed.

Next pointer advances to **T-01561** (`Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy: Research`).

---

## 3. Acceptance Confirmation

- [x] Full relevant suite green with captured output.
- [x] State files updated; next task pointer advanced.
