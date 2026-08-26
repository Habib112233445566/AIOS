# AIOS Research — Closing the Four Open Gaps (2026-08-21)

> **Purpose.** This note closes the four gaps flagged as "still owed" in
> `docs/research/AIOS-V2-RESEARCH-2026-08-20.md` §8 and the roadmap's gaps
> list:
>
> 1. **Kali / MITRE ATT&CK v19 tool taxonomy** → MCP tool minimum (open Q1;
>    feeds Sprint 3 item 3 — expanding the five pentest wrappers).
> 2. **On-device inference** (llama.cpp / Ollama) — Pillar C's remaining gap.
> 3. **AI ↔ desktop hook** (Wayland / KWin / AT-SPI) — Pillar B Phase 4.
> 4. **Prompt-injection defense for MCP tool outputs** (open Q5).
>
> Methodology per `mostimportanAIfolder/RESEARCH_EXECUTION_PROTOCOL.md`:
> every present-tense claim below was fetched from an authoritative upstream
> URL on 2026-08-21 (links verified live). **Fact** = observed in the cited
> source. **Recommendation/Proposal** = AIOS-specific design decision,
> explicitly marked. No code was changed.

---

## 1. Kali / MITRE ATT&CK v19 tool taxonomy → MCP tool minimum

### 1.1 Fact — MITRE ATT&CK is at v19.2 (confirmed live)

Fetched from the MITRE version-history page (2026-08-21):

- **Current: ATT&CK v19.2, April 28, 2026 – current.**
- Prior: v18.1 (Oct 28, 2025 – Apr 27, 2026), v17.1 (Apr 22, 2025 – Oct 27,
  2025), v16.1, v15.1, …

The Enterprise matrix currently lists **15 tactics**: Reconnaissance,
Resource Development, Initial Access, Execution, Persistence, Privilege
Escalation, **Stealth**, **Defense Impairment**, Credential Access,
Discovery, Lateral Movement, Collection, Command and Control, Exfiltration,
Impact. This confirms the project's prior note (AIOS-V2-RESEARCH-2026-08-20
§1.2): **Defense Evasion was split into Stealth + Defense Impairment** in
v19 — our tool taxonomy must use the 15-tactic list, not the old 14.

Citations:
- <https://attack.mitre.org/resources/versions/> (v19.2 current)
- <https://attack.mitre.org/> (Enterprise matrix, 15 tactics)

### 1.2 Fact — Kali's live tool menu is MITRE-ordered

Fetched from <https://www.kali.org/tools/> (2026-08-21). The live category
tree (tools we can wrap, grouped by the MITRE-aligned menu):

