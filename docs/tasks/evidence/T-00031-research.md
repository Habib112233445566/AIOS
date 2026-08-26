# T-00031 — Task Ledger Control: CLI surface Research

**Date:** 2026-08-22
**Type:** research (no code changed)
**Depends on:** T-00030 (core service sub-epic closed)
**Artifact note:** instruction name `T-00031-research.md`; ledger row
declares `T-00031-cli-surface-research.md` (mirrored byte-for-byte).

Central question: the `aiosh task` CLI **already shipped** (T-00016
integration, T-00018 hardening). What — if anything — is genuinely
missing from this component, now that a second, stricter surface (the
`aios.task` MCP tool) exists over the same state machine?

---

## 1. Internal facts (read + verified empirically on scratch ledgers)

| # | Fact | Anchor / evidence |
|---|---|---|
| F1 | CLI exposes all 7 subcommands (`status/done/block/unblock/skip/rebuild/check`); EVERY outcome — including usage errors — writes an honest `task.ledger` audit row; envelope `{ok, subcommand, data|error}`; exit 0 ok / 1 refused-or-error | `aiosh-cli/src/main.rs::cmd_task` |
| F2 | Core machinery is already unified with the MCP path: resolver (D3), rebuild replay (D4), bounded lock wait (T-28), file size caps (T-18) all live in `ledger.rs`, which both surfaces call. No logic drift possible at that layer. | `aiosh-core/src/ledger.rs` |
| F3 | **Q1 (verified):** bare `aiosh task` → JSON usage error, exit 1, audited; cosmetic trailing space in the label (`"subcommand": "task "`). | probe 2026-08-22 |
| F4 | **Q2 (verified):** CLI `done` accepts an EMPTY note (stored `note: ""`) while the MCP surface requires non-empty → two validation truths for one state machine. | probe: stored note `''` |
| F5 | **Q3 (verified):** no length caps on CLI text flags — a 5000-char note stored verbatim vs schema cap 4096; evidence list unbounded count/length vs maxItems 16. | probe: stored len 5000 |
| F6 | **Q4 (verified):** flag-value ambiguity — `block 3 --reason --note` stores `reason = "--note"` (a flag token consumed as a value). | probe: event reason `'--note'` |
| F7 | **Q5:** no `--` end-of-options delimiter; options are placed AFTER operands (`done <id> --note x`) — GNU-style deviation from POSIX G9. | code read |
| F8 | **Q6:** help is a single top-level line mentioning `task`; there is no per-subcommand help. | `main.rs` dispatch match |
| F9 | Evidence values collected as "everything after `--evidence` until next flag" — array-shaped like MCP `evidence[]`, but unbounded. | `cmd_task` |

## 2. External authoritative facts (fetched live 2026-08-22)

Source: IEEE Std 1003.1-2024 / Open Group Base Specifications Issue 8,
XBD ch. 12 *Utility Conventions* —
<https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap12.html>

| # | Guideline | Applicability to `aiosh task` |
|---|---|---|
| E1 | G6 — option and option-argument should be separate arguments | ✅ compliant today |
| E2 | G7 — option-arguments should NOT be optional | ❌ `--note` at end of line silently becomes empty (F4); missing value is never an error |
| E3 | G10 — first `--` delimiter ends options | ❌ unsupported (F7) |
| E4 | G14 — dash-prefixed tokens identified as options should be treated as such | ❌ violated by Q4/F6 (flag token swallowed as a value) |
| E5 | G9 — options should precede operands | ⚠️ deliberate GNU-style deviation (operands-first); acceptable **if documented** — guidelines explicitly allow documented deviations |
| E6 | Numeric operands are decimal; out-of-range must be reported as range errors, not syntax errors | ✅ spirit-matched: u64 parse failure yields "requires <task_id>" refusal (audited) |

## 3. Gap analysis

The core logic cannot drift (F2). The **surface validation can and
does** drift (F4/F5/F6): the MCP tool refuses what the CLI accepts.
Since both write the same append-only history, permissive CLI input
(empty notes, uncapped sizes, flag-eating) becomes permanent,
cross-substrate-visible content that the stricter surface would have
rejected at the gate.

### Candidates

| Candidate | Verdict |
|---|---|
| **A. Route `cmd_task` through `task_service::TaskCall`** (parse_args-equivalent + validate + execute_with), keeping human-facing ergonomics (flag placement flexibility) but inheriting identical validation semantics | **Recommended proposal (AIOS-specific)** — one validation source; surfaces cannot diverge again |
| B. Patch `flag_after` only (reject dash-leading values, enforce caps inline in cmd_task) | Weaker — validation duplicated a third time; drift returns |
| C. Document divergence, change nothing | Rejected — empty-note completions are permanently recorded noise; violates single-truth principle |

## 4. Assumptions (marked, not facts)

- A1: no operator scripts depend on empty `--note` or >4096-char notes
  (behavior change under Candidate A). Mitigation: changelog entry in
  docs task; error messages explicit.

## 5. Decisions needed before Specification (T-00032)

- **D1:** adopt Candidate A (unify CLI validation via `task_service`)?
  Includes making empty `--note` a CLI refusal (parity with MCP).
- **D2:** reject dash-leading flag values explicitly (`--reason --x`
  → usage error) and support `--` delimiter (E2/E3/E4)?
- **D3:** add per-subcommand help (`aiosh task help`) + fix the
  `"task "` trailing-space label (F3)?
- **D4:** enforce evidence maxItems 16 + item length caps on CLI (F9)?
- **D5:** document the options-after-operands deviation as intentional
  (E5) in the spec, rather than re-ordering the grammar?

## 6. Acceptance check

- [x] Facts (F1–F9 internal with probes, E1–E6 external with citation)
      separated from assumptions (A1).
- [x] Citations include fetch date; source authoritative (IEEE/Open Group).
- [x] Unknowns and decisions listed explicitly (D1–D5).
- [x] No code changed in this task.
