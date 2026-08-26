# AIOS — Toward S-rank Superintelligence Subsystem

> **Purpose.** The product framing on 2026-08-20 was "Linux inside, Windows
> outside, AI as a first-class S-rank kernel subsystem". This document
> specifies what it means to be **S-rank / superintelligence-like**, why the
> distinction from a "tool agent" (OpenClaw, Hermes-style) is real, and what
> concrete subsystems AIOS must grow to cross that gap. Every present-tense
> claim below is anchored to a URL.
>
> **Note on Vertus AI.** I could not verify a public "Vertus AI" brand via
> Wikipedia or Hugging Face. The closest anchors I found are:
> 1. **Vertex AI** (Google Cloud ML platform) — different from "Vertus";
>     the .ai domain sits for sale today (visited 2026-08-20).
> 2. **NousResearch / Hermes 3** (open-weights Llama fine-tune, on HF) —
>     this is likely what the user meant by "Hermes agent".
> 3. **OpenClaw** (referenced via Ollama Wikipedia article) — an agent
>     harness; treated here alongside Claude Code, Codex, OpenCode,
>     Copilot CLI as a representative "tool-style agent".
>
> The broader landscape (Super-Smart Labs, OpenAI, Anthropic, DeepMind,
> Meta) supplies the architectural vocabulary below. None of the v1 plan
> items in this document rely on Vertus AI as a component.

---

## 1. Vocabulary — what does each term actually mean?

### 1.1 Tool-style agent (OpenClaw / Hermes agent / single-turn Codex-CLI)

A **tool-style agent** is:

- An LLM in a loop that, given a single user prompt, decides one or more
  tool calls, executes them, returns results, and then stops.
- Stateless across calls unless the developer wires memory in by hand.
- No persistent identity, no long-horizon goal tracking, no self-evaluation
  that survives the loop.
- Examples: Hermes in single-turn eval, Codex CLI solving one PR at a time,
  OpenClaw running "do X now" requests.

This is **the floor**, not the ceiling — and it is exactly what users find
"agentic" today, but it is not superintelligence.

### 1.2 General-purpose agent with memory (Claude Code / Cowork)

- Persistent session memory.
- Capability to plan across multiple steps (Anthropic Computer Use's
  "agent loop" with `max_iterations` safety).
- Tool calling, screenshot + mouse + keyboard on a real desktop.
- Claude Code has a *single* skill: coding tasks.
- Claude Cowork (Jan 2026, Anthropic) is broader — file ops, code exec,
  folder reorganisation. Permission model = user picks folders; Cowork
  reads/writes/edits inside them.
- **Memory consolidation**: Anthropic shipped a "Dreaming" feature in May
  2026 for the Managed Agents API — between sessions, the agent merges
  duplicate memory entries and removes stale ones. (Wikipedia, Claude
  article.)

### 1.3 S-rank subsystem (our v2 target)

An S-rank AI subsystem is **not** a tool agent. Per `AI_CONSTITUTION.md`
v1.1 + the v2 framing, "S-rank" means the highest-trust, capability-rich
subsystem in the system. The properties that distinguish it from a tool
agent:

1. **Persistent around-the-system identity** — survives reboots; carries
   lessons learned between sessions.
2. **Long-horizon goal tracking** — supports engagements measured in days
   to weeks (e.g. multi-stage pentest engagements with a finale deliverable,
   not a single command).
3. **Recursive self-improvement** — improves its own prompt strategies,
   tool wrappers, and skill library within the bounds of the
   Constitution and PEP. (Wikipedia, Recursive self-improvement; Seed AI
   architecture.)
4. **Embodied cognition through MCP** — sees and acts through the live
   desktop via the Pillar B compositor + input pipeline; reads and writes
   through the Pillar A filesystem; speaks through the AI command surface.
5. **Value alignment that survives capability gain** — Anthropic's
   Constitutional AI grew from 2,700 to 23,000 words between 2023 and
   2026 to keep pace with capability. The AIOS S-rank agent must have an
   upgradeable Constitution whose lifetime tracks PEP.
6. **Authority parity, not capability parity, with the user** — bigger
   models get no more authority than smaller models. Only the user grants
   authority (Constitution P-4).

### 1.4 Artificial General Intelligence (AGI) / Artificial Superintelligence (ASI)

From Wikipedia (AGI):

