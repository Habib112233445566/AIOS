# AIOS MASTER BLUEPRINT & TASK BACKLOG

**Version:** 2.0 (course correction)
**Status:** ACTIVE (planning aid — not an override)
**Last updated:** 2026-08-20
**Owner:** Human Project Owner (approvals) + Principal AI Architect (maintenance)

---

## 0. What this document is (and is not)

This file is the **forward plan** for the AI-Native Operating System (AIOS). It answers four
questions the team keeps coming back to:

1. **What are we making?** — the product vision, the three pillars, and the target
   system architecture.
2. **What security do we use?** — the security model, threat posture, and cryptography.
3. **How do we write code?** — the coding standards every agent must follow.
4. **How do many agents work in parallel without destroying each other's work?** — the
   collaboration and task-partitioning rules; the concrete task backlog.

### Precedence

```
AI_CONSTITUTION.md            (highest, immutable engineering laws)
  └── MASTER_PROJECT_EXECUTION_PROTOCOL.md
        └── THIS DOCUMENT      (planning aid — elaborates the above, never contradicts them)
              └── PRODUCT_ROADMAP.md, TASK_DATABASE.json, KNOWLEDGE_GRAPH.json, DEPENDENCY_GRAPH.json
```

If this document conflicts with the Constitution or the MPEP, **the higher document wins**.

### Safety guarantees (read this first)

- This document **only adds** a planning file. It changes **no code** and **no
  control-plane file** (`TASK_DATABASE.json`, `DEPENDENCY_GRAPH.json`,
  `KNOWLEDGE_GRAPH.json`, `PROJECT_MANIFEST.yaml`, `tasks/INDEX.md`, ADRs, RFCs,
  specs, or kernel/userland sources).
- It does **not** mark any task COMPLETED. Completion stays the exclusive job of
  the control plane and the boot smoke evidence in `ci/smoke.sh`.
- It is a **source of intent**, not a source of truth. The canonical task ledger
  remains `mostimportanAIfolder/TASK_DATABASE.json`.

---

## 1. Vision — what we are making (v2)

> **AIOS = a Linux system for ethical hacking on the inside, a Windows-style desktop
> on the outside, with AI as a first-class S-rank kernel subsystem that controls
> the whole system.**

This is the user-stated goal and the discipline every other decision must serve.

### The three pillars

| Pillar | Surface | Tier | Goal |
|---|---|---|---|
| **Pillar A — Linux ethical-hacking platform** | Inside (kernel + CLI + services) | Foundation | A Debian/Ubuntu hardening point with the full Kali / Parrot / BlackArch cat-set of security tooling: recon, web, exploit, wireless, passwords, sniff, forensics, report. Real persistent FS, real network stack, real ELF process model. |
| **Pillar B — Windows-like desktop** | Outside (the user sees this) | Surface | KDE Plasma 6 (default Wayland) dressed in a Windows-look Global Theme (taskbar at bottom, start-menu analogue, system tray) + Wine 11.x / Proton 11.x for running Windows binaries (Office, Photoshop, games). Or, on lighter hardware, Xfce 4.20 with the same theme. |
| **Pillar C — AI as S-rank first-class kernel subsystem** | Cross-cutting (controls both) | S-tier | An on-device AI runtime with the broadest, most-trusted view of system state. Exposes its capabilities to every model (local small models via llama.cpp → mid-tier → large frontier models via API) through Model Context Protocol (MCP) JSON-RPC tools. Every tool call goes through the Policy Enforcement Point (PEP) and emits an audit ring entry. **S-rank** = highest-priority, capability-rich subsystem, allowed to plan, decide, and act across Pillars A and B subject only to PEP. |

The OS should feel like a **Kali install dressed in Windows skin, with the AI as the
operator actually in charge.**

### Non-goals (anchored)

- The `kernel/` RISC-V microkernel is a **research substrate**, not the shipping
  target. Its capability/IPC/scheduler primitives inform our userspace design.
- We are not re-implementing Win32. We use **Wine 11.x** and **Proton 11.x**.
- We are not writing a desktop environment from scratch. Pillar B = KDE Plasma 6
  + Windows-look theme + Wine/Proton.
