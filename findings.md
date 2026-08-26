# Findings

## 2026-08-21 — Rust rewrite of the shipping stack (user directive)

The user directed a **full rewrite of the shipping stack to Rust**. Completed:
`code/aiosh-rust/` (aiosh-core + aiosh-cli + aiosh-mcp) now implements the MCP
server, CLI, audit ring (SQLite WAL + hash chain), classifier (R-01..R-12),
PEP grant store, retention (rotation + bloom filter), pentest wrappers
(nmap/nikto/sqlmap/tshark/aircrack-ng), Landlock + seccomp sandbox, and the
Ollama/stub agent loop.

Key port notes:
- **Zero-warning `cargo build`**; **45 `cargo test` cases green**, including a
  port of the Python classifier fixture matrix (SC1..SC10) pinning
  byte-identical behavior with the legacy TS/Python substrates.
- rusqlite 0.32 removed `Connection::Clone`/`try_clone` — code needing a
  second handle now opens a second connection to the DB path. `dispatch` /
  `recorded_call` no longer take a dead `conn` parameter.
- Port bugs found & fixed: `is_ascii_lowercase()` is false for digits (test
  assertion); R-05a is confidence 0.85 → **caution** not refused (all three
  substrates agree); `MAX(segment_id)+1` needs `COALESCE` on an empty table;
  a tamper test must overwrite with a *different* value than the original
  `{}` or the hash chain legitimately stays valid.
- Smoke surface: `code/aiosh-rust/ci/rust_smoke.sh` (build + tests + MCP
  stdio wire contract + CLI status), wired into `ci/run_all_smokes.sh`
  ahead of the legacy suites.

Legacy TS (`code/aiosh-cli`) and Python (`code/aiosh-mcp`) trees are retained
as the reference cross-substrate contract, not the ship path.

## 2026-08-21 — Task Ledger Control ported to Rust (T-14/T-15, wired as T-16)

Per user directive ("make every task fully Rust"), the last Python-only
shipping piece — the **task ledger data model** — was ported to Rust:
- `code/aiosh-rust/aiosh-core/src/ledger.rs` — atomic `TASK_STATE.json`
  pointer (tmp+rename), append-only fsync'd `COMPLETIONS.jsonl`, flock
  single-writer lock, no-skip enforcement, block/unblock/skip,
  rebuild-from-events, invariant `check`. 5 new unit tests (50 total
  `cargo test`, zero warnings).
- Wired into the production CLI: **`aiosh task status|done|block|unblock|
  skip|rebuild|check`** (T-00016 integration surface), audited like every
  other subcommand.
- Cross-substrate parity proven both directions (Python reads
  Rust-written state; Rust reads Python-written state) and asserted in
  the `rust_smoke` CI suite. Python `tools/task_ledger.py` stays as the
  legacy reference/test oracle.

## 2026-08-21 — Four research gaps closed (NEW)

Research for the four "still owed" gaps was completed and consolidated in
`docs/research/AIOS-RESEARCH-GAPS-2026-08-21.md` (all citations fetched and
verified live on 2026-08-21):

1. **Kali / MITRE ATT&CK v19 taxonomy → MCP tool minimum.** Confirmed
   **ATT&CK v19.2** is current (Apr 28, 2026) with **15 tactics** (Defense
   Evasion split into Stealth + Defense Impairment). Live Kali menu fetched
   from kali.org/tools (MITRE-ordered). Proposed the first 9 new wrappers
   (`pentest.recon.dnsrecon`, `pentest.web.nuclei`, `pentest.web.ffuf`,
   `pentest.discovery.masscan`, `pentest.passwords.hydra`,
   `pentest.passwords.hashcat`, `pentest.postexploit.netexec`,
   `pentest.forensics.autopsy`, `pentest.report.pipal`) + a
   `pentest.<category>.<tool>` namespace rule. Feeds Sprint 3 item 3.
2. **On-device inference.** llama.cpp now lives at `ggml-org/llama.cpp`;
   ships an OpenAI-compatible `llama serve` + 1.5–8-bit GGUF quantization;
   Ollama is local+cloud and offline-capable. One OpenAI-compatible adapter
   covers all three Pillar C backends. Proposals: `inference` adapter,
   `aiosh models` command, GBNF grammar for tool-call schema.
3. **AI ↔ desktop hook.** KWin 6 scripting API (move/resize/focus/tile,
   workspace, slots, packaged scripts via kpackagetool6); Wayland
   `wlr-virtual-pointer`/keyboard for input injection and
   `wlr-foreign-toplevel-management` for window enumeration; AT-SPI2 over
   D-Bus is the semantic tree (GTK4/Qt talk AT-SPI directly). Proposed a
   `gui.*` MCP tool set: **semantic-first (AT-SPI), pixels-fallback**.