| Kali menu category | Representative tools (live on the page) |
|---|---|
| Reconnaissance · Host Info | metagoofil, spiderfoot |
| Reconnaissance · Identity Info | email2phonenumber, emailharvester, instaloader, linkedin2username, photon, sherlock, tookie-osint |
| Reconnaissance · Network Info | amass, autorecon, dmitry, legion, nmap, zenmap, theHarvester, unicornscan |
| Reconnaissance · DNS | dnsmap, dnsrecon, dnsenum, massdns, dnstracer, dnswalk |
| Web Scanning | assetfinder, arjun, dirb, dirbuster, dirsearch, feroxbuster, ffuf, finalrecon, findomain, gobuster, gospider, lbd, parsero, recon-ng, subfinder, sublist3r, uniscan-gui, urlcrazy, uro, wfuzz, wpprobe |
| Vulnerability Scanning | nmap, zenmap, CAT, gvm-start, heartleech |
| Web Vulnerability Scanning | burpsuite, caido, caido-cli, crlfuzz, davtest, joomscan, nikto, nuclei, paros, skipfish, sstimap, subjack, tinja, wapiti, watobo, wcvs, webscarab, whatweb, wpscan, zaproxy |
| Bluetooth | bettercap, bluelog, bluesnarfer, btscanner, blueranger, fang, spooftooph, ubertooth-util |
| WiFi | asleap, bettercap, kismet, sparrow-wifi, wash |
| Radio Frequency | hackrf_info, gnuradio, gqrx, chirp, rfcat |
| Resource Development | msfvenom, msfpc, searchsploit, ghidra, radare2, rizin, cutter, apktool, jadx-gui, d2j-dex2jar, pompem |
| Initial Access | dns-rebind, gophish, setoolkit, metasploit-framework, sqlmap, sqlninja, sqlsus, jsql, commix |
| Execution | armitage, metasploit-framework, evilgrade, beef-xss, xsser, nishang, powersploit |
| Persistence | laudanum, phpggc, seclists, webacoo, webshells, weevely, backdoor-factory, cymothoa |
| Privilege Escalation | lynis, metasploit-framework, peass, linpeas, winpeas, unix-privesc-check, bloodyad |
| Defense Evasion · Pass-the-Hash | crackmapexec, evil-winrm, impacket-scripts, mimikatz, netexec, rubeus, smbmap, xfreerdp3 |
| Credential Access · OS Credential Dumping | chntpw, creddump7, mimikatz, rubeus, samdump2 |
| Credential Access · Brute Force | CAT, crackmapexec, crowbar, hydra, hydra-gtk, legba, medusa, ncrack, netexec, patator, thc-pptp-bruter |
| Credential Access · Password Cracking | cmospwd, crackle, fcrackzip, hashcat, john, johnny, ophcrack, rcrack, sipcrack, sucrack |
| Credential Access · WiFi | aircrack-ng, airgeddon, bully, cowpatty, eapmd5pass, fern-wifi-cracker, pixiewps, reaver, wifiphisher, wifite |
| Credential Access · Kerberoasting | kerberoast, krbrelayx, responder |
| Discovery · Network Service Discovery | amass, autorecon, masscan, nmap, zenmap, sctpscan, unicornscan, ike-scan |
| Discovery · SSL/TLS | sslscan, sslyze, tlssled |
| Discovery · SNMP | snmp-check, braa, onesixtyone |
| Discovery · Network Sniffing | arpspoof, darkstat, dnschef, driftnet, dsniff, hexinject, netsniff-ng, wireshark, scapy, tcpdump, tcpflow |
| Discovery · Remote System Discovery | arping, arpwatch, fierce, fping, hping3, p0f |
| Discovery · Account Discovery | apache-users, smtp-user-enum |
| Discovery · Network Share Discovery | crackmapexec, enum4linux, enum4linux-ng, nbtscan, netexec, smbclient, smbmap |
| Discovery · Process Discovery | pspy |
| Discovery · Active Directory | azurehound, bloodhound, bloodhound-python, ldeep, sharphound |
| Lateral Movement | crackmapexec, evil-winrm, impacket-smbexec, impacket-psexec, netexec, xfreerdp3, rdesktop |
| Collection | httrack, ettercap, evilginx2, mitmproxy, mitm6, ssldump, sslsplit, wifipumpkin3 |
| Command and Control · Protocol Tunneling | chisel, dns2tcp, dnscat, iodine, ligolo-proxy, miredo, proxychains4, proxytunnel, sshuttle, stunnel4 |
| Command and Control · Non-Application Layer | dbd, ncat, netcat, penelope, powercat, sbd, socat |
| Exfiltration | netcat, impacket-smbserver, goshs, raven |
| Impact | dhcpig, goldeneye, iaxflood, mdk3, rtpflood, siege, slowhttptest, t50, thc-ssl-dos, scapy |
| Forensics · Imaging | dc3dd, dcfldd, ewfacquire, guymager |
| Forensics · Carving | foremost, magicrescue, photorec, scalpel, testdisk |
| Forensics · Sleuth Kit | autopsy, blkcalc, fls, fsstat, icat, ils, mmls, tsk_recover |
| Services · Reporting | cherrytree, dradis, faraday, maltego, obsidian, pipal, cutycapt, eyewitness, witnessme |

Citation: <https://www.kali.org/tools/> (fetched 2026-08-21).

### 1.3 Fact — current wrappers cover 5 of these categories

