# Documentation Index — v2 (2026-08-20 course correction)

> **v2.1 amendment (2026-08-21) — implementation language.** The shipping
> stack is now **Rust** (`code/aiosh-rust/`: aiosh-core + aiosh-cli +
> aiosh-mcp). The legacy TypeScript (`code/aiosh-cli`) and Python
> (`code/aiosh-mcp`) implementations are retained in-repo as the
> cross-substrate reference contract — they are NOT the ship path. See
> `../README.md` and `../findings.md` (2026-08-21 entry) for details.
>
> **v2 framing.** The product vision has been restated:
> *"a Linux system for ethical hacking on the inside, a Windows-style desktop
> on the outside, with AI as a first-class S-rank kernel subsystem that
> controls the whole system."* Three pillars drive every decision:
> - **Pillar A — Linux ethical-hacking platform** (foundation)
> - **Pillar B — Windows-like desktop** (user-facing surface)
> - **Pillar C — AI as S-rank first-class kernel subsystem** (control plane)
>
> See `../README.md`, `../mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0),
> and `../mostimportanAIfolder/PRODUCT_ROADMAP.md` (v2.0) for the canonical v2
> plan.

## Existing research sub-trees (preserved from v1)

The research volumes below informed the v1 product framing (a from-scratch
RISC-V microkernel as shipping target). In v2 they are preserved as
**research substrate** for our userspace capability/IPC/scheduler designs.
Their findings remain relevant and authoritative; they are no longer the
shipping-path definition.

| Directory | What it contains | v2 role |
|---|---|---|
| `research/` | 13 V1-XX files: operating-system theory, systems theory, computer architecture, security principles, AI theory, threat model, system ABI, memory protection, IPC, AI privilege model, research refresh, AI attack surface, AI kernel-safety primitives | Substrate: informs Pillar A, B, and C capability/IPC/PEP designs |
| `research-architecture/` | 27 AI-Native-OS architecture research files covering kernel, memory, networking, storage, virtualization, AI runtime, developer platform, security, observability, performance, hardware | Substrate: explicit OS-level inputs to all three Pillars |
| `gui-pointer-research.md` | GUI pointer research | Inputs Pillar B (Windows-like desktop) |
| `hardware-notes.md` | Hardware notes | Inputs Pillar A drivers and Pillar B input delivery |
| `research_cursor_lag.md` | Cursor-lag notes | Inputs Pillar B rendering / input |

## v2 critical-path artifacts (authoritative)

| Document | Authority |
|---|---|
| `../README.md` | Product pitch and v2 mission |
| `../mostimportanAIfolder/AI_CONSTITUTION.md` v1.1 | Highest — immutable engineering laws incl. ratified S-rank AI principles **P-1..P-6** |
| `../mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` v2.0 | Forward plan |
| `../mostimportanAIfolder/PRODUCT_ROADMAP.md` v2.0 | Phased plan |
| `../mostimportanAIfolder/AI_BOOT_PROTOCOL.md` v1.1 | How agents start work |
| `../mostimportanAIfolder/MASTER_PROJECT_EXECUTION_PROTOCOL.md` v1.1 | How agents execute work |
| `../mostimportanAIfolder/RESEARCH_EXECUTION_PROTOCOL.md` v1.1 | How agents research without hallucinating (with v2 citation anchors) |

## v2 citation anchors (refresh monthly)

- Kali tool taxonomy — <https://www.kali.org/tools/>
- Kali Linux distro — <https://en.wikipedia.org/wiki/Kali_Linux>
- Parrot OS — <https://en.wikipedia.org/wiki/Parrot_OS>
- BlackArch — <https://en.wikipedia.org/wiki/BlackArch>
- KDE Plasma — <https://en.wikipedia.org/wiki/KDE_Plasma>
- Xfce — <https://en.wikipedia.org/wiki/Xfce>
- Wayland — <https://en.wikipedia.org/wiki/Wayland_(display_server_protocol)>
- Wine — <https://en.wikipedia.org/wiki/Wine_(software)>
- Proton — <https://en.wikipedia.org/wiki/Proton_(software)>
- MCP introduction — <https://modelcontextprotocol.io/introduction>
- AI agent / OS integration — <https://en.wikipedia.org/wiki/AI_agent>
- Anthropic — <https://en.wikipedia.org/wiki/Anthropic>

## Task ledger (Sprint 3+)

The canonical work queue lives in `tasks/`:

| File | Role |
|---|---|
| `tasks/MASTER_TASK_LEDGER.jsonl` | 10,000 sequential tasks (T-00001..T-10000), each with goal, instructions, acceptance criteria |
| `tasks/MASTER_TASK_LEDGER.md` | Human index: phase map + first 25 tasks in detail |
| `tasks/TASK_STATE.json` | Live pointer: `next_task` is the ONLY task allowed to start |
| `tasks/GOALS.md` | Mission, governing-doc precedence, and the NO-SKIP execution law |
| `tasks/evidence/` | Per-task evidence (`T-NNNNN-*.md`) |

**Rule: agents must read `tasks/TASK_STATE.json` and execute only
`next_task`.** Complete a task with the Rust shipping surface
`aiosh task done <id> --note "…"` (export `AIOSH_TASKS_DIR="$PWD/docs/tasks"`
first — see `SPEC-TASK-LEDGER.md` §7 L2) or the legacy
`python3 tools/complete_task.py <id>` — both refuse out-of-order
completion and advance the pointer by exactly one. Never skip ahead.
Agents may also drive the ledger over MCP via the **`aios.task`** tool
(read-only `status`/`check`/`validate`; mutations require a PEP grant) —
operator reference and copy-pasteable calls: `SPEC-TASK-LEDGER.md` §8.
Integrity drift check: `aiosh task validate` (read-only, report-only —
`SPEC-TASK-LEDGER.md` §9).
Data model, command reference, and known limitations:
**`docs/SPEC-TASK-LEDGER.md`** (T-00019/T-00029).
`../mostimportanAIfolder/TASK_DATABASE.json` is a
NON-authoritative graph-derived reconstruction; do not use it to pick work.

## CI Smoke Orchestration (T-00111..T-00120)

`bash ci/run_all_smokes.sh` delegates to **`tools/ci_run.py`**, which
executes the suite registry `tools/ci_suites.py` — the single source of
suite order (order IS contract; suites share rebuild state, never
parallelize). Additions over the legacy bash runner: per-suite wall-clock
**timeouts** with process-group kill, and an atomic machine-readable run
summary.

```bash
# Run full CI with a custom summary location:
AIOSH_CI_RESULTS=/tmp/ci-summary.json bash ci/run_all_smokes.sh