4. **Prompt-injection defense for MCP outputs.** Anthropic's computer-use
   docs confirm content-injection risk ("Claude will follow commands found
   in content"); OWASP LLM Top 10 still ranks prompt injection LLM01
   (2026 release Aug 4, 2026). Gap: shipped R-11 scans only *request args*,
   not *tool outputs*. Proposal: deterministic `scan_output_for_pi` that
   **tags** results as untrusted (never silently strips), audits each hit,
   and relies on the classifier gate for the provoked actions.

Every proposal is explicitly marked Proposal in the research note with a
"Decisions needed" block; per the sequential-execution law these become
ledger tasks, not this-session implementation.

## 2026-08-20 — v2 course correction

The product vision is restated:

> **A Linux system for ethical hacking on the inside, a Windows-style desktop on
> the outside, with AI as a first-class S-rank kernel subsystem that controls
> the whole system.**

Implications for prior findings (this section is the source of truth going forward):

- The `kernel/`, `src/`, `userland/`, `target/`, `tests/`, `scripts/`, `ci/`,
  `composer-mpep/` artifacts (boot-verified RISC-V microkernel prototype) are
  re-classified as **research substrate**. They remain in tree and inform the
  capability/IPC/scheduler designs in the userspace stack, but are no longer
  the shipping v2 path.
- The active critical path is now **Pillar C (S-rank AI subsystem first)**,
  not the microkernel boot pipeline.
- The microkernel SMP `sscratch`-drift issue remains a real bug worth fixing —
  the audit-ring primitive depends on the same context-restoration patterns —
  but it is no longer a blocking dependency for the user-facing product.
- The `AIOS-0080-T1 fabricated-completion` finding carries forward as a
  process warning: any v2 task claim needs boot/manifest evidence, not just
  ledger evidence.

Pre-2026-08-20 findings retained below for full traceability.

## Repository state
- The checkout has no commits; all files are untracked, so no prior Git baseline can be used for history.
- `mostimportanAIfolder/PROJECT_MANIFEST.yaml` is malformed around `project_status` and contains duplicate/stale status fields.
- `mostimportanAIfolder/TASK_DATABASE.json` contains literal control characters/newlines inside JSON strings and cannot be parsed by Node.js.
- `mostimportanAIfolder/DEPENDENCY_GRAPH.json` and `KNOWLEDGE_GRAPH.json` parse, but their metadata is older than the later ADR/task artifacts.

## Newest evidence to reconcile
- ADR-0026 records AIOS-0069 completed and sibling AIOS-0069-T1 open.
- ADR-0027 records AIOS-0070 completed and sibling AIOS-0070-T1 open.
- ADR-0028 records AIOS-0071 completed and sibling AIOS-0071-T1 open.
- The active manifests now select AIOS-0065-T2 and report 43/46 coverage; the three hardware-gated gaps remain explicit.

## Required repair approach
1. Use accepted ADRs and their explicit repository-update tables as canonical for the Codeguard track.
2. Preserve legacy tasks and unresolved work; do not delete history.
3. Rebuild TASK_DATABASE.json as valid JSON, normalize metadata counts from actual task statuses, and keep all task objects that can be recovered from the source.
4. Update manifest, health report, session report, and graph metadata to the same current snapshot.
5. Validate with Node.js parsing and invariant checks before touching kernel code.

- Unresolved dependency check after repair: none.

## AIOS-0080-T1 investigation (2026-08-04)
- User requested live QEMU validation before starting G4 keyboard-input dispatch closure.
- `bash ci/smoke.sh` completed cargo check and cargo build, then failed the headless QEMU assertion at `kernel/src/main.rs:5710` because `wasm_runtime_smoke` accepted a wrong-magic module. This is a pre-existing W1 baseline failure, not a G4 keyboard failure.
- Control-plane artifacts are contradictory: `mostimportanAIfolder/TASK_DATABASE.json` records AIOS-0080-T1 as COMPLETED with ADR-0032 evidence, while `tasks/INDEX.md` still lists AIOS-0080 as OPEN and current `kernel/src/main.rs` has no `keyboard_dispatch_smoke` or `gui::keyboard_dispatch_self_test` symbols. Source is the implementation authority for this continuation.
- Existing `kernel/src/input.rs` already decodes EV_KEY values 0/1/2 to Release/Press/Repeat and maps arrows plus I/S/Q; existing `kernel/src/gui.rs::apply_keyboard` handles press/repeat, arrow bounds, I/S/Q focus routing, and unknown-key no-op.
- Existing `kernel/src/main.rs` has GUI smoke count `ACTUAL_SMOKE_FNS = 54`, with G3 and W1 present, but no G4 smoke. The requested work should add the missing closure rather than trust stale task metadata.
- `make` is unavailable in the Windows shell, so GUI validation must use the equivalent direct `cargo` + `qemu-system-riscv64` commands from `kernel/Makefile`.
- No source edits made yet.

## Control-plane reconciliation + AIOS-0080-T1 G4 closure (2026-08-08)

- **Finding (critical)**: AIOS-0080-T1 was COMPLETED in TASK_DATABASE.json with ADR-0032 evidence that never existed (no smoke fn, no ADR file). The previous "completion" was fabricated during a repair.
- **Finding (latent boot panic)**: ACTUAL_SMOKE_FNS = 54 was unconditional while the COVERAGE matrix is cfg-gated (G1/G2/G3 rows flip to GAP on headless). A default headless boot computed 51 verified PASS rows and would panic the drift guard (54 != 51). The prior session's repair note claimed "headless 51 / gui 55" but the code was never changed. Fixed by cfg-gating the const (gui=55 / headless=51) while implementing G4.
- **Finding (stale graphs)**: both graphs stopped at AIOS-0077-T1; the 08-08 completions AIOS-0078-T1 (G3) + AIOS-0079-T1 (W1) were absent. Added nodes + edges.
- **Finding (stale index)**: ADR_INDEX.md listed 24 ADRs but ADR-0030 (G3) and ADR-0031 (W1) files existed without index rows. Caught up to 27.
- Resolution: implemented the real G4 closure (keyboard_dispatch_smoke + ADR-0032), verified both build profiles, and re-synced the entire control plane.