Existing shipped wrappers (Sprint 1) and their taxonomy home:

| Shipped tool | Kali category | MITRE tactic |
|---|---|---|
| `pentest.nmap` | Recon · Network Info / Discovery · Network Service | Reconnaissance / Discovery |
| `pentest.nikto` | Web Vulnerability Scanning | Initial Access (exploit public-facing) |
| `pentest.sqlmap` | Initial Access / Exploitation | Initial Access |
| `pentest.tshark` | Discovery · Network Sniffing | Collection |
| `pentest.aircrack-ng` | Credential Access · WiFi | Credential Access |

Source: `code/aiosh-mcp/aiosh_mcp/pentest.py` + `code/aiosh-cli/src/pentest.ts`.

### 1.4 Proposal — MCP tool minimum for a "minimal useful pentest agent"

Open Q1 asked: *walk a recon-to-report engagement and enumerate which tools
the AI must have; for each, document the underlying CLI invocation and PEP
capability.* Based on the live Kali menu (§1.2) and the roadmap's Phase 1
categories, the proposed minimum is **four more wrappers per phase** (each
is a **Proposal**, to be ratified by a spec/ADR task before scaffolding):

| New MCP tool | CLI (real Kali command) | Kali category | Why it's in the minimum |
|---|---|---|---|
| `pentest.recon.dnsrecon` | `dnsrecon -d <domain>` | Recon · DNS | Enumeration of DNS records (SRV/TXT/MX) — the AI's first step after target hand-off |
| `pentest.web.nuclei` | `nuclei -u <url> -t <templates>` | Web Vulnerability Scanning | Template-based vuln scanning (the modern replacement for nikto breadth) |
| `pentest.web.ffuf` | `ffuf -u <url>/FUZZ -w <wordlist>` | Web Scanning | Content/path fuzzing — core web recon |
| `pentest.discovery.masscan` | `masscan -p1-65535 <cidr> --rate 1000` | Discovery · Network Service | Full-port sweep at scale; nmap is the follow-up detail scan |
| `pentest.passwords.hydra` | `hydra -l <user> -P <wordlist> <target> ssh` | Credential Access · Brute Force | Online credential validation — central to any engagement |
| `pentest.passwords.hashcat` | `hashcat -m <mode> <hash> <wordlist>` | Credential Access · Password Cracking | Offline hash cracking after credential dumping |
| `pentest.postexploit.netexec` | `nxc smb <target> -u <u> -p <p>` | Defense Evasion · Pass-the-Hash / Lateral Movement | SMB/AD post-exploitation (successor to crackmapexec) |
| `pentest.forensics.autopsy` | `autopsy --no-browser` | Forensics · Sleuth Kit | Evidence triage → report input |
| `pentest.report.pipal` | `pipal <wordlist>` | Services · Reporting | Password-analysis stats for the engagement report |

**Taxonomy rule (Proposal):** namespace = `pentest.<kali-category-slug>.<tool>`
so the MCP surface mirrors the human Kali menu — the AI's tool map equals
the operator's. Keep the flat legacy names (`pentest.nmap` etc.) as aliases
for backward compatibility (existing grants, agent.ts tool allowlists, and
classifier R-01 all reference the flat names).

**PEP capability (Proposal):** every new wrapper is C-1 (Pillar A) and
inherits R-01's 0.50 caution band; R-08 (persist) and R-09/R-10 (target
scope) apply unchanged; brute-force/credential tools should carry a
`rate`-limit argument and an engagement-scope check (target must be within
the grant's `scope.paths`/CIDR list) — same pattern as
`test_pentest_smoke.py` S3/S4.

**Decisions needed before implementation** (each becomes a task):
1. Confirm the 9-tool minimum vs. the full Phase-1 category set (the
   roadmap lists ~8 categories; the minimum is the first slice).
2. Ratify the `pentest.<category>.<tool>` namespace change vs. flat names.
3. Choose wordlist/scope defaults so wrappers never guess a target.

---

## 2. On-device inference (llama.cpp / Ollama) — Pillar C's remaining gap

### 2.1 Fact — llama.cpp (live GitHub README, 2026-08-21)

Fetched from <https://github.com/ggml-org/llama.cpp> (repo now under the
`ggml-org` org; previously `ggerganov`):