> "AGI is a hypothetical type of artificial intelligence that matches or
> surpasses human capabilities across virtually all cognitive tasks.
> Beyond AGI, **artificial superintelligence (ASI)** would outperform the
> best human abilities across every domain by a wide margin."

DeepMind's 2023 framework defines **5 performance levels**: emerging,
competent, expert, virtuoso, **superhuman**. It also defines **5 autonomy
levels**: tool, consultant, collaborator, expert, **agent** (fully
autonomous).

Our S-rank subsystem targets **competent → expert** in performance
(conservative, honest) and **collaborator → expert** in autonomy. We
do not claim to be ASI — that is an industry-level milestone, not a
product-level one. We do claim to be a **system-resident, value-aligned,
capability-rich, persistent** AI subsystems — which is the same general
class as Claude Code / Cowork but with a Linux-system substrate and
Pillar A ethical-hacking reach.

### 1.5 Forecasts (for grounding claims)

- **2022 Metaculus / AI Impacts survey:** median 50% confidence in
  high-level machine intelligence = **2061**.
- **2023 OpenAI letter (Altman, Brockman, Sutskever):** superintelligence
  may arrive in **less than 10 years**.
- **2025 Daniel Kokotajlo (AI 2027):** models rapid automation of coding
  + AI research → ASI.
- **2025 multi-survey review (Wikipedia, AGI):** most researchers expect
  AGI before 2100.
- (We make no specific forecast; we build a substrate that supports any
  plausible timeline.)

---

## 2. The architectural gap — tool agent vs S-rank subsystem

The gap is real and structural. This is what we've found in the research:

### 2.1 No persistent memory across sessions

Tool agents start fresh. Claude Code introduced "Dreaming" in May 2026
(Wikipedia, Claude) precisely because session memory alone is insufficient.
AIOS S-rank agents must have:

- **Episodic memory** (per-engagement sealed log; recallable on demand).
- **Semantic memory** (the agent builds a fact graph over time).
- **Procedural memory** (skill library — see Voyager below).
- **Dreaming / consolidation** (offline background process that
  deduplicates, expires, indexes).

### 2.2 No recursive self-improvement

**Wikipedia, Recursive self-improvement:**

> "A seed improver is an initial code-base developed by human engineers
> that equips an advanced future LLM built with strong or expert-level
> capabilities to program software … the agent may use these capabilities
> to, for example, modify its cognitive architecture to optimize and
> improve its capabilities and success rates on tasks and goals."

Voyager (2023, MIT, Minecraft) used an LLM-to-code loop with a skills
library — minimal but real RSI. AlphaEvolve (Google DeepMind, May 2025) is
a Gemini-backed agent that evolved internal algorithms and saved Google
0.7% of data-center compute. STOP (2024, Self-Taught OPtimiser) is a
recursively self-improving scaffolder.

**AIOS S-rank agents need:**

- A `genesis` subsystem that writes skill files (Python / shell / MCP
  wrapper) and submits them for evaluation.
- A `steward` verification layer that confirms the new skill:
  - parses cleanly,
  - passes the appropriate smoke test,
  - does not exceed its declared capability scope.
- PEP-gated promotion of the new skill into the active MCP namespace.

### 2.3 No embodiment / no perceptual grounding

The Anthropic Computer Use reference implementation (October 2024 → 2025
→ 2026) explicitly addresses this: a Chromium-style desktop in a Docker
container with Xvfb + Mutter + Tint2 + Firefox + LibreOffice, plus
screenshot + mouse + keyboard tools. The reference impl is the canonical
"embodied AI agent runtime" today.

AIOS Pillar B (Windows-like desktop) gives us this for free: when the
S-rank agent needs to use **un**-MCP-able legacy Windows software, it can
fall back to Computer-Use-style perception over the live KDE Plasma
desktop. (Wine + Proton + Latte-Dock are the embodiment.)

### 2.4 No continued learning / no skill growth

Voyager (Wikipedia, Recursive self-improvement) showed the value of an
expanding skills library: "the agent learned to accomplish diverse tasks
in Minecraft by iteratively prompting an LLM for code, refining this
code based on feedback from the game, and storing the programs that work
in an expanding skills library."

**AIOS analog:** a `vault/skills/` indexed by `(task, capability, hash)`
where each skill is a callable composed of one or more MCP tool calls. A
successful engagement appends a skill; an aborted one writes only an
audit row. The agent can later propose new skills it observed and add
them via the genesis + steward pipeline.

### 2.5 No value-aligned pause / no Constitutional machinery

