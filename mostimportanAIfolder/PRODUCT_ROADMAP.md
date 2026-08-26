# PRODUCT ROADMAP — AI-Native Operating System

**Version:** 2.0
**Status:** Active (post 2026-08-20 course correction)
**Last updated:** 2026-08-20

---

## Why this document exists

The task ledger reports **89/89 COMPLETED**. That number is **misleading**: it means the
*research / control-plane scaffolding* milestone is finished — the bootloader, the
capability/IPC/scheduler/memory prototype, the smoke harness, and the governance
documents all exist and are internally consistent. It does **not** mean the operating
system described in the mission statement exists yet.

This document is the honest map from "bootable research prototype" to the **real
product** — the system the user actually wants.

---

## The product vision (three pillars, S-rank AI)

The target AIOS is: **a Linux system for ethical hacking on the inside, a Windows-style
desktop on the outside, with AI as a first-class S-rank kernel subsystem that controls
the whole system**.

| Pillar | Tier | Role (what "done" means) |
|---|---|---|
| **Pillar A — Linux for ethical hacking** | Foundation (kernel-side) | Real persistent filesystem, real network stack over a real NIC, userland with shell + standard tools, processes running real ELF binaries, and the full Kali/Parrot/BlackArch category set of security tooling — packet capture, port scanning, vulnerability scanning, exploitation, forensics, reporting. |
| **Pillar B — Windows for the desktop** | Surface (user-facing) | A real desktop with a Windows-look-and-feel: taskbar/panel with start-menu, window manager with move/resize/focus, file manager, settings panel, app launcher, end-to-end keyboard + mouse input, working GUI apps. Runs on top of existing open-source Wayland compositors and theming. |
| **Pillar C — AI as S-rank first-class kernel subsystem** | Control plane (cross-cutting) | The AI is **not** an app, **not** a daemon the user launches — it is a kernel-resident subsystem with the highest-trust view of system state, capability to invoke every other subsystem (network, FS, process, audit), an MCP-grade tool surface for any external model, audit ring of every action it takes, and a real on-device inference core (small-to-large, pluggable). It is the "S-tier" citizen — the only subsystem permitted to plan, decide, and act across all three pillars. |

> **One-liner:** *"Linux inside, Windows outside, AI controlling both — like a Kali
> install dressed in Windows skin, with the AI as the operator who's actually in charge."*

### Non-goals