- **LLM inference in C/C++**, "minimal setup and state-of-the-art
  performance on a wide range of hardware — locally and in the cloud".
- Plain C/C++ **without any dependencies**; Apple Silicon first-class
  (ARM NEON, Accelerate, Metal); AVX/AVX2/AVX512/AMX on x86; RVV/ZVFH
  on RISC-V.
- **Integer quantization: 1.5/2/3/4/5/6/8-bit** (GGUF) for reduced memory.
- Backends: CUDA (NVIDIA), HIP (AMD), MUSA (Moore Threads), Vulkan, SYCL
  (Intel GPU), Metal (Apple), OpenCL (Adreno), CANN (Ascend NPU), OpenVINO
  (Intel CPU/GPU/NPU, in progress), WebGPU, RPC, ZenDNN (AMD CPU), BLAS/BLIS.
- **CPU+GPU hybrid inference** for models larger than VRAM.
- **`llama serve` = OpenAI-compatible API server**; `llama cli -hf <repo>`
  pulls models directly from Hugging Face; **GBNF grammars** for constrained
  output (useful for forcing JSON/tool-call schema).

**Relevance to AIOS (Fact-level):** llama.cpp exposes an OpenAI-compatible
HTTP API (`llama serve`), so one OpenAI-compatible client can target
llama.cpp locally, Ollama locally, or a remote frontier API — a single
adapter abstraction serves all three Pillar C backends.

### 2.2 Fact — Ollama (live homepage, 2026-08-21)

Fetched from <https://ollama.com/>:

- "Open models, on your computer and in the cloud"; **free to start**.
- Launch agents from the **Ollama CLI**; connect editors/frameworks via
  Ollama's **API** ("See all integrations").
- Privacy claims: data **never trained on**; **run entirely offline**
  ("Disconnected — Run entirely offline for mission critical work");
  cloud regions US / Europe / Singapore.
- Trending models listed (2026-08-21): `qwen3.8` (vision, tools, thinking,
  27B), `deepseek-v4-flash` (cloud, tools, thinking), `kimi-k3` (cloud,
  vision, tools, thinking).

Combined with the project's existing record (Ollama 0.22.1, MLX for Apple
Silicon — AIOS-V2-RESEARCH-2026-08-20 §3.2), the current state is: Ollama
is the lowest-friction local path (package manager install + `ollama pull`
+ OpenAI-compatible `:11434` endpoint); llama.cpp is the zero-dependency
fallback for constrained hardware; remote APIs are the frontier tier.

### 2.3 Proposal — wiring into the existing agent loop

The shipped agent loop (`code/aiosh-cli/src/agent.ts`) already talks to an
Ollama-0.22.1 backend with a deterministic stub fallback. **Proposal:**

1. Introduce a thin `inference` adapter with three backends keyed by config:
   - `ollama` (existing path; OpenAI-compatible `http://localhost:11434/v1`),
   - `llamacpp` (same client; point base URL at `llama serve` port, default
     `http://localhost:8080/v1`),
   - `remote` (Anthropic/OpenAI-compatible API; already the stub's remote
     target).
2. Add `aiosh models list/pull/status` mirroring `ollama list/pull` so the
   operator can switch models without code changes; the classifier /
   audit ring / PEP layer is **backend-agnostic** (it gates tool calls, not
   inference).
3. Prefer **4-bit GGUF** quantized models for the on-device default (fact:
   llama.cpp supports 1.5–8-bit quantization §2.1); a ~7–8B Q4 model runs
   on a mid-range laptop CPU and fits the "small-to-large, pluggable"
   roadmap line.
4. Use **GBNF grammar** (llama.cpp) to force the agent's tool-selection
   JSON schema when on-device, mirroring the structured-call guarantee the
   remote models give natively.