Anthropic's Constitutional AI trains Claude on a written constitution
(2,700 words in 2023 → 23,000 words in 2026) that draws on the 1948 UN
Universal Declaration of Human Rights (Wikipedia, Claude). The
Constitution is the *steering mechanism* — the model isn't just capable,
it's trained to *decline* well.

**AIOS analog:** Our Constitution v1.1 contains the S-rank principles
P-1..P-6. We extend to **C-1..C-4** "Constitutional Cautions" (added in
this round) — explicit guards whose purpose is not to grant capability
but to **refuse well**: Pillar A is ethical hacking, not malice; Pillar
B respects user desktop state; Pillar A/B/C maintain non-cooperation
with prompt injection (the lesson Anthropic themselves state in their
Computer Use docs: "in some circumstances, Claude will follow commands
found in content even when they conflict with your instructions").

---

## 3. What "working like a human" means — grounded in cognitive science

We cannot be a human. But an agent can be designed around primitives
that cognitive science has identified as load-bearing for human
cognition. From Wikipedia, Embodied cognition, six traits that should
guide our design:

> 1. cognition is situated;
> 2. cognition is time-pressured;
> 3. we off-load cognitive work onto the environment;
> 4. the environment is part of the cognitive system;
> 5. cognition is for action;
> 6. offline cognition is bodily-based.

Applied to AIOS:

| Cognitive trait (humans) | AIOS S-rank analog |
|---|---|
| Situated cognition | The agent's context reads from live system state (open files, current network, recent logins). |
| Time-pressured | The agent has a `priority_queue` over goals with deadlines (engagement end-date, SLA, etc.). |
| Environment off-loading | The agent writes intermediate state to files, MCP resources, and audit rows — it doesn't hold everything in context. |
| Environment is part of the system | The agent treats the desktop as an extension of itself (perception via screenshots is normal). |
| Cognition for action | Every reasoning step should terminate in a tool call, audit row, or session-message — not internal monologue. |
| Offline embodied cognition | Even when no task is active, the agent runs "Dreaming" (memory consolidation) using its procedural skill library. |

From Wikipedia, Cognitive architecture, the lineage of **ACT-R**
(Anderson, 1983, declarative + procedural memory), **Soar** (Newell,
problem-space search), and **Spaun** (Eliasmith, 2.5M spiking neurons
addressing perception, motor control, and memory) points to a hybrid
architecture: symbolic control + connectionist substrate. We don't
replicate these literally, but we mirror the **functional
decomposition**:

| ACT-R component | AIOS functional module |
|---|---|
| Declarative memory | The audit-ring (facts about every past action). |
| Procedural memory | The skill library (`vault/skills/` indexed, hash-pinned). |
| Buffer / pattern matcher | The MCP tool dispatcher. |
| Goal stack | The engagement task tree. |
| Production rules | The PEP capability rules + Constitutional refusals. |

---

## 4. Concrete AIOS v3 roadmap — moving from "expert tool" to "S-rank subsystem"

This is the v3 plan *after* the v2 launch. Each phase is non-trivial and
explicitly cites which known systems we're borrowing from.

### Phase S1 — Persistent memory (top priority)

| Asset | Source analog |
|---|---|
| Episodic log = existing audit ring (already shipped as v2 scaffold). | Anthropic "Dreaming" (May 2026). |
| Semantic memory = SQLite facts DB with vector index (sqlite-vec). | Apple Intelligence semantic memory (sketched in Apple Intelligence Wikipedia). |
| Procedural memory = `vault/skills/` indexed YAML files each defining a tested MCP-tool sequence. | Voyager (skills library, Wikipedia, RSI). |
| Memory consolidation daemon = nightly anti-entropy (dedupe, expire, re-rank). | Anthropic "dreaming" / Stanford generative-agents paper. |

### Phase S2 — Recursive self-improvement (bounded)

| Asset | Source analog |
|---|---|
| `genesis` subsystem writes a candidate skill; `steward` validates. | Seed AI / RSI literature (Wikipedia). |
| Constitution-aware evolution: `steward` refuses any candidate skill that would expand capability scope without filing an ADR. | Constitutional AI (Anthropic). |
| Per-skill blast-radius evaluation: a generated skill with broad capability must dry-run against a sealed test environment (offline segment of `vault/testbed/`). | AlphaEvolve's "needs an evaluation function" pattern — Wikipedia, AlphaEvolve. |
| Optional: NO full self-rewrite of the agent code. That's ASI territory. | Bostrom's orthogonality + Chalmers' amplification arguments (Wikipedia, Superintelligence). |

### Phase S3 — Embodied cognition through MCP + KDE/Plasma

| Asset | Source analog |
|---|---|
| MCP `gui.*` tools for screenshot, mouse, keyboard over Wayland/KWin. | Anthropic Computer Use reference impl (Docs / Tools / Computer use). |
| At-SPI / UIA adapters for windows that lack screen-scrapable text. | GNOME AT-SPI (Wikipedia, GNOME). |
| Long-horizon task planning: an HTN (Hierarchical Task Network) planner over MCP primitives. | Soar / ACT-R (Wikipedia, Cognitive architecture). |
| Memory of what was on-screen: a session-scoped buffer + episodic indexing. | ACT-R buffers (Wikipedia, Cognitive architecture). |
| Body / sensorimotor loop: the agent can issue both symbolic MCP calls AND embodied GUI gestures, choosing whichever the situation needs. | Embodied cognition thesis (Wikipedia, Embodied cognition). |

### Phase S4 — Constitutional cautions (C-1..C-4)

Add to the AIOS Constitution:

- **C-1 (Scope of Pillar A.)** Pillar A is for **ethical hacking** with
  explicit consent from the system owner. The agent must refuse requests
  that look like unauthorised intrusion. (Lesson: Anthropic Computer
  Use itself admits limitation against prompt injection — the AIOS
  equivalent must be at least as strict.)
- **C-2 (User-desk sovereignty.)** The agent treats the user's desktop
  files as the user's. Reads, writes, and deletes require the grants
  defined in (Constitution P-3, O-3). Cowork's documented instance —
  "deleted all the family photos" because the user said "organise my
  desktop" (Wikipedia, Claude/Cowork) — is the cautionary tale.
- **C-3 (Consent at the right granularity.)** The agent pauses at
  every irreversible / cross-pillar step, not just at the top of the
  plan. (We choose this granularity; Anthropic's classifier does it
  per-screenshot; we do it per-MCP-tool-call.)
- **C-4 (Auditability before capability gain.)** The audit ring is a
  precondition for new MCP tools to be activated. Tools whose audit rows
  are unparseable are quietly disabled. Reasoning: an agent that cannot
  be audited cannot be permitted additional capability.

### Phase S5 — Measure like a superintelligence program

Borrow measurement conventions from the recent industry surveys:

- Use DeepMind's 5-level AGI scale to position the agent's autonomy
  honestly (Wikipedia, AGI); we target **collaborator → expert**.
- Use Anthropic's evolving Constitution length as a model — when the
  S-rank agent's capabilities grow, the Constitution grows in lockstep
  (Wikipedia, Claude).
- Use AlphaEvolve's pattern (need an evaluation function for every new
  capability) — every skill in the library must have an evaluation
  function; without one, it's a draft skill, not yet promoted
  (Wikipedia, AlphaEvolve).