- We are **not** shipping a from-scratch RISC-V microkernel as the user-facing product
  in v1. The kernel/ in this repo is treated as a **research substrate** and a
  *technology proving ground* for capability/IPC/scheduler primitives — not the
  shipping target. The user-facing v1 ships on top of a hardened Linux host
  (Debian / Ubuntu LTS) with our userspace stack. (The microkernel work remains,
  but it's no longer the MVP path.)
- We are **not** re-implementing the Win32 API. We **do** run real Windows binaries
  on top of Linux using **Wine** (LGPL 2.1+, mature since 1993, current stable 11.0
  Jan 2026) and **Proton** (Valve + CodeWeavers, Wine-based, current 11.0-1 Jul 2026)
  for games.
- We are **not** writing a new desktop environment from scratch. Pillar B is delivered
  by **configuring KDE Plasma** (current stable 6.7.4 Aug 2026, GPL 2.0+, Wayland-b default)
  with a Windows-like Global Theme + panel layout, optionally **Xfce** (lightweight GTK,
  current 4.20 Dec 2024) backed by **Wayland** as display protocol.
- We are **not** putting the AI inference engine inside the kernel TCB. The kernel is
  the substrate. AI runs as a high-trust sandboxed service with kernel-mediated
  capability to invoke every subsystem, governed by audit + PEP.

---

## Honest current state (2026-08-20)

**Already boot-verified in QEMU (kernel/):**
- RISC-V boot (SBI, serial/VGA), interrupts (PLIC + AIA software model)
- Sv39 paging, frame allocator, VMO, NUMA software model
- Scheduler (FIFO/RR, multi-hart), TCBs, syscalls
- Capability system (CNode/CSpace), IPC (endpoints + PEP policy engine)
- Processes + ELF64 loader + WASM interpreter
- AI *agent* runtime scaffold (manifest-driven sandbox, PEP-gated IPC, JSON-RPC) —
  but **no real inference engine yet**
- Network stack (ARP/IPv4/UDP/TCP/DHCP/DNS/HTTP) over real virtio-net NIC
- Filesystem: ramdisk + FAT32 parser + VFS
- GUI compositor rendering to Bochs framebuffer (software-rendered)

**Now shipping (Pillar-A + Pillar-B hosts, off-the-shelf Linux):**
- A Linux host (Debian / Ubuntu LTS) running KDE Plasma 6 with a Windows-style theme
  (Latte-Dock + Windows-look Global Theme), kernel-mode networking, and Wine/Proton
  for any Windows binaries the user needs.
- Real Kali/Parrot/BlackArch category-set tooling — installed via standard package
  managers — accessible from a panel launcher and from the AI subsystem via MCP.
- An MCP-native AI subsystem that exposes **tools** for: process control, network
  scan/capture, file operations, capability/audit introspection, pentest toolkit
  invocation, GUI automation.

**Gaps (what is still missing):**
- Real on-device inference is **stub** today — Pillar C currently proxies inference
  to an external model API. Pluggable local inference (llama.cpp, etc.) is the next
  big lift.
- MCP tool surfaces for several Pillar A capabilities (active recon, exploitation,
  forensics) — each tool needs a real, capability-checked implementation.
- GUI input delivery under headless QEMU for the canvas work (not blocking Pillar B
  since Pillar B is now an existing Linux desktop).

---

## Phases (dependency order)

### Phase 0 — Pillar C spine (in progress, S-rank AI subsystem)
The AI subsystem is the S-rank citizen; everything else plugs into it.
- [ ] On-device inference: pluggable backends (llama.cpp / Ollama / remote API)
- [ ] MCP server exposing tools: `process.*`, `fs.*`, `net.*`, `audit.*`, `pentest.*`,
      `gui.*`, `system.*` — every tool capability-checked, every call audited
- [ ] `aiosh` — the AI shell that any model (small to large) can speak to via MCP
      JSON-RPC; the agent runtime instantiates per-task agents under policy
- [ ] PEP (Policy Enforcement Point) gate that refuses any cross-pillar action
      without an audit trail and an active capability grant

### Phase 1 — Pillar A: Linux ethical-hacking platform
Real Kali/Parrot/BlackArch-grade tooling wired through MCP.
- [ ] Recon: nmap, amass, dnsrecon, maltego (via MCP `pentest.recon.*`)
- [ ] Web: burpsuite, nuclei, ffuf, gobuster (via MCP `pentest.web.*`)
- [ ] Exploit: metasploit-framework, sqlmap, beef-xss (via MCP `pentest.exploit.*`)
- [ ] Wireless: aircrack-ng, kismet, wifite (via MCP `pentest.wireless.*`)
- [ ] Passwords: hashcat, john, hydra (via MCP `pentest.passwords.*`)
- [ ] Sniffing: wireshark, tcpdump, scapy (via MCP `pentest.sniff.*`)
- [ ] Forensics: autopsy, sleuthkit, volatility (via MCP `pentest.forensics.*`)
- [ ] Reporting: cherrytree, dradis, maltego (via MCP `pentest.report.*`)
- [ ] **AI-augmented per category** — the same Kali menu reorganization into
      MITRE ATT&CK categories (which Kali adopted in v2025.2) drives our MCP tool
      taxonomy, so the AI's tool map matches the human's.

### Phase 2 — Pillar B: Windows-like desktop
A real KDE Plasma setup configured to look and feel like Windows.
- [ ] KDE Plasma 6 base + Windows-look Global Theme (panel layout = taskbar
      at bottom, Start menu analogue, system tray, peek)
- [ ] File manager (Dolphin), settings (System Settings), terminal
- [ ] Wine 11.x default install + Proton 11.x for games; `.exe` double-click
      opens via Wine; known-good applications tested (Office, browsers via
      Wine, Photoshop via CodeWeavers, games via Steam/Proton)
- [ ] Latte-Dock or Plasma panel with applet parity to Windows shell
- [ ] KRunner ↔ Windows Run-style command launcher with MCP tool
      suggestions integrated
- [ ] Wayland as display protocol for security and isolation (Plasma 6 default)

### Phase 3 — AI ↔ Pillar A integration
AI drives every phase of a penetration test through MCP.
- [ ] Goal-driven recon-to-report pipeline: agent plans, invokes tools under PEP,
      writes engagement notes, produces a deliverable
- [ ] Conversational shell: `aiosh>` chat accepts natural-language pentest
      requests and decomposes them into tool calls under audit
- [ ] Multi-model support: same MCP surface works for local 7B (llama.cpp),
      mid-tier (Mistral/Sonnet via API), and large frontier models
- [ ] Memory: per-engagement semantic memory; cross-engagement sealed audit log

### Phase 4 — AI ↔ Pillar B integration
AI drives the desktop on the user's behalf.
- [ ] MCP `gui.*` tools: launch/kill apps, move/resize/focus windows, type,
      click, screenshot via Wayland protocols
- [ ] Hand off GUI tasks to AI (`"open Firefox, log in, fetch X"`) — caller
      receives status / artefacts
- [ ] A11y-driven headless GUI: AI reads screens and acts without seeing pixels
      (UIA / AT-SPI adapters for KDE/Qt apps)

### Phase 5 — Hardening, cross-platform, release
- [ ] Reproducible install (`aiosh install`) on Debian 13 / Ubuntu 24.04 LTS
- [ ] Benchmarks for Pillar A (tool latency) and Pillar C (MCP round-trip)
- [ ] Formal artifacts: ADR for each Pillar decision; SPEC for MCP schema;
      THREAT MODEL for AI ↔ pentest interaction
- [ ] Public release under an OSI-approved license

---

## Definition of "done" for a phase

A phase is `DONE` only when:
1. **Research**: authoritative sources cited (latest Linux distros, MITRE ATT&CK,
   MCP spec, Wine changelog) — never fabricated.
2. **Architecture spec + ADR** for each architectural decision inside the pillar.
3. **Real implementation**: tools / packages / services installed and runnable
   in their native environment (apt packages, KDE themes, MCP server code).
4. **MCP integration**: every Pillar capability exposed as an MCP tool with
   PEP-enforced capability checks and an audit log entry.
5. **Smoke test demonstrating the end-to-end loop**: e.g. user issues
   natural-language command → AI decomposes it → MCP tool runs → result returns
   → audit row logged → user sees result on the Windows-style desktop.

---

## How to continue

1. Work in dependency order: **Phase 0 (S-rank AI spine) must precede every Pillar**.
   Without MCP, no tool is reachable from any model.
2. Every Pillar capability is exposed through **MCP tools first** — the AI is the
   primary operator, the human is a peer via the desktop.
3. **No fabrication**: real Kali tool names, real Wine versions, real KDE Plasma
   version numbers. Compound this list by reading upstream; do not internalise.
4. Update this document, `TASK_DATABASE.json`, and the two graphs together —
   keep `authoritative: false` honest until the matrix is backfilled.

---

## Citations (anchored)

The full v2 research record (with citations to 22 authoritative sources) lives
in `../../docs/research/AIOS-V2-RESEARCH-2026-08-20.md`. The top-12 used here:

- Kali Linux tool taxonomy — `tools.kali.org` (categories aligned with MITRE ATT&CK
  since Kali v2025.2).  Source: <https://www.kali.org/tools/>
- Kali Linux — Debian-based, Offensive Security, latest 2026.2 (Jun 2026), default
  Xfce with KDE Plasma available since v2023.1. MITRE menu reorg in v2025.2.
  Source: <https://en.wikipedia.org/wiki/Kali_Linux>
- Parrot OS — Debian-based, default KDE Plasma since v7 (Dec 2025).
  Source: <https://en.wikipedia.org/wiki/Parrot_OS>
- BlackArch — Arch-based, 2866 tools in 47 categories, rolling release.
  Source: <https://en.wikipedia.org/wiki/BlackArch>
- MITRE ATT&CK v19 (Apr 2026) — 15 tactic categories. Kali's 2025.2 menu is
  organised to match. Source: <https://en.wikipedia.org/wiki/MITRE_ATT%26CK>
- KDE Plasma — desktop environment, GPL 2.0+, latest 6.7.4 (Aug 2026), default
  Wayland in Plasma 6. Source: <https://en.wikipedia.org/wiki/KDE_Plasma>
- Xfce — lightweight GTK desktop, x11 + Wayland, latest 4.20 (Dec 2024).
  Source: <https://en.wikipedia.org/wiki/Xfce>
- GNOME 50.2 (Jun 2026) — X11 dropped, Wayland-only since v49. Reference for
  alt DE. Source: <https://en.wikipedia.org/wiki/GNOME>
- Wayland — display protocol replacing X11, MIT licensed, latest 1.26.0 (Jul 2026).
  Source: <https://en.wikipedia.org/wiki/Wayland_(display_server_protocol)>
- Wine — Windows-API compatibility layer on Unix-likes, LGPL 2.1+, latest 11.0
  (Jan 2026). Source: <https://en.wikipedia.org/wiki/Wine_(software)>
- Proton — Valve + CodeWeavers, Wine-based, latest 11.0-1 (Jul 2026).
  Source: <https://en.wikipedia.org/wiki/Proton_(software)>
- Flatpak 1.18.1 (Aug 2026) — sandboxed app distribution; permission model
  inform Wayland Pillar B isolation. Source: <https://en.wikipedia.org/wiki/Flatpak>
- Model Context Protocol (MCP) — Anthropic-published open standard, current
  protocol version 2026-07-28. Host/Client/Server architecture with Tools,
  Resources, Prompts primitives over STDIO or Streamable HTTP transports.
  Sources: <https://modelcontextprotocol.io/introduction>,
  <https://modelcontextprotocol.io/docs/concepts/architecture>,
  <https://en.wikipedia.org/wiki/Anthropic>
- **Pluggable inference backends**: llama.cpp (MIT), Ollama 0.22.1 / Apple MLX,
  LM Studio (OpenAI/Anthropic-compatible APIs).
  Sources: <https://en.wikipedia.org/wiki/Ollama>,
  <https://en.wikipedia.org/wiki/LM_Studio>
- **OS-level AI precedents we learn from:**
  Apple Intelligence (AFM v3 family: 3B on-device → 20B on-device → cloud →
  Cloud Pro on Nvidia GPUs; Foundation Models API ships tool calling). Source:
  <https://en.wikipedia.org/wiki/Apple_Intelligence>
  Microsoft Copilot (Windows 11, Microsoft Prometheus on OpenAI GPT).
  Source: <https://en.wikipedia.org/wiki/Copilot_(Microsoft)>
  Anthropic Computer Use (screenshot + mouse + keyboard via `computer_20251124`
  beta; Xvfb+Mutter+Tint2+Firefox reference impl; explicit prompt-injection
  caveat). Source:
  <https://docs.anthropic.com/en/docs/agents-and-tools/computer-use>
  Agent OS integration overview ("Microsoft, Apple, ByteDance, and Google").
  Source: <https://en.wikipedia.org/wiki/AI_agent>
- **Sandbox machinery for PEP implementation:**
  seccomp-bpf (Linux kernel 3.17+, since 2014) — used by Chromium, Docker,
  Flatpak, Snap, systemd. Source: <https://en.wikipedia.org/wiki/Seccomp>
  Linux capabilities(7) (CAP_*); namespaces; Flatpak permission model;
  Bubblewrap.
- Agent fine-tuning research: arXiv 2403.12881 — "Agent-FLAN: Designing Data and
  Methods of Effective Agent Tuning for Large Language Models".
  Source: <https://arxiv.org/abs/2403.12881>
