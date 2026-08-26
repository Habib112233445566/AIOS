# T-00123 — CI Smoke Orchestration / core service: Scaffold

Date: 2026-08-23 · Status: SCAFFOLD COMPLETE

## What was added

New module `tools/ci_service.py` (stdlib only, imports `ci_suites` — no
redefinition of shared shapes):

| Element | State at scaffold |
|---|---|
| `resolve_path(explicit)` | implemented (pure precedence: `--file` > `AIOSH_CI_RESULTS` > default) |
| `load_summary(path)` | loud-fail stub ("scaffolded (T-00123); lands in T-00124") |
| `failure_rows(summary)` | loud-fail stub |
| `human_report(summary)` | loud-fail stub |
| CLI (`main`) + argparse-style hand parser | wired: actions `show/failures/check`, `--file PATH`; usage errors exit 2 naming the token; load errors already routed to the exit-2 stderr envelope; `check` action raises loudly until T-00124 |

## Call-site referencing (acceptance #2)

The CLI is a live call site for every interface; verified by invocation:
- `python3 tools/ci_service.py show` → reaches `load_summary` stub →
  loud NotImplementedError traceback (fail-loudly contract).
- `python3 tools/ci_service.py` → `missing action`, usage line.
- unknown action → exit 2.

## Build/import verification

- Module imports cleanly (only dependency: sibling `ci_suites`).
- No existing file modified; zero regression surface
  (`tools/test_task_ledger.py` U1..U16 still PASS).

## Notes

- Hand-rolled arg loop instead of argparse to keep exit-code semantics
  exact per spec §6 (argparse's exit code is 2 but its messages/streams
  differ; spec pins stream + wording).