- The AI does not live inside the kernel TCB. It is a high-trust sandboxed service
  with kernel-mediated capability to invoke every subsystem.

---

## 2. Where we are now (honest state, 2026-08-20)

**Already boot-verified in QEMU (kernel/, research substrate):**
- RISC-V boot (SBI, serial/VGA)
- Sv39 paging, frame allocator, VMO / NUMA software model
- Scheduler (FIFO/RR, multi-hart), TCBs, syscalls
- Capability system (CNode/CSpace), IPC + PEP policy engine
- Processes + ELF64 loader + WASM interpreter
- AI agent runtime scaffold (manifest sandbox, PEP-gated IPC, JSON-RPC) — *no real
  inference yet*
- Network stack (ARP/IPv4/UDP/TCP/DHCP/DNS/HTTP) over real virtio-net NIC
- Filesystem: ramdisk + FAT32 parser + VFS
- GUI compositing to framebuffer (software-rendered)

**Shipping path (Pillars A+B on a hardened Linux host):** pragmatic v1 ships before
the from-scratch OS catches up; nothing here blocks the user-facing product.

**Active blockers (now):**
- **Pillar C** — on-device inference is a stub; need pluggable backends
  (llama.cpp / Ollama / remote API).
- **Pillar A** — MCP tool surface for many Kali tools not yet implemented; wires
  the AI ↔ exact tooling commands.
- **Pillar B** — depending on host config; most work is *configuration* of
  upstream KDE Plasma rather than new code.

**Honest gap:** the user-stated vision is the goal; we are ≈1% of the way in **Pillar C**
(agent runtime scaffold only), ≈70% in **Pillar A** via off-the-shelf packages,
≈60% in **Pillar B** via KDE Plasma + Wine/Proton + theming.

---

## 3. Target architecture — what we need to make

The system is decomposed into **3 pillars × 4 layers**. Each layer is owned by an
agent / subsystem with a clear cut-line.

### Layer 1 — Distribution & boot
- Debian 13 (Trixie) **or** Ubuntu 24.04 LTS base image
- Reproducible `aiosh install` that provisions: KDE Plasma 6, Xfce (alt),
  Wine 11.x, Proton 11.x, the Kali/Parrot tool set we choose to ship, the MCP
  server, and the audit ring.
- A signed, deterministic installer ISO image.

### Layer 2 — Interface
- **Pillar A:** CLI / TTY for power users; tool surface mirrors Kali categories.
- **Pillar B:** KDE Plasma session; Windows-look theme; KRunner ↔ integrated
  AI suggestions.
- **Pillar C:** MCP JSON-RPC server listening on localhost + an authenticated
  remote channel; multi-model adapter (local llama.cpp → API).

### Layer 3 — Capability & policy
- Audit ring: every Pillar C tool call writes a tamper-evident row
  (append-only, hash-chained).
- PEP: capability grants — agents cannot invoke a tool without an explicit
  grant from the user; session-scoped or persistent grants, revocable.
- Approval UI: when the agent wants to do something dangerous, the desktop
  shows a Windows-style consent dialog backed by the audit ring.

### Layer 4 — Tooling & data
- The Kali/Parrot/BlackArch tool surface (100s of packages) installed via the
  host package manager; each MCP tool wraps one or more packages.
- Wine / Proton prefix management for Windows apps.
- Per-engagement semantic memory (over an encrypted SQLite DB).

---

## 4. Threat model — what is "S-rank" protecting?

The S-rank AI holds the broadest capabilities; we must hold it accountable.

- **Agentic prompt-injection** — third-party content (a fetched URL, a PDF)
  attempts to issue MCP tool calls. PEP must reject any tool call whose authority
  chain does not trace back to a user-confirmed goal.
- **Capability confusion** — an agent invokes a tool it was not granted.
  PEP denies; audit records the attempt.
- **Audit tampering** — the AI itself tries to scrub its own log. Audit ring is
  append-only + hash-chained; external verifier can detect rewrites.
