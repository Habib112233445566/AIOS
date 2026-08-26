# T-00040 — Task Ledger Control CLI surface: Verification & Evidence

**Date:** 2026-08-22
**Type:** verification & evidence (no code changed)
**Depends on:** T-00039 documentation
**Artifact note:** instruction path `T-00040-verify.md`; ledger row's
declared artifact (`…-verification-evidenc.md`) mirrored byte-for-byte.

## 1. Epic-specific suites — all PASS

```
$ cargo test (workspace)  → 13 CLI + 64 core = 77 passed; 0 failed (0 warnings)
$ python3 tools/test_task_ledger.py            → PASS: … (U1..U16)
$ python3 tools/test_task_ledger_scaffold.py   → PASS
$ python3 code/aiosh-mcp/tests/test_task_service_smoke.py → PASS (W1..W8)
$ python3 code/aiosh-cli/tests/test_task_cli_smoke.py     → PASS (C1..C9)
```

## 2. Full baseline smoke set — 12/12 PASS

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke            # build + 77 tests + wire + 4-way parity
PASS: classifier_smoke / mcp_smoke / task_service_smoke
PASS: pentest_smoke / sandbox_smoke / retention_smoke / demo_smoke
PASS: cli_bash_smoke
PASS: task_cli_smoke        # NEW: unified CLI validation in CI
PASS: task_ledger_unit / task_ledger_scaffold
== ALL 12 SMOKE SUITES PASS ==
```

## 3. State-file verification

```
pre-completion pointer : next_task = 40, completed = 1..39, seq = 39
post-completion pointer: next_task = 41, completed = 1..40, seq = 40
aiosh task check → {"ok":true,"total_tasks":10000}
```

Pointer advanced exactly one via the shipped surface under test
(`aiosh task done 40 --note … --evidence …`).

## 4. Milestone — CLI surface sub-epic CLOSED (T-00031..T-00040)

Research (4 empirically-probed defects vs the stricter MCP surface +
POSIX.1-2024 grounding) → specification (D1–D5 owner-locked) →
scaffold → implementation (single validation source via
`task_service::TaskCall`; `--` delimiter; dash-value rejection; caps;
per-subcommand help) → unit tests (13 Rust + C1..C9 wire) →
integration (CI 12/12) → security review (no open bypass) →
hardening (non-UTF-8 argv panic eliminated, lossy+audited) →
documentation (SPEC §8.1) → this verification.
`task_plan.md` / `progress.md` updated. Task Ledger Control now has
four of its ten components closed (data model, core service, CLI
surface complete; configuration component begins at T-00041).