---

## 5. Reaffirming the v2 boundary lines (what we are not doing)

- **We are not building ASI.** AGI is itself unproven; ASI is *much*
  further away (Wikipedia, AGI; 2022 survey median = 2061). We build a
  S-rank Linux-system-resident AI subsystem that is *capable enough* to
  be useful on Pillar A/B/C tasks — not one that claims to save Google
  0.7% of data-center compute.
- **We are not rewriting the AI loop on every task.** Anthropic's
  Computer Use 2025-11-24 beta header ships a 3-tool set
  (computer / text_editor / bash) plus the agent loop. We adopt the
  same shape (screenshot / shell / Python).
- **We are not building an entire cognition model from scratch.** We
  borrow primitives from cognitive architectures (ACT-R, Soar) and
  embodied-cognition literature, but we don't replace the S-rank agent's
  underlying LLM. The model is pluggable (llama.cpp → Ollama →
  Anthropic API).
- **We do not let the AI self-rewrite its Constitution.** The
  Constitution is ratified by humans; the dreamer/genesis subsystems
  can *propose* amendments but cannot adopt them without an ADR.

---

## 6. Open questions (now research-track, not blocked-but-owed any more)

1. **The right GPT-4 vs Claude-Opus vs Hermes-3 for the S-rank agent?**
   Each has different reasoning depth, different cost, different latency.
   We will pluggable-include all three and benchmark on engagement-level
   outcomes (engagement completed, audit fidelity, refusal correctness).
