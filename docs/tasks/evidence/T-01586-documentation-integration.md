# T-01586 — Filesystem Layout documentation: Integration

## Metadata
- **Task ID:** `T-01586`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / documentation
- **Status:** Complete — integrated documentation suite FL13 into `tools/test_fs_layout_suites.py`; all criteria FL1..FL13 passing.
- **Date:** 2026-09-19
- **Depends on:** `T-01585` (Unit Test)
- **Feeds:** `T-01587` (Security Review)
- **Artifacts:** `docs/tasks/evidence/T-01586-documentation-integration.md`, `docs/tasks/evidence/T-01586-integration.md`

---

## 1. Integration Scope

1. Added `FL13` criterion to `tools/test_fs_layout_suites.py`:
   - Runs `code/aiosh-cli/tests/test_fs_layout_documentation.py`.
   - Validates D1 (CLI subcommands in `--help`), D2 (MCP manifest schema parity), D3 (evidence link integrity), D4 (JSON syntax validity), and D5 (error code documentation completeness).
2. Verified all 13 criteria in the master test runner:
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
   - `FL12`: Observability (O1..O5)
   - `FL13`: Documentation (D1..D5)

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
[+] FL12 filesystem layout observability (O1..O5: audit emission completeness, correlation & queryability, outcome fidelity, state inspection parity, destructive mutation flagging)
[+] FL13 filesystem layout documentation (D1..D5: CLI subcommands, MCP schema parity, evidence links, JSON syntax, error codes)

PASS: fs_layout_suites criteria (FL1..FL13)
```

---

## 3. Acceptance Confirmation

- [x] Feature reachable through its production surface.
- [x] Integration smoke passes end-to-end.
