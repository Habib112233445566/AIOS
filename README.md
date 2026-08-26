# AI-Native Operating System (AIOS)

**Status:** Research & Architecture Phase — v2 (course correction 2026-08-20)
**Version:** 2.0-draft
**Last updated:** 2026-08-20

## The product vision

> **AIOS = a Linux system for ethical hacking on the inside, a Windows-style desktop on
> the outside, with AI as a first-class S-rank kernel subsystem that controls
> the whole system.**

Three pillars back the vision:

| Pillar | Surface | Tier |
|---|---|---|
| **A — Linux ethical-hacking platform** | Inside (kernel + CLI + services) | Foundation |
| **B — Windows-like desktop** | Outside (the user sees this) | Surface |
| **C — AI as S-rank first-class kernel subsystem** | Cross-cutting (controls both) | S-tier |

The OS should feel like a **Kali install dressed in Windows skin, with the AI as
the operator actually in charge**.

## Mission

Research, design, specify, verify, implement, and maintain an AI-Native
Operating System that:

1. ships on a hardened Linux host (Debian 13 / Ubuntu 24.04 LTS) and inherits
   that ecosystem's stability,
2. presents a Windows-look Windows-feel desktop on top through KDE Plasma 6 and
   Wine / Proton, and
3. places an S-rank AI subsystem (pluggable inference: local llama.cpp →
   Ollama → remote API) at the apex of the capability stack, exposing every
   system capability through Model Context Protocol (MCP) tools governed by a
   Policy Enforcement Point (PEP) and an append-only audit ring.

## What is in this repository

```
docs/                                Research, architecture, specifications, ADRs, RFCs
docs/research/                       V1-XX operating-system theory (KEEP from v1)
docs/research-architecture/          AI-Native-OS architecture research (KEEP from v1)
docs/tasks/                          Canonical sequential task ledger (T-00001..T-10000)
docs/gui-pointer-research.md         GUI pointer research (Pillar B input)
docs/hardware-notes.md               Hardware notes
docs/research_cursor_lag.md          Notes from cursor-lag research
ci/                                  CI runners (run_all_smokes.sh)
code/aiosh-rust/                     **Primary implementation (Rust, since 2026-08-21):**
                                     aiosh-core (audit ring, classifier R-01..R-12, PEP,
                                     retention, pentest wrappers, sandbox, agent loop),
                                     aiosh-cli (`aiosh` binary — includes `aiosh task`
                                     ledger control), aiosh-mcp (MCP server),
                                     aiosh-sandbox (standalone Landlock+seccomp executor
                                     used by the legacy TS CLI `aiosh run`),
                                     ci/rust_smoke.sh (build + tests + MCP wire contract)
code/aiosh-cli/                      LEGACY TypeScript CLI + audit ring + retention
                                     (kept for cross-substrate reference, not the ship path)
code/aiosh-mcp/                      LEGACY Python MCP server + classifier + retention
                                     (kept for cross-substrate reference, not the ship path)
mostimportanAIfolder/                Master project state (blueprint, constitution,
                                     roadmap, knowledge/task graphs, manifest, etc.)
README.md                            You are here
START_HERE.md                        Project boot procedure
FREEBUFF_PROTOCOL.md                 Persistence protocol (workspace.py)
WORKSPACE.md                         Workspace rules
research-findings.md                 Findings ledger
findings.md                          Per-task findings
progress.md                          Time-ordered progress log
task_plan.md                         Active task plan
PROJECT_MANIFEST.yaml                Machine-readable project state
```

The pre-2026-08-20 `kernel/`, `src/`, `userland/`, `target/`, `tests/`, etc.
artifacts are **research substrate** for capability/IPC/scheduler studies and
inform the design but are not the shipping v2 target.

**Implementation language (2026-08-21): the shipping v2 stack is Rust.**
`code/aiosh-rust/` is the single source of truth for the MCP server, CLI,
audit ring, classifier, PEP, retention, pentest wrappers, sandbox, and agent
loop — `cargo build` is zero-warning and `cargo test` (45 tests) is green,
including the ported classifier fixture matrix (SC1..SC10) that locks
byte-identical behavior with the legacy TS/Python substrates. The legacy
`code/aiosh-cli/` (TypeScript) and `code/aiosh-mcp/` (Python) trees remain
in-repo as the reference contract; `ci/run_all_smokes.sh` runs the Rust
suite first, then the legacy suites to keep the cross-substrate invariant
honest.