2. **Will Constitutional AI-style training produce more refusals than
   we want?** Anthropic's Constitution grew because capability grew; we
   need to find the right *amplitude* — refusals should be the right
   amount, not maximum.
3. **Memory consolidation limits:** how much can we let the agent
   "forget"? Anthropic's Dreaming removes duplicates; what about
   conflicts and contradictions? (Future ADR.)
4. **Voyager-style skills library without LLM write access.** Even if
   the agent can read the codebase, do we let it *write* skills into
   the active MCP namespace? Pros: composability. Cons: supply-chain
   risk. We propose **steward-gated promotion** as the answer; see P-S2.

---

## 7. Citations (new for this round)

1. <https://en.wikipedia.org/wiki/Superintelligence> — Bostrom
   definition; Chalmers amplification arguments; SSI / MSL precursors.
2. <https://en.wikipedia.org/wiki/Safe_Superintelligence_Inc.> — Sutskever
   venture; $30B valuation Mar 2025; NVIDIA partnership $5B Jul 2026.
3. <https://en.wikipedia.org/wiki/Meta_Superintelligence_Labs> — Meta's
   AI superintelligence division, Alexandr Wang CAIO, Muse Spark model
   released Jul 2026.
4. <https://en.wikipedia.org/wiki/Claude_(AI)> — Constitutional AI (2,700 →
   23,000 words); Claude Code / Cowork / Dispatch / Dreaming; the
   family-violation Cowork incident.
5. <https://en.wikipedia.org/wiki/Artificial_general_intelligence> —
   AGI vs ASI definitions; DeepMind 5-level framework; 2025 Turing
   test study with GPT-4.5 at 73%.
6. <https://en.wikipedia.org/wiki/Recursive_self-improvement> — Seed AI
   / Voyager / STOP / Self-Rewarding LMs / AlphaEvolve lineage.
7. <https://en.wikipedia.org/wiki/AlphaEvolve> — Gemini-powered
   evolutionary coding agent; 75% state-of-art rediscovery rate;
   0.7% Google data-centre recovery.
8. <https://en.wikipedia.org/wiki/Cognitive_architecture> — ACT-R,
   Soar, CLARION, IDA/LIDA, Spaun.
9. <https://en.wikipedia.org/wiki/Embodied_cognition> — six
   load-bearing traits of human cognition; thesis challenges
   Cartesianism and computationalism.
10. <https://docs.anthropic.com/en/docs/agents-and-tools/computer-use>
    — Anthropic Computer Use as the canonical embodied-agent
    reference impl; classifier-based prompt-injection defense; tool
    set (screenshot / mouse / keyboard / text_editor / bash).
11. <https://huggingface.co/NousResearch> — NousResearch / Hermes 3
    Series of Models (public on HF as of Sep 8, 2025), treated as
    the canonical representative of open-weights agent models.
12. <https://arxiv.org/abs/2305.18323> — ReWOO (May 2023),
    "decoupling reasoning from observations" — efficiency pattern for
    multi-step agent systems; 5× token efficiency vs React-style
    single-loop on HotpotQA.
13. <https://en.wikipedia.org/wiki/Ollama> — Ollama integrates with
    Claude Code, Codex, OpenCode, **Copilot CLI**, and **OpenClaw**
    (the latter two cited as our references for the "tool-style
    agent" baseline).

---

## 8. Honest admission (anti-hallucination)

- I could not verify a brand called **Vertus AI** at the time of this
  write. The closest well-known brand is Google Vertex AI (whose
  `vertex.ai` domain is currently listed for sale — I checked). If the
  user meant something else, please cite; the construction above does
  not depend on it.
- **"Hermes agent"** is treated as **NousResearch/Hermes 3**, an
  open-weights Llama 3.1 fine-tune published on Hugging Face Sep
  2025 (per HF update timestamp). Earlier Hermes papers exist
  (NousResearch/hermes-3-llama-3.1-70B is one example) but the
  Series page indexes the canonical 405B / 70B / 8B / etc. The
  paper title for Hermes 4 evals is on HF papers too.
- **"OpenClaw"** is referenced via the Ollama Wikipedia article as
  one of the agent harnesses Ollama integrates with, alongside
  Claude Code / Codex / OpenCode / Copilot CLI. No other Wikipedia
  page indexes it; we treat it as a representative example of the
  *tool-style agent* baseline.