**Decisions needed:** (1) which quantized model family is the v1 default
(needs a benchmark task, not a guess); (2) whether `llama serve` runs as a
systemd unit or is spawned per-session by `aiosh`; (3) whether on-device
inference is gated behind the same R-01 classifier path (it should be —
inference is not a tool, but its outputs enter the agent loop and must be
audited like any other context).

---

## 3. AI ↔ desktop hook (Wayland / KWin / AT-SPI) — Pillar B Phase 4

### 3.1 Fact — KWin scripting is the native Pillar-B automation surface

Fetched from <https://develop.kde.org/docs/plasma/kwin/api/> (KWin 6.0 API
reference) and <https://develop.kde.org/docs/plasma/kwin/> (tutorial):

- KWin scripts are **JavaScript** (or QML) run inside the window manager.
  Global object exposes `workspace`, `options`, `KWin` enums; helpers
  `registerShortcut`, `registerScreenEdge`, `callDBus`,
  `registerUserActionsMenu`.
- `workspace` (WorkspaceWrapper): `stackingOrder`, `activeWindow`,
  `currentDesktop`, `currentActivity`, `windowAt(pos)`, `getClient(id)`,
  `raiseWindow(w)`, `clientArea(...)`, `showOutline(...)`, `createDesktop`,
  and slot actions (`slotWindowMove`, `slotWindowResize`, `slotWindowClose`,
  `slotWindowMaximize/Minimize`, quick-tile slots, switch-desktop slots,
  `slotToggleShowDesktop`).
- `window` (KWin::Window): `clientGeometry`, `pos`, `size`, `x/y/width/height`,
  `caption`, `pid`, `internalId`, `active`, `closeable`, `minimizable`,
  `output`, `stackingOrder`, plus NETWM type flags (normal/dialog/dock/...).
- **Packaging/install:** KPackage format (`contents/code/main.js` +
  `metadata.json`), install with `kpackagetool6 --type=KWin/Script`,
  enable with `kwriteconfig6 --file kwinrc --group Plugins --key
  <name>Enabled true` + `qdbus org.kde.KWin /KWin reconfigure`. A dev
  console is available (`plasma-interactiveconsole --kwin`) for iteration.

**Conclusion (Fact):** move/resize/focus/close/launch/tile + virtual
desktops + screen-edge/shortcut hooks are all natively scriptable on the
Pillar-B desktop (KDE Plasma 6) via KWin scripting — no pixel-reading
required for window *control*.

### 3.2 Fact — Wayland protocols for input injection and window listing

Fetched from Wayland Explorer (2026-08-21):

- **`wlr-foreign-toplevel-management` (v3)** — "list and control opened
  apps"; a client receives every toplevel (title, app_id, states) and can
  `set_maximized`, `set_minimized`, `activate(seat)`, `close`,
  `set_fullscreen`, `set_rectangle`. This is the taskbar/dock pattern —
  the AI can enumerate and drive windows at the compositor level.
  Source: <https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1>
- **`wlr-virtual-pointer` (v2)** — "allows clients to emulate a physical
  pointer device": `motion`, `motion_absolute`, `button`, `axis`,
  `axis_discrete`, `frame`. Source:
  <https://wayland.app/protocols/wlr-virtual-pointer-unstable-v1>
  (companion: `wlr-virtual-keyboard` for key events — the standard
  Wayland input-injection mechanism, used by tools like ydotool on
  compositors that support it).

**Note (Fact):** on Wayland, unlike X11, **arbitrary global input
injection requires compositor cooperation** — the virtual-pointer/keyboard
protocols are the sanctioned path, and KWin implements them. For Pillar B
(KDE Plasma 6) the preferred path is KWin scripting + KWin's own input
facilities; the wlr protocols are the portable fallback / wlroots path.

### 3.3 Fact — AT-SPI2 is the a11y tree for "AI reads screens without pixels"

Fetched from the freedesktop Accessibility wiki and the at-spi2-core dev
guide (2026-08-21):