- **Memory poisoning** — adversarial content injected into RAG / semantic memory.
  Memory entries are signed and schema-validated on read.
- **Coastline drift** — the user's intent and the agent's interpretation diverge.
  Every multi-step plan must surface back to the user for re-confirmation at
  configurable granularity.

## 4.1 — S-rank subsystem design layers (not just defence — capability)

Defence is necessary but not sufficient. The S-rank agent must
**also** carry the cognitive primitives that distinguish it from a
tool-style agent. We adopt a five-layer model, each justified by a
cited external precedent:

- **Layer 1 — Memory (episodic + semantic + procedural).** Episodic = the
  audit ring. Semantic = SQLite + sqlite-vec facts DB. Procedural =
  `vault/skills/` indexed skill files. Pattern after: Anthropic Claude's
  "Dreaming" feature (May 2026, Wikipedia, Claude) + Voyager's
  Minecraft skills library (Wikipedia, RSI).
- **Layer 2 — Bounded recursive self-improvement.** `genesis` writes
  candidate skills; `steward` validates them in a sealed testbed before
  PEP permits promotion. Pattern after: AlphaEvolve's "needs an
  evaluation function" (Wikipedia, AlphaEvolve) + the Seed AI
  architecture (Wikipedia, RSI). We deliberately do *not* let the
  agent rewrite itself; that is ASI territory.
- **Layer 3 — Embodiment via MCP + KDE Plasma desktop.** MCP `gui.*`
  tools for screenshot, mouse, keyboard over Wayland/KWin; AT-SPI /
  UIA adapters for accessibility-tree reads. Pattern after: Anthropic
  Computer Use reference impl (Claude executor in a Docker container
  with Xvfb + Mutter + Tint2 + Firefox).
- **Layer 4 — Constitutional scaffolding.** The C-1..C-4 cautions in
  the Constitution operate as the *refusal engine*. Pattern after:
  Anthropic Constitutional AI (Wikipedia, Claude) — Claude's
  constitution grew from 2,700 words (2023) to 23,000 (2026) to keep
  pace with capability; we adopt the same self-consistent principle.
- **Layer 5 — Cognitive-architecture primitives.** Off-load to
  environment (audit rows, skill files), time-pressure the agent
  (engagement deadlines), situated in system state (live open files,
  recent logins), action-terminated reasoning (every plan step ends in
  a tool call). Pattern after: embodied-cognition literature
  (Wikipedia, Embodied cognition) and Symbolic cognitive architectures
  (ACT-R, Soar, Wikipedia, Cognitive architecture).

### 4.2 — Why this isn't ASI — honest positioning

Worth saying out loud: we do not claim to build Artificial
Superintelligence. The AGI/ASI literature (Wikipedia, AGI) places
high-level machine intelligence around a 2022-survey median of **2061**
and AGI itself "before 2100" in most surveys. Our S-rank subsystem
sits at the **competent → expert** rung of DeepMind's 5-level AGI
scale, and the **collaborator → expert** rung of its 5-level autonomy
scale. We build the substrate that *any* plausible timeline sup-
ports: persistent, value-aligned, capable, Linux-resident AI.

### 4.3 — Layers we are not skipping

- No "zero memory" agents. (Tool-style pattern, brittle on long
  engagements.)
- No "no ethics" agents. (Refusal is part of safety, not an option.)
- No "no embodiment" agents. (MCP `gui.*` is non-optional in v3.)
- No "Constitutional AI without a growing Constitution". (Capability
  growth requires constraint growth.)

---

## 5. How we write code (engineering rules)

### Coding standards
- Use idiomatic language conventions (Python → PEP 8, Rust → clippy pedantic,
  shell → ShellCheck clean). Cite lint output in PR.
- Every commit message references the AIOS-XXXX task ID it advances.
- Tests are required for any production code; smoke output quotes are required
  for any boot-visible change.

### MCP schema discipline
- Tools are versioned (`aiosh.pentest.recon.nmap` v1, v2...).
- Tools give structured responses (JSON, not prose) so the AI can chain them.
- Tool capability tags are explicit; PEP refuses cross-tag calls without the
  matching grant.