# Consume the result programmatically:
python3 -c "import json;d=json.load(open('/tmp/ci-summary.json'));\
print(d['all_pass'], [(r['suite'],r['status'],r['duration_ms']) for r in d['results']])"
```

Summary schema (stable additive-only key set):
`{tool, schema_version, started_at, finished_at, total, passed, failed,
all_pass, results:[{suite,index,status,exit_code,duration_ms,started_at,
finished_at,log_path}]}`; status ∈ pass|fail|timeout|error.

Limitations (honest): timeout kills the process GROUP, but a suite that
double-forks can still escape it; log files under /tmp are uncapped on
disk (orchestrator memory exposure is bounded to a 64 KiB tail); the
summary is advisory telemetry — the exit code remains the CI verdict.
Evidence: `tasks/evidence/T-00111-research.md` …
`T-00120-data-model-verification-evidenc.md`.
### CI Summary Service (T-00121..T-00130)

The core service parses and validates the machine-readable summary output of ci_run.py. To align with the v2.1 shipping stack mandate, this service is implemented natively in Rust under iosh-core and exposed via the iosh ci CLI command. It serves as a strict gating mechanism that validates artifact schema, arithmetic coherence, and bounds limits, effectively sealing the CI run.

`ash
# Validate the CI output artifact explicitly (defaults to /tmp/aiosh-ci-results.json)
aiosh ci check

# Display a human-readable run report
aiosh ci show

# List only the failing suites and their log paths
aiosh ci failures
`

Limitations (honest): the read operation implements bounded retries to handle orchestrator lock contention but assumes a final artifact will eventually exist. The JSON payload is read completely into memory (capped at 1MB to prevent OOM) rather than streamed. 
Evidence: 	asks/evidence/T-00121-research.md .. T-00130-verification.md (and intermediate evidence like 	asks/evidence/T-00126-core-service-integration.md and 	asks/evidence/T-00128-core-service-hardening.md).


## Documentation invariants (Task Ledger Control, T-00091..T-00100)

`tools/check_task_docs.py` keeps THIS doc set rot-proof. Read-only,
stdlib-only, exit 0/1 — runs in CI as `task_docs_unit` +
`task_docs_scaffold` and standalone:

```bash
python3 tools/check_task_docs.py
# [✓] C1 spec-health        SPEC exists, marker-free
# [✓] C2 component sections ### 8.1..8.6 keep their frozen epic ranges
# [✓] C3 referenced paths   backticked docs/code/ci/tools paths resolve
#                           (fenced blocks + example x.md excluded)
# [✓] C4 phase map          MASTER_TASK_LEDGER.md table == JSONL phases
# [✓] C5 index health       marker-free docs; links stay inside checkout
# [✓] C6 no volatile counts living docs never embed "CI n/n" snapshots
# PASS: task docs criteria (C1..C6)
```

Limitations (honest): structural checks only — prose quality is human
judgment; C2's frozen section list grows monotonically when a new
component closes (add one entry + its range); C5's containment boundary
is the repo root (`../` links into the tree are fine, escapes are
flagged); reads are capped at 16 MiB; deliberately NOT exposed over
MCP (operator surface only).

Evidence chain: research `tasks/evidence/T-00091-research.md` · spec
T-00092 · scaffold T-00093 · implementation T-00094 · unit tests
T-00095 · integration T-00096 · security T-00097 · hardening
T-00098 · verification T-00100.

## Quick links

- [`../START_HERE.md`](../START_HERE.md)
- [`../PROGRESS_LOG`](../progress.md)
- [`../TASK_PLAN`](../task_plan.md)
- [`tasks/TASK_STATE.json`](tasks/TASK_STATE.json) — live task pointer
- [`tasks/MASTER_TASK_LEDGER.md`](tasks/MASTER_TASK_LEDGER.md) — task index