- **AT-SPI2** is the freedesktop accessibility infrastructure over
  **D-Bus**; toolkits expose a semantic tree of accessible objects.
  GTK3 via ATK + atk-bridge; **GTK4 drops ATK and talks AT-SPI directly**;
  **Qt5 implements the at-spi D-Bus protocol itself** (Qt6 likewise).
- Inspect with **accerciser**; the KDE Human Interface Guidelines and
  Plasma Accessibility docs cover KDE-side support.
- W3C **Core-AAM 1.2** (Candidate Rec. Draft, 2026-08-05) maps web content
  semantics onto platform APIs including **ATK/AT-SPI** — i.e., the same
  tree the roadmap's "a11y-driven headless GUI (UIA / AT-SPI adapters)"
  relies on.

Citations:
- <https://wiki.freedesktop.org/www/Accessibility/>
- <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/index.html>
- <https://www.w3.org/TR/core-aam-1.2/>

### 3.4 Fact — Anthropic's computer-use reference uses X11 + VNC in a container

Fetched from the computer-use demo repo (2026-08-21): the reference
implementation runs a **Docker container with X11 + VNC** (Xvfb + Mutter +
Tint2 + Firefox pattern per the v2 research record) and a Python agent loop
using the `computer_20251124`-era tools; the repo notes it is a "deliberately
minimal, containerized reference" and points to a separate best-practices
quickstart for production patterns (tool definitions, image sizing/pruning,
prompt caching, batched tool calls, sandboxed shell, trajectory recording).
Source: <https://github.com/anthropics/claude-quickstarts/tree/main/computer-use-demo>

**Relevance (Fact):** the industry pattern is screenshot+click on a virtual
display. For AIOS we can do strictly better on Pillar B: **KWin scripting
for window control (§3.1) + AT-SPI for semantic UI state (§3.3) + Wayland
virtual input (§3.2)** — control without pixels, with screenshots
(portal/KWin capture) only as a fallback for non-a11y apps.

### 3.5 Proposal — MCP `gui.*` tool set

| MCP tool | Mechanism | PEP note |
|---|---|---|
| `gui.window.list` | KWin `stackingOrder` / wlr-foreign-toplevel | read-only |
| `gui.window.get` | KWin `getClient`/`clientGeometry` + AT-SPI subtree | read-only |
| `gui.window.move` / `.resize` | KWin `pos`/`size` assignment | C-2 (R-02 0.90 caution) |
| `gui.window.focus` / `.close` / `.minimize` | KWin `activateWindow` / `slotWindowClose` / `slotWindowMinimize` | C-2 |
| `gui.window.tile` | KWin quick-tile slots | C-2 |
| `gui.launch` | `kstart`/`qdbus org.kde.plasmashell` or `process.run` | C-2 + R-05 checks |
| `gui.input.type` / `.key` | Wayland virtual keyboard / KWin shortcut inject | C-2 |
| `gui.input.click` / `.move` | Wayland virtual pointer | C-2 |
| `gui.screen.read` | AT-SPI tree dump (semantic) — primary | read-only |
| `gui.screen.screenshot` | KWin/portal capture — fallback | read-only |
| `gui.a11y.dump` | AT-SPI D-Bus query of a window subtree | read-only |

**Design rules (Proposal):**
- **Semantic-first, pixels-fallback** — AT-SPI first (deterministic, cheap,
  no OCR), screenshots only where the app exposes no a11y tree. This is
  what the roadmap calls "AI reads screens and acts without seeing pixels".
- Every `gui.*` call hits the same **classifier → PEP → audit** gate as
  `pentest.*` (R-02 already tags `gui.*` at 0.90 caution); input injection
  and window mutation are consequential actions → one audit row each
  (O-2, ADR-0035 §F-2).
- The agent loop's `computer-use` port replaces screenshots+clicks with
  `gui.window.*` + `gui.input.*` + `gui.screen.read` — a "desktop-native
  computer use".

**Decisions needed:** (1) KWin scripting D-Bus bridge (call scripts via
`qdbus org.kde.KWin /KWin` reload + eval) vs. long-running packaged script
exposing an IPC channel; (2) whether AT-SPI queries run through a small
Python helper (pyatspi) invoked by the MCP server or a native D-Bus client;
(3) fallback policy when an app exposes no a11y tree (screenshot + VLM, or
refuse with an audited row).