### Reproducibility
- All install steps in `aiosh install` are deterministic (pinned package
  versions; signed manifests).
- A "golden image" can be regenerated from the manifest and verified against
  a known-good checksum.

---

## 6. How many agents work in parallel

- **Role agents:** Pillar A agent (pentest), Pillar B agent (desktop ops),
  Pillar C agent (orchestrator). Single orchestrator coordinates;
  sub-agents isolated by capability scope.
- **Human in the loop:** every multi-step plan surfaces for explicit approval
  before execution. Single-click "yes" advances. "No" aborts and rolls back
  side effects where possible.
- **Conflict-free collaboration:** each agent owns its slice; cross-pillar
  plans route through the orchestrator (singleton).

---

## 7. Task backlog (top-level)

The canonical detail lives in `TASK_DATABASE.json`. This is the high-priority
list:

1. **Pillar C spine:** pluggable inference backends (llama.cpp + Ollama + remote API)
2. **Pillar C MCP server:** tool surface for `process.*`, `fs.*`, `net.*`, `audit.*`,
   `pentest.*`, `gui.*`, `system.*`
3. **Pillar B installer config:** KDE Plasma 6 + Windows-look theme + Wine + Proton
4. **Pillar A wrapper set:** top-of-list Kali/Parrot tools with MCP face,
   matching MITRE ATT&CK category ordering (Kali menu reorganization from
   v2025.2 is the canonical index)
5. **Pillar C Policy Enforcement Point (PEP):** capability grants + audit ring +
   consent UI
6. **Pillar A engagement memory:** per-engagement sealed SQLite + cross-engagement
   redaction-aware query surface
7. **Cross-pillar smoke:** `aiosh demo` — natural-language command → AI decomposes
   → MCP tool runs → audit row appended → user-visible result on desktop

---

## 8. NOW (where to start work)

1. Fix the active blocker in **Pillar C** (Pillar C spine is the prerequisite
   for everything else).
2. Then in dependency order: Pillar A tools with MCP wrappers → Pillar B
   installer → Pillar C PEP/audit UI → cross-pillar smoke.

> **Direction change vs. v1:** the v1 blueprint emphasised a from-scratch RISC-V
> microkernel as the foundation. That is great research but it slowed the
> user-facing product. In v2, **the foundation is the Linux host + AI spine**;
> the microkernel remains a research substrate that informs our capability/IPC
> designs but is not the shipping target.

---

## Citations (anchored)

The complete v2 research record with 22 authoritative URLs is in
`../docs/research/AIOS-V2-RESEARCH-2026-08-20.md`. Highlights for the
canonical v2 plan:

**Pillar C — AI subsystem / MCP transport / inference backends:**

- MCP architecture (Host → Client → Server; Tools/Resources/Prompts
  primitives; STDIO + Streamable HTTP transports; discovery via
  `server/discover`; **protocol version 2026-07-28**; `Sampling` deprecated).
  Sources: <https://modelcontextprotocol.io/introduction>,
  <https://modelcontextprotocol.io/docs/concepts/architecture>
- Anthropic (the MCP author + Claude family Haiku/Sonnet/Opus/Vision).
  Source: <https://en.wikipedia.org/wiki/Anthropic>
- Pluggable inference: **llama.cpp** (MIT) backend; **Ollama** v0.22.1
  (MIT, libcurl REST on :11434, Apple MLX support since Mar 2026, 9M users,
  65M funding Jul 2026); **LM Studio** (proprietary, OpenAI/Anthropic-
  compatible APIs, Bionic agent added Jul 2026). Sources:
  <https://en.wikipedia.org/wiki/Ollama>,
  <https://en.wikipedia.org/wiki/LM_Studio>
