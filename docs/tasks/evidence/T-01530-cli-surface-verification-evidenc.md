# T-01530: Filesystem Layout - CLI Surface: Verification & Evidence

## Metadata
- **Task ID:** `T-01530`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface (`code/aiosh-rust/aiosh-cli::cmd_fs_layout`, `docs/filesystem_layout.md`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (10/10) — **CLI Surface COMPLETE**
- **Dependencies:** `T-01529` (CLI Surface Documentation)
- **Next Task:** `T-01531` (Filesystem Layout / MCP/API surface: Research)

---

## 1. Executive Summary & Verification Matrix

Task `T-01530` executes the full verification battery for the **Filesystem Layout CLI Surface**
sub-epic (`T-01521..T-01530`). It confirms the CLI subcommand surface, its standard envelopes and
exit codes, audit emission, cross-surface CLI ↔ MCP parity, input hardening, and the full baseline
smoke set with zero regressions — including the `--standard` preset selector wired in `T-01529`.

### 1.1 Test Battery Execution

```
======================================================================
1. FsLayout aggregate runner (criteria FL1..FL7)
======================================================================
$ python tools/test_fs_layout_suites.py
[+] FL1 filesystem layout data model integrity & invariants (FL1..FL5)
[+] FL2 filesystem layout core service (store, probe, diff, fstab, persistence)
[+] FL3 filesystem layout CLI surface smoke & boundaries (cmd_fs_layout)
[+] FL4 filesystem layout CLI audit emission & escape-injection security proof
[+] FL5 filesystem layout CLI in-tree unit test
[+] FL5 filesystem layout MCP in-tree unit test
[+] FL6 filesystem layout cross-surface CLI <-> MCP integration parity
[+] FL7 filesystem layout CLI hardening (non-regular paths, bounded reads, atomic persistence)

PASS: fs_layout_suites criteria (FL1..FL7)

======================================================================
2. Rust workspace unit & integration suites
======================================================================
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core
test result: ok. (30 targets) 582 passed; 0 failed; 0 ignored; 0 measured

$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 98.32s

$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.38s

======================================================================
3. Baseline master smoke suites
======================================================================
$ python tools/test_session_suites.py
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP in-tree unit test suite
[+] SB3 session MCP tool surface JSON-RPC smoke test (aios.session.*)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)
[+] SB5 session configuration resolution, invariants & precedence (SC1..SC7)
[+] SB6 session automated integration test suite (SBT1..SBT5)
[+] SB7 session security policy enforcement & invariants (SSP1..SSP7)
[+] SB8 session observability & telemetry metrics (SSO1..SSO6)
[+] SB9 session documentation architecture & operational guide (D1..D6)

PASS: session_suites criteria (SB1..SB9)

$ python tools/test_session_doc.py
[+] D1 doc existence and size bounds (26647 bytes)
[+] D2 all 9 required sections present
[+] D3 zero forbidden placeholders/markers
[+] D4 policy invariants, CLI commands, and MCP tools coverage complete
[+] D5 negative rejection assertions verified
[+] D6 zero volatile snapshot counts (C6 compliant)

PASS: test_session_doc criteria (D1..D6)

$ python code/aiosh-mcp/tests/test_session_mcp_smoke.py
PASS: tools/list contains all 6 aios.session.* tools
PASS: aios.session.validate (valid, invalid, boundary, missing)
PASS: aios.session.list & aios.session.get (filtering, inspection, error modes)
PASS: aios.session.action (PEP enforcement, lock, unlock, unknown action, missing params)
PASS: aios.session.create & persistence (PEP enforcement, create, get, duplicate rejection, invalid spec)
PASS: Cross-surface CLI <-> MCP parity & state sharing
PASS: aios.session.check validation and quarantine recovery
PASS: MCP session hardening (payload limits, query bounds, ID injection, store path sanitization)

ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!

$ python code/aiosh-cli/tests/test_service_cli_smoke.py
PASS: aiosh service check (healthy, JSON, corruption detection, --fix quarantine recovery)

ALL SERVICE CLI SMOKE TESTS PASSED!

======================================================================
4. Governance & evidence integrity
======================================================================
$ python tools/check_task_docs.py
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)

$ python tools/check_evidence.py
[+] E1 directory-health: found 3888 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 3888 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```

### 1.2 Changed-Surface Regression Check

The `T-01529` documentation task also wired the `--standard` selector, which `aiosh layout --help`
had advertised but the resolver ignored. That change is covered by the FL3 smoke criterion:

```
$ python code/aiosh-cli/tests/test_fs_layout_cli_smoke.py
PASS: aiosh layout --help and unknown subcommand (exit 2)
PASS: aiosh layout list (prose and JSON, presets present)
PASS: aiosh layout show (valid, preset selector, unknown id -> exit 1)
PASS: aiosh layout validate/check (valid, FL1 + pass boundary, malformed spec)
PASS: aiosh layout probe (viable, minimum boundary, below-minimum, invalid bytes)
PASS: aiosh layout diff (destructive, self-diff, unknown ids -> exit 1)
PASS: aiosh layout fstab (prose and JSON, six-field rows)
PASS: aiosh layout register/set-active/remove/import-fstab store lifecycle
PASS: aiosh layout argument boundaries (missing args, control chars, path length)

ALL FILESYSTEM LAYOUT CLI SMOKE TESTS PASSED!
```

Two assertions pin the flag: `show --standard` returns `aios-uefi-standard-v1`, and `show --standard
--store <store whose active layout is custom>` still returns `aios-uefi-standard-v1` (proving the
flag selects the preset rather than falling through to the active layout).

---

## 2. Milestone Conclusion & State Advances

- **Sub-Epic Status**: `Filesystem Layout / CLI surface` (10/10 tasks, `T-01521..T-01530`) **COMPLETE**.
- **State Updates**:
  - `task_plan.md` updated with the milestone entry.
  - `progress.md` updated with the chronological progress record.
- **Next Task**: `T-01531` (`Phase 1 — Linux Base System & Bootable Target / Filesystem Layout /
  MCP/API surface: Research`).

---

## 3. Acceptance Verification

- [x] **Full relevant suite green with captured output.** FL1..FL7, `aiosh-core` (582 tests across 30
      targets), `aiosh-cli` (24), `aiosh-mcp` (12), plus the SB/D baseline smoke set, service CLI
      smoke, and the C/E governance checks — all PASS, zero failures (§1.1).
- [x] **State files updated; next task pointer advanced.** `task_plan.md` and `progress.md` updated
      (§2); `TASK_STATE.json` advances to `T-01531` via `tools/complete_task.py`.
