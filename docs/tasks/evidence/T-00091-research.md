# T-00091 — documentation component: Research (evidence)

**Date:** 2026-08-22 · **No code changed** (research-only task).
Scope question: what does "documentation of Task Ledger Control" mean
here, what already exists, what is missing, and what must be decided
before scaffold/implementation.

## Facts — in-repo inventory (every item verified this session)

| Artifact | Verified state |
|---|---|
| `docs/SPEC-TASK-LEDGER.md` | 474+ lines; §1–§7 data model/operator guide; §8 core service + **six** per-component subsections `### 8.1..8.6` mapping to closed components (CLI T-31..40, MCP/API T-41..50, config T-51..60, tests T-61..70, security policy T-71..80, observability T-81..90); §9 evidence index table |
| Evidence files referenced by SPEC | ALL exist EXCEPT one intentional placeholder inside a copy-paste EXAMPLE command (`--evidence docs/tasks/evidence/x.md`, §8) — a naive link checker would false-positive here |
| `docs/README.md` | Task-ledger section names Rust surface, AIOSH_TASKS_DIR note, links SPEC; quick links resolve |
| `docs/tasks/MASTER_TASK_LEDGER.md` | Human index = law + phase map (10 phases) + detail for FIRST 25 TASKS ONLY; file is generation-owned ("immutable after generation; regenerate wholesale only" per SPEC §2) |
| `docs/tasks/GOALS.md` | Mission + NO-SKIP law + governing precedence |
| CI doc enforcement precedent | `tools/check_security_policy.py` (S1..S5): exists/no-TODO/pinned-URL/prose-floor/**in-tree link existence** — the repo's only machine-checked doc today |
| Count rot observed | Prose hardcodes suite counts ("CI 16/16", "15 suites") that went stale when metrics_smoke made it 17 — historical evidence texts are fine frozen, but living docs must not embed volatile counts |

## Facts — external anchors (fetched live 2026-08-22)

- **Diátaxis** (<https://diataxis.fr/>, fetched): four user needs → four
  forms: tutorials / how-to guides / technical reference / explanation;
  "documentation should be organised around the structures of those
  needs". Adopted by Cloudflare/Gatsby/Vonage per the page.
- Repo's own binding rule (Constitution Art. 14, START_HERE QUALITY):
  undocumented work is incomplete; every artifact documented.

## Assumptions (marked as assumptions — not facts)

- A1: "the documentation of Task Ledger Control" means the ledger's
  operator/agent-facing doc set (SPEC + index + GOALS + evidence),
  NOT general project docs (README/blueprint) — those belong to other
  epics. Basis: every prior epic scoped its components to its own surface.
- A2: docs-as-code enforcement should mirror the security-policy
  pattern (small deterministic Python checker + CI suite) rather than
  a heavy docs toolchain. Basis: repo precedent and zero-new-deps law.

## Decisions needed before implementation

- D1: Scope lock → operator/agent docs of THE LEDGER only (A1).
- D2: Add `tools/check_task_docs.py` invariant checker + wire as
  permanent CI suite. Candidate checks: (a) SPEC has one `### 8.x`
  section per CLOSED component; (b) every `docs/…` path referenced in
  SPEC §9 exists in-tree (excluding fenced-code examples like x.md);
  (c) MASTER_TASK_LEDGER.md phase map matches generator PHASES exactly;
  (d) no TODO markers in SPEC/index; (e) docs/README ledger links resolve.
- D3: Do NOT hand-edit `MASTER_TASK_LEDGER.md` (generation-owned);
  any change goes through `tools/generate_master_tasks.py`.
- D4: Living docs must derive counts dynamically or avoid them.

## Unknowns

- U1: whether future phases (1+) will want per-phase generated indexes —
  deferred; out of this component's scope either way.