- **OS-level AI precedents we explicitly learn from:**
  Apple Intelligence (AFM v3 family: 3B on-device → 20B on-device → cloud →
  Cloud Pro on Nvidia GPUs via Private Cloud Compute; Foundation Models API
  ships tool calling; Apple ↔ Google Gemini partnership for next-gen). Source:
  <https://en.wikipedia.org/wiki/Apple_Intelligence>
  Microsoft Copilot / Windows Copilot (Windows 11 integration, Microsoft
  Prometheus model on OpenAI GPT; November 2025 Windows 11 build can run
  background tasks reading/writing files). Source:
  <https://en.wikipedia.org/wiki/Copilot_(Microsoft)>
  Anthropic Computer Use (Claude Opus 4.5 + Sonnet 4.5 + Haiku 4.5; beta
  header `computer-use-2025-11-24`; Xvfb + Mutter + Tint2 + Firefox
  reference impl in Docker; **explicit prompt-injection caveat**: "In some
  circumstances, Claude will follow commands found in content even when
  they conflict with your instructions"; classifier-based defense). Source:
  <https://docs.anthropic.com/en/docs/agents-and-tools/computer-use>
  OS-wide integration overview: "Microsoft, Apple, ByteDance, and Google".
  Source: <https://en.wikipedia.org/wiki/AI_agent>

**Pillar A — Linux ethical-hacking base:**

- Kali tool taxonomy (MITRE ATT&CK-aligned since v2025.2; v2026.2 in Jun 2026
  added 9 tools incl. `gemini-cli`, `hexstrike-ai`).
  Source: <https://www.kali.org/tools/>,
  <https://en.wikipedia.org/wiki/Kali_Linux>
- MITRE ATT&CK v19 (Apr 2026) — 15 tactic categories now (Defense Evasion
  split into Stealth + Defense Impairment). Source:
  <https://en.wikipedia.org/wiki/MITRE_ATT%26CK>
- Parrot OS 7 (Jun 2026) — KDE Plasma default since Dec 2025.
  Source: <https://en.wikipedia.org/wiki/Parrot_OS>
- BlackArch — 2866 tools across 47 categories, rolling release.
  Source: <https://en.wikipedia.org/wiki/BlackArch>

**Pillar B — Windows-like desktop:**

- KDE Plasma 6.7.4 (Aug 2026) — Wayland default since 6.0; Windows-look
  themeable. Source: <https://en.wikipedia.org/wiki/KDE_Plasma>
- Xfce 4.20 (Dec 2024) — lightweight GTK alt on low-spec hosts (Kali default).
  Source: <https://en.wikipedia.org/wiki/Xfce>
- GNOME 50.2 (Jun 2026) — X11 dropped since GNOME 49; Wayland-only. Source:
  <https://en.wikipedia.org/wiki/GNOME>
- Wayland 1.26.0 (Jul 2026) — current display protocol. Source:
  <https://en.wikipedia.org/wiki/Wayland_(display_server_protocol)>
-Wine 11.0 (Jan 2026) + Proton 11.0-1 (Jul 2026) — Windows binary compat
  on Linux. Sources: <https://en.wikipedia.org/wiki/Wine_(software)>,
  <https://en.wikipedia.org/wiki/Proton_(software)>

**PEP substrate / sandbox machinery:**

- seccomp-bpf — Linux kernel facility since 2.6.12 (2005), BPF-rule mode
  since 3.17 (2014); used by Chromium, Docker, Flatpak, Snap, systemd, OpenSSH,
  vsftpd, QEMU, Firejail. Source: <https://en.wikipedia.org/wiki/Seccomp>
- Flatpak 1.18.1 (Aug 2026) — sandboxed app distribution with permission
  model (Bluetooth, sound, network, files). Source:
  <https://en.wikipedia.org/wiki/Flatpak>
- Linux `capabilities(7)` (CAP_* fine-grained split of root), namespaces
  (`user_namespaces(7)`, `network_namespaces(7)`), Bubblewrap.

**Adjacent research:**

- Agent fine-tuning methodology: arXiv 2403.12881 *Agent-FLAN*.
  Source: <https://arxiv.org/abs/2403.12881>
- Forensic analysis of local LLM artifacts (Ollama / LM Studio / llama.cpp):
  arXiv 2603.23996 (cited via the LM Studio Wikipedia article above).