## Governing documents

| Document | Description |
|---|---|
| `mostimportanAIfolder/AI_CONSTITUTION.md` v1.1 | Immutable engineering laws (highest authority) — ratified S-rank AI principles P-1..P-6 |
| `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` v2.0 | Forward plan, three pillars, S-rank AI subsystem |
| `mostimportanAIfolder/PRODUCT_ROADMAP.md` v2.0 | Phased plan to ship the vision |
| `mostimportanAIfolder/MASTER_PROJECT_EXECUTION_PROTOCOL.md` | How agents execute work |
| `mostimportanAIfolder/AI_BOOT_PROTOCOL.md` | How any agent picks up where the previous one stopped |
| `mostimportanAIfolder/RESEARCH_EXECUTION_PROTOCOL.md` | How to do research without hallucinating |

## Where we are now

- **Pillar A** ≈70% (off-the-shelf Kali/Parrot/BlackArch tooling reaches
  capability; MCP wrapping is the gap — five pentest wrappers shipped).
- **Pillar B** ≈60% (KDE Plasma + Wine/Proton + theming is configured; UX
  polish and the AI ↔ desktop hook is the gap).
- **Pillar C** ≈30% (classifier→PEP→audit gate + Ollama/stub Computer-Use
  agent loop shipped and verified 2026-08-21; audit-ring retention shipped
  same day — checkpointed segment rotation + bloom filters, ADR-0036;
  on-device inference is the remaining gap).

The end-to-end Pillar-C chain is now working and verified: user →
natural-language command (`aiosh agent`) → AI decomposition → MCP tool →
classifier/PEP gate → audited tool result (`tests/test_demo_smoke.py`,
D1/D2/D3). As of 2026-08-21 the whole chain is ported to **Rust**
(`code/aiosh-rust/`, verified via `code/aiosh-rust/ci/rust_smoke.sh`: build,
45 unit tests, MCP stdio wire contract, CLI status). The next phase hardens
this (full Kali tool taxonomy, real on-device inference) and adds the
Pillar-B desktop hook.

## Task ledger (read this before doing any work)

The canonical work queue is `docs/tasks/MASTER_TASK_LEDGER.jsonl`
(T-00001..T-10000, human-readable index in `docs/tasks/MASTER_TASK_LEDGER.md`).
The live pointer is `docs/tasks/TASK_STATE.json`.

**Rule: agents must read `docs/tasks/TASK_STATE.json` and execute ONLY
`next_task`.** When a task is done, run
`python3 tools/complete_task.py <id>` — it refuses out-of-order
completion and advances the pointer by exactly one. Never jump to task
56 or 89 while task 2 is unfinished. Every task carries its own
instructions and acceptance criteria; produce the listed evidence before
marking it complete. Run `bash ci/run_all_smokes.sh` before and after
any code change.

## How to keep going

1. Read `START_HERE.md` then run the AI Boot Protocol.
2. Read `research-findings.md` for prior research.
3. Read `docs/tasks/TASK_STATE.json` and work ONLY the `next_task`
   from `docs/tasks/MASTER_TASK_LEDGER.jsonl` (no skipping).
4. `mostimportanAIfolder/TASK_DATABASE.json` is a NON-authoritative
   graph-derived reconstruction — do not use it to pick work.

## Security

Found a vulnerability? See [`SECURITY.md`](SECURITY.md) — private
reporting via GitHub Security Advisories, 90-day coordinated
disclosure, and the index of all component security reviews.

## License

TBD — see governance artifacts.

---

## Citations (anchored, product v2)

- Model Context Protocol — Anthropic-published open standard for AI ↔ tools.
  Source: <https://modelcontextprotocol.io/introduction>
- AI agent + OS integration — Wikipedia, AI agent (2026). Source:
  <https://en.wikipedia.org/wiki/AI_agent>
- Kali tool taxonomy (MITRE ATT&CK ordering since v2025.2). Sources:
  <https://www.kali.org/tools/>, <https://en.wikipedia.org/wiki/Kali_Linux>
- KDE Plasma 6.7.4 (Aug 2026) — Windows-like theming, default Wayland.
  Source: <https://en.wikipedia.org/wiki/KDE_Plasma>
- Wine 11.0 (Jan 2026), Proton 11.0-1 (Jul 2026) — Windows binary compat
  on Linux. Sources: <https://en.wikipedia.org/wiki/Wine_(software)>,
  <https://en.wikipedia.org/wiki/Proton_(software)>