---

## 4. Prompt-injection defense for MCP tool outputs (open Q5)

### 4.1 Fact — the threat is vendor-acknowledged

Fetched from the Anthropic computer-use demo README (2026-08-21):

> "In some circumstances, Claude will follow commands found in content even
> if it conflicts with the user's instructions. For example, instructions on
> webpages or contained in images may override user instructions or cause
> Claude to make mistakes. We suggest taking precautions to isolate Claude
> from sensitive data and actions to avoid risks related to prompt
> injection."

Same source lists the recommended precautions: dedicated VM/container with
minimal privileges; avoid sensitive data access; internet **allowlist**;
**human confirmation** for consequential actions (cookies, financial
transactions, terms of service).
Source: <https://github.com/anthropics/claude-quickstarts/tree/main/computer-use-demo>

### 4.2 Fact — OWASP ranks prompt injection #1 (LLM01)

Fetched from OWASP (2026-08-21): the OWASP **GenAI LLM Top 10 2026**
(published August 4, 2026) is the current release, maintained under the
**OWASP GenAI Security Project** (canonical source:
`GenAI-Security-Project/GenAI-LLM-Top10`). Historical v1.1 archive still
lists **LLM01 Prompt Injection** ("manipulating LLMs via crafted inputs can
lead to unauthorized access, data breaches, and compromised
decision-making"), plus LLM02 Insecure Output Handling, LLM07 Insecure
Plugin Design, and LLM08 Excessive Agency — all directly relevant to an MCP
tool surface.
Citations:
- <https://owasp.org/www-project-top-10-for-large-language-model-applications/>
- <https://genai.owasp.org/llm-top-10/>

### 4.3 Fact — AIOS already has a *request-side* injection detector

The shipped rule-pack classifier already scans **arguments** for injection
fragments: `PROMPT_INJECTION_FRAGMENTS = ["ignore constitution", "skip
consent", "exfil", "no audit", "bypass pep"]`, fired by **R-11**
(tool_pattern `*`, c3, confidence 0.95 → refusal band), implemented
identically in TS and Python (cross-language invariant enforced by
`tests/test_classifier_smoke.py`). Source:
`code/aiosh-mcp/aiosh_mcp/classifier.py` (§R-11,
`_scan_arg_text_for_pi`).

**The gap (Fact):** R-11 scans what the *user/agent sends into a tool*
(`args`). Open Q5 is the mirror: what a *tool returns* (scraped page, HTTP
response, nmap banner, pcap strings) may contain injection text that flows
back into the model's context.

### 4.4 Proposal — output-side injection defense (mirror of Anthropic's classifier)

1. **`scan_output_for_pi(text)`** — a deterministic scanner (same rule-pack
   spirit as R-11, extended fragment list: instruction verbs like "ignore
   previous", "disregard", "now run", "override", "system:", "assistant:",
   plus `PROMPT_INJECTION_FRAGMENTS`) applied to **tool results** before
   they are returned to the agent loop. Marked **Proposal**: fragments need
   a tuning task to avoid false positives on legitimate pentest content
   (e.g. a webpage literally teaching prompt-injection attacks).
2. **Tag, don't silently strip** — on match, wrap the result in a
   `[untrusted-content]` block with a `pi_suspect: true` flag (and the
   matched fragment) so the *model* sees it is untrusted, rather than the
   server editing attacker content (editing can itself be spoofed).
3. **Audit every scan** — one audit row per tool result that triggers the
   scanner (consequential: it changes what the model sees), consistent with
   ADR-0035 §F-2 fail-open (write an honest row even if the scan itself
   fails).
4. **PEP-side guard for the worst case** — the classifier's refusal band
   (R-11) plus the **allowlist + human-confirmation** posture from §4.1:
   a tool result may *contain* injection, but the *actions* it tries to
   provoke still pass through the classifier gate, so a `pentest.*`/`gui.*`
   call with an injected target/args is refused before execution.
5. **Keep the cross-language invariant** — any new output scanner must ship
   in both TS and Python with byte-equal fixtures, per the existing
   cross-substrate rule (`test_classifier_smoke.py:cross_lang_invariant`).

**Decision needed:** whether output scanning is a new rule (R-13+) in the
shared rule pack (bumping `policy_revision` — `sprint-2-rule-pack-v1`
today) or a separate `output_sanitizer` module invoked at the agent-loop
boundary. Recommendation (**Proposal**): separate module — input policy
(classifier) and output hygiene (sanitizer) are different concerns and
should not share a revision counter.

---

## 5. Summary — what this unblocks

| Gap | Key facts (anchored) | Next action (task) |
|---|---|---|
| 1. Kali/ATT&CK taxonomy | ATT&CK **v19.2** current (Apr 28, 2026), 15 tactics; Kali menu is MITRE-ordered; 5 wrappers shipped | Spec + implement first 9 new wrappers (Sprint 3 item 3 slice) |
| 2. On-device inference | llama.cpp: OpenAI-compatible `llama serve`, 1.5–8-bit GGUF, CUDA/HIP/Metal/Vulkan/SYCL; Ollama: local + cloud, offline-capable | Inference adapter + `aiosh models` (Phase 0) |
| 3. AI ↔ desktop | KWin 6 scripting (move/resize/focus/tile); wlr virtual pointer/keyboard + foreign-toplevel; AT-SPI2 over D-Bus; computer-use ref is X11+VNC | `gui.*` MCP tool set (Phase 4) |
| 4. Prompt-injection defense | Anthropic acknowledges content-injection risk; OWASP LLM01 #1; R-11 covers request args only | Output-side scanner + audit (hardening) |

All decisions that need a spec/ADR are listed in the "Decisions needed"
blocks above; per the sequential-execution law, they become tasks in the
ledger rather than being implemented here.

## 6. Citations (all fetched/verified 2026-08-21)

1. <https://attack.mitre.org/resources/versions/> — ATT&CK v19.2 current.
2. <https://attack.mitre.org/> — Enterprise matrix, 15 tactics.
3. <https://www.kali.org/tools/> — live Kali tool taxonomy (MITRE-ordered).
4. <https://github.com/ggml-org/llama.cpp> — llama.cpp README (backends,
   quantization, `llama serve`).
