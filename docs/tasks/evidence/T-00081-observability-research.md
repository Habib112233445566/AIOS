# T-00081 — Task Ledger Control: observability Research

**Date:** 2026-08-22
**Type:** research (no code changed)
**Depends on:** T-00080
**Artifact note:** instruction name `T-00081-research.md`; ledger row
declares `T-00081-observability-research.md` (mirrored).

Central question: can an operator answer *"is the ledger healthy?"*
today, and if not, what single surface should this component add?

## 1. Internal facts (probed 2026-08-22)

| # | Fact |
|---|---|
| F1 | **No consolidated health surface exists**: grep for health/metrics/observability across aiosh-rust returns nothing. |
| F2 | `aiosh status` covers the audit ring only (verify_ok, rows, head_hash) + version/constitution — zero ledger-specific signal (no pointer progress, blocked/skipped, event stats, hygiene). |
| F3 | Answering "is the ledger healthy?" requires THREE separate calls today: `task status` (pointer), `task check` (invariants), `task config` (knobs) — plus manual file inspection for stale tmps/lock. |
| F4 | `COMPLETIONS.jsonl` timestamps enable velocity (inter-completion intervals) — unexposed. |
| F5 | The audit ring already records every `task.ledger` outcome (ok/refused/error) — an error-rate signal exists but is unexposed. |
| F6 | File sizes vs the (now configurable, T-00054) caps = a saturation signal — unexposed. |

## 2. External authoritative facts (fetched live 2026-08-22)

Source: Google SRE Book, ch. 6 *Monitoring Distributed Systems* —
<https://sre.google/sre-book/monitoring-distributed-systems/>

| # | Principle | Application |
|---|---|---|
| E1 | **Four golden signals**: latency, traffic, errors, saturation | Maps directly (see §3) |
| E2 | Symptoms vs causes: monitor what's broken first | Health view = symptoms (pointer/invariants/hygiene); causes live in audit rows |
| E3 | "As simple as possible, no simpler" — signals unused by dashboards/alerts are removal candidates | One read-only command, no new metrics stack, no Prometheus export |

## 3. Golden-signal mapping for the ledger

| Signal | Ledger analog | Source (existing, unexposed) |
|---|---|---|
| Traffic | completions count + event seq | `COMPLETIONS.jsonl` |
| Errors | `task.ledger` audit rows by outcome (refused/error rate) | audit ring |
| Saturation | ledger/events/state file sizes vs configured caps | file metadata + `LedgerConfig` |
| Latency (velocity) | mean + recent inter-completion intervals | event `ts` deltas |
| (+ symptoms) | invariants ok? stale tmps? lock held? next_task set? | `assert_ledger_invariants`, fs |

## 4. Candidates

| Candidate | Verdict |
|---|---|
| **A. Read-only `aiosh task health` subcommand** — one consolidated JSON envelope (progress/velocity/errors/saturation/hygiene) reusing ledger+audit+config modules | **Recommended (AIOS-specific)** |
| B. Prometheus/metrics-file export | Rejected — E3: no metrics stack exists; would be a signal nobody consumes |
| C. Status quo (three calls + manual ls) | Rejected — the question is unanswerable in one step (F3) |

## 5. Assumptions (marked)
A1: velocity over full history + last-3 intervals is meaningful for a
sequential-pointer ledger (no parallel completions by design).

## 6. Decisions needed before Specification (T-00082)

- **D1:** scope = `aiosh task health` CLI subcommand only (read-only)?
  (default yes; MCP exposure deferred — operator surface, parity note)
- **D2:** golden-signal field mapping per §3? (default yes)
- **D3:** health runs audited like every task subcommand? (default yes)
- **D4:** velocity = full-history mean + last-3 intervals? (default yes)
- **D5:** include effective-config summary (reuse `to_json_with_sources`)?
  (default yes)
- **D6:** hygiene = invariant check + stale-tmp count + lock-file
  presence (never blocking)? (default yes)

## 7. Acceptance check
- [x] Facts vs assumptions separated; external citation w/ date.
- [x] Decisions listed explicitly. [x] No code changed.
