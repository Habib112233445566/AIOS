# ADR-0035 — AIOS S-rank Agent Architecture

| Field | Value |
|---|---|
| **Status** | Accepted (binding) |
| **Date** | 2026-08-20 |
| **Authors** | AIOS planning session |
| **Replaces** | — |
| **Supersedes** | — |
| **Index** | This document is entry **#35** in the project's ADR numbering. ADRs #0001–#0034 covered the RISC-V microkernel substrate and were retired 2026-08-20 (`gc --execute` cleanup). This ADR re-anchors the numbering on the v2 framing (Linux-hacking inside, Windows desktop outside, AI as **S-rank kernel subsystem**). |
| **Companion docs** | `mostimportanAIfolder/AI_CONSTITUTION.md` (v1.1.5), `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0 §4), `mostimportanAIfolder/PRODUCT_ROADMAP.md` (v2.0 Phase S1..S5), `docs/research/AIOS-SUPERINTELLIGENCE-2026-08-20.md`, `docs/research/AIOS-DYNAMIC-NEURAL-TOPOLOGY-2026-08-20.md`, `docs/research/AIOS-V2-RESEARCH-2026-08-20.md`. |

---

## 1. Context

The three v2 research documents in `docs/research/` reached the same conclusion from three different angles: an operating system that wants "AI as a first-class kernel subsystem" cannot achieve that goal by bolting a `chat.openai.com` widget to a dock. The integration has to be **architectural**, not cosmetic, and the constituent subsystems (the agent loop, the tool surface, the memory, the value-alignment substrate) each need a binding commitment before code is written, because changing them later is exponentially more expensive.

This ADR records those commitments as **D-1..D-6**. Collectively they operationalise the S-rank principles P-1..P-6 in `AI_CONSTITUTION.md` v1.1.5 and the Pillar C subspec in `AIOS_MASTER_BLUEPRINT.md` v2.0 §4.

> **Terminology note:** "**S-rank**" (S for *Subsystem*) refers to the same role this OS attributes to scheduling, IPC, FS, drivers, and memory — peer-of-the-kernel subsystems, not apps. In particular it does **not** claim "superintelligent" in the Bostrom/Chalmers sense; see §6.

---

## 2. Decision

### D-1 — Adopt the Anthropic Computer Use agent loop as the canonical AIOS agent loop

Every AI subprocess inside AIOS — from shell assistants to the GUI session steward to the pentest operator — runs the same loop:

```
Observe (perceive environment) → Think (reason with LLM) →
Act (call MCP tool) → Loop (re-observe to verify outcome)
```

**Why binding:** any variation in the loop (different prompting scheme per agent, different tool-format, different retry policy) makes audit, debugging, and Constitution compliance **per-agent** instead of **system-wide**. The Computer Use implementation by [Anthropic](https://docs.anthropic.com/en/docs/agents-and-tools/computer-use) is the most thoroughly battle-tested open-loop available.

**Rejected alternatives:**
- *LangChain ReAct* — same shape but per-library implementation drift; not OS-grade.
- *Open Interpreter* — local-only, lacks the explicit "re-observe before acting again" guarantee in Computer Use.
- *Hand-rolled custom loop* — no auditability from day one.

### D-2 — MCP is the **only** tool-call protocol inside the kernel surface

All kernel subsystems expose their capabilities to the AI subsystem through a single **Model Context Protocol (MCP)** server. There are no side channels, no direct syscalls-from-prompt, no `tool_use` extensions.

Spec reference: [`modelcontextprotocol.io/introduction`](https://modelcontextprotocol.io/introduction), [`/docs/concepts/architecture`](https://modelcontextprotocol.io/docs/concepts/architecture).

Three MCP primitives only:
- **Tools** (what the model can call),
- **Resources** (what the model can read),
- **Prompts** (curated templates).
No other RPC mechanism is permitted for kernel→AI traffic. Transports are restricted to **STDIO** (in-process) for kernel-resident tools and **Streamable HTTP** for cross-host tools (e.g. remote Kali pod).

**Current MCP spec version:** `2026-07-28`. The deprecated **Sampling** primitive is removed from our MCP manifests from day one (a deliberate tightening tighter than the public spec — see §5).

**Why binding:** you cannot reason about security of "AI calling kernel tools" if there are multiple protocols. Single protocol = single PEP gate = single audit row format.

**Rejected alternatives:**
- *Native function-calling per LLM provider (OpenAI, Anthropic tool_use, Gemini function_calling)* — three formats, three PEP gates, three audit-row schemas.
- *gRPC* — fine for component-to-component traffic but was designed for service-to-service, not model-to-tool RPC.
- *Custom JSON schema* — invented today, drift tomorrow.

### D-3 — Skill library uses Voyager-style growth with AlphaEvolve-style promotion gate

The skill library (the long-term procedural memory of the agent subsystem) **grows** over time, but every addition has to survive an **evaluation function before promotion to durable memory**.

- **Shape borrowed from Voyager** (Wang et al., MIT, 2023) — a "skills library" as the long-term memory of an LLM-driven agent that explores an environment; the original Voyager demonstrated this on a Minecraft crafting curriculum ([`arxiv.org/abs/2305.16291`](https://arxiv.org/abs/2305.16291), Wikipedia: [Voyager (AI)](https://en.wikipedia.org/wiki/Voyager_(AI))).
- **Promotion gate borrowed from AlphaEvolve** (DeepMind, 2025) — the system is required to formalise an **evaluation function** for every candidate code/skill change before evolution is allowed to mutate it ([`en.wikipedia.org/wiki/AlphaEvolve`](https://en.wikipedia.org/wiki/AlphaEvolve)). The lesson: open-ended generation without an evaluator drifts to entropy.

**Operationalisation in AIOS:**
- Three named roles: `genesis` (proposes new skills), `steward` (runs each skill in a sealed, instrumented testbed), `librarian` (commits successful skills to the durable store and writes an audit row).
- **No skill is durable without a recorded evaluation function.** A skill that performs well on heuristics alone stays in a `provisional` shelf; it graduates only after the steward publishes a quantitative `eval_metric(name, target, threshold)` tuple.
- An audit ring row records every skill promotion: `(skill_id, eval_function_hash, pass_rate, parent_skills)`.

**Why binding:** removing this gate is the most common pattern in failed self-improving agent projects — they call it "the goal drift problem" after enough iterations. The gate is the technical instantiation of Constitution P-6 ("any change to the AI subsystem must be auditable end-to-end before it is committed durable").

**Rejected alternatives:**
- *Fixed skill library forever* — kills long-horizon capability gain.
- *Promotion by human approval only* — does not scale to multi-day engagements.
- *Promotion by LLM-as-judge* — too sycophancy-prone without a numeric gate.

### D-4 — Constitution C-1..C-4 are **lexically binding** on the agent loop

The four cautions added to `AI_CONSTITUTION.md` v1.1.5 §1.4 are **not advisory**. They are checked before every MCP tool dispatch:

| # | Caution (verbatim from Constitution) | Concrete prompt-injection vector it guards |
|---|---|---|
| C-1 | "AI may only operate Pillar-A (pentest) tools in an engagement designated by a real human user, scoped to that engagement's PEP rules." | Stops a Midnight Blizzard-style indirect-prompt-injection attack that asks the AI to scan arbitrary networks. |
| C-2 | "AI owns nothing on the user's desktop; the user always retains override authority and the right to revert any AI-mediated change within a grace window." | Stops the AI from auto-pinning apps, changing default wallpapers, or installing persistent daemon tools without consent. |
| C-3 | "AI must request granular consent before any tool call whose effects are non-reversible (rm, dd, write to existing FS, network exfil, decommissioning a process)." | Stops agentic "agent failure mode" chains (Auto-GPT-class one-shots). |
| C-4 | "AI must produce an audit row for every consequential action, even if the action itself succeeds." | Stops "shadow LLM" behaviour where a perfectly competent tool-call has no audit trail, defeating post-hoc incident response. |

**Mechanism:** the MCP server's pre-flight hook reads the active Constitution revision from `/etc/aios/active_constitution.sha256`, hashes the LLM output against that revision, and **refuses** the dispatch if any C-1..C-4 rule is triggered. The refusal row is itself audited.

**Why binding:** without this lexical binding the Constitution is documentation, not enforcement. The Microsoft "Sydney" persona-confusion incident (February 2023, [Wikipedia: Copilot (Microsoft)](https://en.wikipedia.org/wiki/Copilot_(Microsoft))) is the cautionary case for treating any constitution as cheap-to-bypass.

### D-5 — Training (when we fine-tune) uses **Constitutional AI**, not generic RLHF

When AIOS funds or hosts a fine-tune of any local model (Llama, Hermes, Mistral, custom), value-alignment training must use **Constitutional AI** methodology (Anthropic 2022, [en.wikipedia.org/wiki/Constitutional_AI](https://en.wikipedia.org/wiki/Constitutional_AI)): an LLM critiques and revises outputs against a written, versioned set of principles (our `AI_CONSTITUTION.md`), then RL is done on the revised distribution.

**Why binding:** RLHF on human preferences alone bakes in preference bias, sycophancy, and reward-hacking at scale. CAI ties the behaviour to a **document we can read and version-control** instead of to opaque rater preferences.

**Rejected alternatives:**
- *RLHF on user thumbs-up alone (default in closed-source commercial LLMs)* — opaque, undocumentable, raters also indirect-prompt-injected.
- *RLAIF (RL from AI feedback)* — partial solution; suffers the same critique-target stability problem unless coupled with a written constitution.
- *No training, prompts only* — fine for a single user session; insufficient for a kernel subsystem expected to operate multi-day engagements.

**Honest scope note:** this decision binds **when** we train. We currently operate prompt-only at OS level; CAI is the methodology queued for the first on-device fine-tune. Tan et al. 2023 ([`arxiv.org/abs/2212.08073`](https://arxiv.org/abs/2212.08073)) and follow-up RLAIF work ([`arxiv.org/abs/2309.00267`](https://arxiv.org/abs/2309.00267)) are the primary references we cite in training-market decisions.

### D-6 — Dynamic neural topology per query is held as a **design aspiration**, not a current capability

The pitch "millions of neurons interconnect in configurations adapted specifically to your problem" is technically adjacent to several published primitives but **no production system today retopologises millioscale networks per query**.

**Closest real primitives we will borrow from** (all cited in `docs/research/AIOS-DYNAMIC-NEURAL-TOPOLOGY-2026-08-20.md`):
- **Mixture of Experts** ([Wikipedia](https://en.wikipedia.org/wiki/Mixture_of_experts)) — per-input sparse activation (Jacobs 1991, Shazeer 2017, Fedus 2021 Switch Transformer).
- **Neural Architecture Search** ([Wikipedia](https://en.wikipedia.org/wiki/Neural_architecture_search)) — DARTS and friends generate topology per task (Zoph & Le 2017).
- **Liquid Time-Constant Networks** ([Wikipedia](https://en.wikipedia.org/wiki/Liquid_time-constant_networks)) — time-continuous adaptive dynamics (Hasani et al., MIT CSAIL 2021).
- **HyperNetworks** ([`arxiv.org/abs/1609.09106`](https://arxiv.org/abs/1609.09106)) — small network generates large network's weights per task (Ha 2016).
- **Capsule Networks** ([`arxiv.org/abs/1710.09829`](https://arxiv.org/abs/1710.09829)) — Hinton 2017 dynamic part-whole routing per input.
- **Connectomics** ([Wikipedia](https://en.wikipedia.org/wiki/Connectome)) — C. elegans (302 neurons), Drosophila FlyWire 2024 (25,000 neurons, 53M synapses) — biological analogue justifying the scale of the aspiration.
- **Neurogenesis** literature (Sorrells 2018 retraction → Duque-Moreira 2022 micro-CT + Taberner 2023 MRI) — biological precedent that mammalian brains do form new neurons in adulthood.
- **Spiking Neural Networks** ([Wikipedia](https://en.wikipedia.org/wiki/Spiking_neural_network)) and **neuromorphic chips** (Intel Loihi 2, Tianjic) — energy-budget pattern for the on-device path.

**Phase S5 of `PRODUCT_ROADMAP.md`** is the formal milestone: "per-query topology generation, evaluated against D-3's gate". Until S5 ships we explicitly document the system as **statistical MoE-scale**, not **millioscale dynamic topology**.

**Why binding:** advertising what the system is not yet doing is the single most damaging thing an AI-OS project can do (cf. Sydney, cf. every "AGI by Q3" press release). Pinning the aspiration to a roadmap phase makes over-claiming a process violation of the Constitution, not just a marketing choice.

---

## 3. Consequences

### Positive
- **Single-shot reasoning about safety.** "What can the AI do?" is the question "What MCP tools fire under which Constitution revision under which PEP grant?" — not a per-agent case-by-case audit.
- **Audit is one format.** Every consequential action emits the same audit-row shape, anchored to a hash-chained ring. Incident response becomes dataset query, not archaeology.
- **Trainable.** When Pillar C training begins (Phase S3 per roadmap), CAI on a written Constitution replaces "tilting at opaque preference models".
- **Predictable progression.** Phase S1..S5 are each gated by a specific evaluation function and Constitution revision — investors and users can read progress, not vibes.

### Negative / risked
- **MCP spec drift.** If Anthropic revises MCP after our v=2026-07-28 lock-in, we either follow (and re-audit every tool) or fork and document why.
- **Single-protocol risk.** MCP is not battle-tested at multi-engagement pentest scale. If it throttles, we have no Plan B in-protocol — fall-back is the **C-4 audit discipline** (every refused-dispatch row is recorded).
- **Constitutional AI gate latency.** Every tool dispatch adds a Constitution check; we may have to amortise this on a hot path (cached Constitutional hash + short-circuit on identical-context calls).
- **Voyager/AlphaEvolve inheritance.** Both come from a different research tradition (embodied agents, code generation, respectively). Adapting them to a kernel subsystem is original work, not port-and-pray.

### Reversibility
Each decision is independently reversible through a successor ADR. Reversion of **D-4** (Constitution binding) requires Pillar-A scope remediation first.

---

## 4. Considered alternatives (decision-by-decision)

| ID | We chose | Alternatives considered | Why rejected |
|---|---|---|---|
| D-1 | Computer Use loop | LangChain ReAct; Open Interpreter; hand-rolled | Format drift; lacks re-observe guarantee. |
| D-2 | MCP only | Native per-vendor function-calling; gRPC; bespoke JSON | Multi-protocol = multi-PEP = multi-audit. |
| D-3 | Voyager + AlphaEvolve | Fixed skill lib; human-approval-only; LLM-as-judge | No growth / no scale / sycophancy. |
| D-4 | Lexically binding C-1..C-4 | Advisory Constitution; per-agent guardrails | Documentation-only constitutions fail at scale (Sydney 2023). |
| D-5 | Constitutional AI | RLHF; RLAIF; prompts-only | Opaque / sycophancy-prone / single-session-only. |
| D-6 | Aspirational, roadmap-pinned | No claim; production claim | Over-claim; over-honesty. |

---

## 5. Tighter-than-spec commitments

D-2 states that **Sampling** (the deprecated MCP primitive) is **removed from our MCP manifests from day one**. The public MCP spec permits Sampling; we do not use it. Rationale: Sampling lets a server ask the model to "reason" before responding, which is exactly the surface that prompt injection from tool output can weaponise. We preroute it by disallowing the primitive, not by inspecting its payloads.

D-4 states a **single active-constitution revision** is enforced (read from `/etc/aios/active_constitution.sha256`). Any change to the Constitution rolls the AI's "active laws" forward and **invalids the active grant token** — so a malicious Constitution update cannot take effect mid-engagement.

---

## 6. S-rank vs superintelligence — the ASCII scope statement

This ADR binds the **S-rank** cell of the framing. It **does not** claim ASI.

The AGI/ASI literature has median forecasts in the **2050–2100** range (see `en.wikipedia.org/wiki/Superintelligence` and the 2022 expert survey cited there; also [Bostrom's *Superintelligence* (2014)](https://en.wikipedia.org/wiki/Superintelligence)). Companies working on the higher end of that curve are documented and cited where relevant (SSI — Sutskever 2024, $30B valuation Mar 2025, $5B NVIDIA partnership Jul 2026, [Wikipedia: Safe Superintelligence Inc.](https://en.wikipedia.org/wiki/Safe_Superintelligence_Inc.); Meta Superintelligence Labs, Muse Spark model Jul 2026, [Wikipedia: Meta Superintelligence Labs](https://en.wikipedia.org/wiki/Meta_Superintelligence_Labs); Anthropic Constitutional AI, [Wikipedia: Constitutional AI](https://en.wikipedia.org/wiki/Constitutional_AI)). Where we borrow from them, we cite the specific primitive and credit it; we do not claim "we did the same thing" when we are at most "we use the same abstraction at one layer down".

---

## 7. References

### Internal (project)
- `mostimportanAIfolder/AI_CONSTITUTION.md` v1.1.5 — Articles 1.1, 1.2, 1.3, 1.4 (S-rank P-1..P-6 + C-1..C-4).
- `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` v2.0 §4.1–§4.2 — Pillar C subspec + honest ASI positioning.
- `mostimportanAIfolder/PRODUCT_ROADMAP.md` v2.0 — Phase S1..S5 mapping to this ADR.
- `docs/research/AIOS-V2-RESEARCH-2026-08-20.md` — 22 citations, 3-pillar framing.
- `docs/research/AIOS-SUPERINTELLIGENCE-2026-08-20.md` — 13 citations, S-rank vs tool-style agents.
- `docs/research/AIOS-DYNAMIC-NEURAL-TOPOLOGY-2026-08-20.md` — 23 citations, per-query topology pattern.
- `docs/research/V1-13-ai-kernel-safety-primitives.md` — original kernel-safety primitives (pre-v2, retained for substrate continuity).

### External (primary, no fabrication)
- [Model Context Protocol — Introduction](https://modelcontextprotocol.io/introduction)
- [Model Context Protocol — Architecture](https://modelcontextprotocol.io/docs/concepts/architecture)
- [Anthropic — Computer Use docs](https://docs.anthropic.com/en/docs/agents-and-tools/computer-use)
- [Anthropic — Constitutional AI](https://en.wikipedia.org/wiki/Constitutional_AI) (and Bai et al. 2022 paper)
- [Voyager (Wang et al., 2023) — Wikipedia](https://en.wikipedia.org/wiki/Voyager_(AI)) + [`arxiv.org/abs/2305.16291`](https://arxiv.org/abs/2305.16291)
- [AlphaEvolve (DeepMind, 2025) — Wikipedia](https://en.wikipedia.org/wiki/AlphaEvolve)
- [Mixture of experts — Wikipedia](https://en.wikipedia.org/wiki/Mixture_of_experts)
- [Neural architecture search — Wikipedia](https://en.wikipedia.org/wiki/Neural_architecture_search)
- [Liquid time-constant networks — Wikipedia](https://en.wikipedia.org/wiki/Liquid_time-constant_networks)
- [HyperNetworks — `arxiv.org/abs/1609.09106`](https://arxiv.org/abs/1609.09106)
- [Capsule Networks — `arxiv.org/abs/1710.09829`](https://arxiv.org/abs/1710.09829)
- [Connectome — Wikipedia](https://en.wikipedia.org/wiki/Connectome)
- [Spiking neural network — Wikipedia](https://en.wikipedia.org/wiki/Spiking_neural_network)
- [Copilot (Microsoft) — Wikipedia](https://en.wikipedia.org/wiki/Copilot_(Microsoft)) (Sydney incident, Feb 2023)
- [Superintelligence — Wikipedia](https://en.wikipedia.org/wiki/Superintelligence)
- [Safe Superintelligence Inc. — Wikipedia](https://en.wikipedia.org/wiki/Safe_Superintelligence_Inc.)
- [Meta Superintelligence Labs — Wikipedia](https://en.wikipedia.org/wiki/Meta_Superintelligence_Labs)
- [Anthropic — Agentic loop documentation](https://docs.anthropic.com/en/docs/build-with-claude/agent-loop)
- [Karpathy, *Software 2.0*; ReAct (Yao et al. 2022); Tree-of-Thoughts (Yao et al. 2023); Reflexion (Shinn et al. 2023)] — method references cited in Pillar C reasoning substrate.
- Landlock, seccomp-bpf, Flatpak sandboxing — PEP substrate references carried from `AIOS-V2-RESEARCH-2026-08-20.md`.

---

## 8. Review & amendment

This ADR has a **mandatory review trigger**:
- Whenever MCP spec revision > our pinned `2026-07-28`, or
- Whenever `AI_CONSTITUTION.md` is bumped to a new minor/major version, or
- Whenever a Phase S gate (per `PRODUCT_ROADMAP.md`) flips.

Amendment procedure: successor ADR (ADR-0036..ADR-00xx) cross-references this one and writes the change into §2 with a red-line diff.

---

*ADR-0035 is binding as of 2026-08-20.*
