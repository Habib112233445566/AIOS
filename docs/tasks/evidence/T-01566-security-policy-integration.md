# T-01566 — Filesystem Layout security policy: Integration

## Metadata
- **Task ID:** `T-01566`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** Complete — integrated security policy suite FL11 into the master test runner `tools/test_fs_layout_suites.py`; all criteria FL1..FL11 passing.
- **Date:** 2026-09-19
- **Depends on:** `T-01565` (Security Policy Unit Test)
- **Feeds:** `T-01567` (Security Review for Security Policy)
- **Artifacts:** `docs/tasks/evidence/T-01566-security-policy-integration.md`, `docs/tasks/evidence/T-01566-integration.md`

---

## 1. Integration Scope

1. Added `FL11` criterion to `tools/test_fs_layout_suites.py`:
   - Runs `code/aiosh-cli/tests/test_fs_layout_security_policy.py`.
   - Validates PEP gating, wrong tool scope, path confinement, positive authorized mutations, and read-only ungated tool execution.
2. Verified all 11 criteria in the test harness:
   - `FL1`: Data model & invariants
   - `FL2`: Core service (store, probe, diff, fstab, persistence)
   - `FL3`: CLI surface smoke & boundaries
   - `FL4`: CLI audit emission & escape-injection security proof
   - `FL5`: CLI & MCP in-tree unit tests
   - `FL6`: Cross-surface CLI <-> MCP integration parity
   - `FL7`: CLI hardening proof
   - `FL8`: MCP contract
   - `FL9`: Configuration contract (validation E-1..E-7, store parse)
   - `FL10`: Automated lifecycle & edge cases (A1..A8)
   - `FL11`: Security policy (P1..P5)

---

## 2. Test Execution Output

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
[+] FL11 filesystem layout security policy (P1..P5: PEP gating, wrong scope, path confinement, positive authorized mutation, read-only ungated access)

PASS: fs_layout_suites criteria (FL1..FL11)
```

---

## 3. Acceptance Confirmation

- [x] FL11 integrated into `tools/test_fs_layout_suites.py`.
- [x] Full test battery (FL1..FL11) executed and passing.
- [x] Sub-Epic 7 (Security Policy) integration milestone met.