5. <https://ollama.com/> — Ollama homepage (local+cloud, offline).
6. <https://develop.kde.org/docs/plasma/kwin/api/> — KWin 6.0 scripting API.
7. <https://develop.kde.org/docs/plasma/kwin/> — KWin scripting tutorial
   (kpackagetool6, kwriteconfig6, reconfigure).
8. <https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1>
   — window listing/control protocol.
9. <https://wayland.app/protocols/wlr-virtual-pointer-unstable-v1> — input
   injection protocol.
10. <https://wiki.freedesktop.org/www/Accessibility/> — AT-SPI2 stack.
11. <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/index.html>
    — at-spi2-core dev guide.
12. <https://www.w3.org/TR/core-aam-1.2/> — Core-AAM 1.2 (CRD 2026-08-05),
    platform API mappings incl. ATK/AT-SPI.
13. <https://github.com/anthropics/claude-quickstarts/tree/main/computer-use-demo>
    — computer-use reference; prompt-injection caution + precautions.
14. <https://owasp.org/www-project-top-10-for-large-language-model-applications/>
    — OWASP LLM Top 10 (LLM01..) → GenAI project.
15. <https://genai.owasp.org/llm-top-10/> — OWASP GenAI LLM Top 10 2026
    (Aug 4, 2026).
16. <https://modelcontextprotocol.io/introduction> — MCP overview
    (redirects to protocol 2026-07-28 docs).
17. <https://modelcontextprotocol.io/docs/concepts/architecture> — MCP
    architecture (hosts/clients/servers, layers, transports).
18. <https://www.anthropic.com/engineering/building-effective-agents> —
    Anthropic agent patterns (workflows vs agents; MCP).
