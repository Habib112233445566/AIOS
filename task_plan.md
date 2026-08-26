# AIOS Active Task Plan — v2 (course correction 2026-08-20)

## Goal (v2, amended)

Deliver the user-stated vision:

> **A Linux system for ethical hacking on the inside, a Windows-style desktop on
> the outside, with AI as a first-class S-rank kernel subsystem that controls
> the whole system.**

This supersedes the v1 plan that focused on a from-scratch RISC-V microkernel
as the shipping target. The microkernel work is preserved as research
substrate but is no longer the shipping path.

## Phase ordering — v2 critical path

1. **Phase 0 — Pillar C spine (S-rank AI subsystem).** Pluggable inference
   backends; MCP JSON-RPC server; PEP capability grants; audit ring.
   **This is the NEW critical path.**
2. **Phase 1 — Pillar A wrappers.** Top Kali/Parrot/BlackArch tools
   exposed as MCP tools, MITRE ATT&CK category-aligned.
3. **Phase 2 — Pillar B installer.** Debian/Ubuntu LTS + KDE Plasma 6 with
   Windows-look theme + Wine 11 + Proton 11.
4. **Phase 3 — AI ↔ Pillar A integration.** Goal-driven recon-to-report
   pipeline orchestrated by the S-rank agent.
5. **Phase 4 — AI ↔ Pillar B integration.** GUI automation over Wayland /
   KWin / UIA / AT-SPI.
6. **Phase 5 — Hardening, cross-platform, release.**

## Status (live — 2026-08-21)

### 2026-08-23 — MILESTONE: CI Smoke Orchestration data model CLOSED 10/10 (T-00111..T-00120)

`tools/ci_suites.py` (registry: 19 suites, order-is-contract, timeouts)
+ `tools/ci_run.py` (production orchestrator: per-suite wall-clock
timeouts with process-group kill, bounded log tails, atomic
machine-readable run summary at `$AIOSH_CI_RESULTS`). `ci/run_all_smokes.sh`
is now a delegating shim; the registry is the single source. Hardening:
group-kill verified zero survivors; 5 MiB log → 12 B tail output.
Verified: full CI **19/19 PASS** through the new path (180515 ms) + W-suite
W1..W7 (`docs/tasks/evidence/T-00120-verify.md`). Ledger pointer: T-00121.

### 2026-08-23 — MILESTONE: Recovery & validation component CLOSED 10/10 (T-00101..T-00110)

`task validate` shipped on all four surfaces (Rust CLI, Rust MCP,
Python MCP reference, Python reference CLI): read-only integrity report
comparing live `TASK_STATE.json` against deterministic event-log replay —
drift/seq/pointer checks fatal, evidence existence+orphans warning-only;
report-only by design (rebuild stays sole repair). Hardening closed
security finding F-1 (evidence path confinement); full findings payload is
byte-parity across Rust-MCP and Python-MCP (modulo audit_id). Verified:
full CI **19/19 PASS** + cargo 82 tests
(`docs/tasks/evidence/T-00110-verify.md`). Ledger pointer: T-00111.

### 2026-08-22 — MILESTONE: Task Ledger Control epic CLOSED 10/10 (T-00011..T-00100)

Documentation component shipped and verified: `tools/check_task_docs.py`
(six structural doc-invariants C1..C6, capped reads, root-bounded link
containment), U-suite **20/20** incl. a blindness-sensitivity proof,
both suites permanent in CI. Docs: README §"Documentation invariants"
with live-verified example + limitations. Verified: full CI **19/19
PASS** + cargo 79 tests (`docs/tasks/evidence/T-00100-verify.md`).
All ten components now closed: data model, core service, CLI,
MCP/API, configuration, automated tests, security policy,
observability, documentation, recovery & validation ← next: T-00101.

### 2026-08-22 — Observability sub-epic CLOSED (T-00081..T-00090)

`aios.task {action:"metrics"}` / `aiosh task metrics` shipped and
verified: stable additive-only snapshot `{tasks, audit, config}`,
counts-only disclosure, grant-free read-only, one honest audit row per
call. Tests-first caught two real defects pre-review (wire accepted
task_id on metrics; CLI silently ignored stray operands) — both fixed
and pinned by the new permanent `metrics_smoke` suite (O1–O8).
Discoverability added to the published inputSchema enum on both
substrates; O(1) COUNT(*) hardening; SPEC §8.6 operator docs.
Verified: cargo 79 tests + **CI 17/17 PASS**
(`docs/tasks/evidence/T-00090-verify.md`). Ledger pointer: T-00091.

### 2026-08-22 — Security policy sub-epic CLOSED (T-00071..T-00080)

Root `SECURITY.md` shipped (OpenSSF criteria met; owner-provided
advisory channel; scope from six component reviews; 7d/90d CVD;
rule-pack governance) + permanent `security_policy` CI suite. Review:
no fabrications/secrets; links verified. **CI 16/16 PASS**
(`docs/tasks/evidence/T-00080-verify.md`). Ledger pointer: T-00081.

### 2026-08-22 — Automated tests sub-epic CLOSED (T-00061..T-00070)

New cross-surface matrix suite (`test_ledger_matrix_smoke.py`, M1–M8)
pins what per-surface tests cannot: one-grant-both-servers,
narrow-grant rejection, concurrent-writer lock-busy (bounded), config
propagation, grant expiry fail-closed, block/unblock. Wired into CI →
**15/15 suites PASS**. Suites hardened (subprocess timeouts,
holder kill-safety); suites themselves security-reviewed (no leaks/no
bypass). Design fact encoded: `rebuild` is lock-free by design.
Ledger pointer: T-00071.

### 2026-08-22 — Configuration sub-epic CLOSED (T-00051..T-00060)

Ledger knobs are now operator-configurable via six `AIOSH_LEDGER_*`
env vars (Twelve-Factor-aligned; defaults == shipped constants; loud
named errors; floors + 86400s lock ceiling; python parity). Exposed by
`aiosh task config`; deliberately NOT agent-exposable via MCP (D5).
Security-reviewed (no open bypass); documented SPEC §8.3. Verified:
79 cargo tests + all wire suites + **CI 14/14 PASS**
(`docs/tasks/evidence/T-00060-verify.md`). Ledger pointer: T-00061.

### 2026-08-22 — MCP/API surface sub-epic CLOSED (T-00041..T-00050)

The Python reference server now mirrors the Rust `aios.task` tool
(`aios_task`: 7 actions, one grant valid across BOTH substrates).
P-suite caught a real gating hole pre-review (`rebuild` mis-classified
read-only) — fixed and permanently pinned. Hardening: module caching,
audited loader failures, bool-id rejection. Verified: 77 cargo tests +
U/W/C/P suites + **CI 13/13 PASS**
(`docs/tasks/evidence/T-00050-verify.md`). SPEC §7 L5 RESOLVED, §8.2
added. Ledger pointer: T-00051.

### 2026-08-22 — CLI surface sub-epic CLOSED (T-00031..T-00040)

`aiosh task` unified onto `task_service::TaskCall` (one validation
source with MCP): strict grammar (u64≥1, non-empty note/reason,
4096/16 caps, dash-value rejection, `--` delimiter), per-subcommand
help; evidence-item cap added to core; **non-UTF-8 argv panic
eliminated** (lossy + audited). Security-reviewed (no open bypass);
documented in SPEC §8.1. Verified: 77 cargo tests, U1..U16, W1..W8,
C1..C9, **CI 12/12 PASS** (`docs/tasks/evidence/T-00040-verify.md`).
Ledger pointer: T-00041 (configuration component begins).

### 2026-08-22 — Core service sub-epic CLOSED (T-00021..T-00030)

The agent-facing ledger surface shipped end-to-end: `aios.task` MCP
tool behind classifier→PEP→audit (read-only status/check; grant-gated
mutations), D3 resolver repair, D4 rebuild replay in both substrates,
bounded lock wait, 1 MiB transport cap. Security-reviewed with no open
bypass; documented in `docs/SPEC-TASK-LEDGER.md` §7–§9; verified with
64 cargo tests + U1..U16 + W1..W8 + **CI 11/11 PASS**
(`docs/tasks/evidence/T-00030-verify.md`). Ledger pointer: T-00031
(CLI-surface sub-epic begins).

### 2026-08-22 — Task Ledger Control epic CLOSED (T-00011..T-00020)

The 10-task ledger-control epic is verified complete: data model
researched, specified, implemented in Rust (`aiosh-core/src/ledger.rs`),
wired as `aiosh task …`, audited, security-reviewed, hardened,
documented (`docs/SPEC-TASK-LEDGER.md`), and verified — full baseline
10/10 PASS with captured evidence
(`docs/tasks/evidence/T-00020-verify.md`). Ledger pointer: T-00021.
Known limitations L1–L5 recorded in the spec §7 as decisions-needed
(Rust default path resolution; rebuild-vs-skip pointer rewind; flock is
single-host only; evidence attested not validated; parity smoke covers
done+block flows).

### 2026-08-21 — FULL RUST REWRITE (user directive, SHIPPED)

The shipping stack is now **Rust** (`code/aiosh-rust/`), replacing the
TypeScript CLI + Python MCP server as the ship path. All Sprint 0-3
capabilities were ported and verified:
- **aiosh-core** — canonical JSON/sha256, audit ring (SQLite WAL + hash
  chain), classifier R-01..R-12, PEP grants, retention (rotation + bloom +
  verify --full), pentest wrappers (nmap/nikto/sqlmap/tshark/aircrack-ng),
  Landlock + seccomp sandbox, Ollama/stub agent loop.
- **aiosh-cli** — `aiosh` binary: `status`, `run`, `audit tail/verify/
  rotate/segments/seen`, `grant create/list/revoke`, `pentest`, `classify`,
  `agent`.
- **aiosh-mcp** — stdio JSON-RPC MCP server (initialize / tools/list — 12
  tools / tools/call) with every call routed through the classifier→PEP→
  audit gate.
- **Green:** zero-warning `cargo build`; 45 `cargo test` cases, including a
  port of the Python classifier fixture matrix (SC1..SC10) pinning
  byte-identical behavior with the legacy substrates; end-to-end smoke
  `code/aiosh-rust/ci/rust_smoke.sh` (build + tests + MCP wire contract +
  CLI status) wired into `ci/run_all_smokes.sh` first.
- The legacy TS (`code/aiosh-cli`) and Python (`code/aiosh-mcp`) trees are
  retained as the cross-substrate reference contract, not the ship path.

### Done in Sprint 0 (shipped, pre-Rust)
- MCP server skeleton + FastMCP stdio transport.
- aiosh-cli (TypeScript): `status`, `run`, `agent` (stub), `audit tail/verify`,
  `grant create/list/revoke`.
- Hash-chained append-only SQLite WAL audit ring.
- Cross-substrate canonical-JSON invariant (TS ↔ Python).
- 5-tool MCP manifest: `aios.fs.read`, `aios.process.list`,
  `aios.audit.tail`, `aios.audit.verify`, `aios.pentest.nmap` (stub).

### Done in Sprint 1 (shipped 2026-08-20)
- Real Pillar-A pentest wrapper set — five tools:
  `pentest.nmap`, `pentest.nikto`, `pentest.sqlmap`,
  `pentest.tshark`, `pentest.aircrack-ng`.
- Both surfaces (MCP and CLI) share the audit ring through the same
  canonical-JSON invariant.
- Every pentest tool call writes one chain-extending audit row.
- 5-suite pentest smoke (`tests/test_pentest_smoke.py`) passes:
  S1 no-grant → refused; S2 grant+no-binary → refused-no-binary;
  S3 scope.tools mismatch → refused; S4 scope.paths mismatch → refused;
  S5 chain integrity holds across TS+Python writers.
- CLI bridge `code/aiosh-cli/src/pentest.ts` exposed as
  `aiosh pentest {nmap|nikto|sqlmap|tshark|aircrack-ng} <args> --grant <id>`.
- Cross-substrate canonical-JSON bug fixed (TS now stores args_json in
  canonical form so nested undefined→null placeholders round-trip).

### Done in Sprint 1.5 (shipped 2026-08-20)
- Replaced the key-grep `cFlagsFor()` with a **deterministic rule-pack
  classifier** (`R-01`…`R-12`) in both TS and Python.
- `classify()` returns `{c_flags, rule_ids, evidence, overall_verdict,
  policy_revision}` — every fired rule contributes a stable rule ID,
  confidence, and human-readable evidence the audit row carries
  verbatim.
- `policy_revision` field (`sprint-1.5-rule-pack-v1`) makes classifier
  behavior version-stamped; any rule-pack change requires a bump.
- Cross-language invariant proven: 10/10 SC fixtures produce
  semantically-identical classifications in TS and Python (after
  numeric-format normalization); the 4 module-level lists are
  byte-equal.
- Bug caught and fixed during smoke: TS `equals` predicate was not
  resolving the `$DANGEROUS_BINS` sentinel, causing asymmetric
  `R-05a` firing between the two languages. Would have shipped
  silent refusals-by-default.
- New `aiosh classify <tool> [--target <t>] [--json-args '{...}']`
  CLI surface for user-driven checks.
- Formal spec: `docs/SPEC-CONSTITUTION-CLASSIFIER.md`.
- See `docs/SPRINT-0.md` §9 for full evidence trail.

### Sprint 2 — agent loop (SHIPPED + VERIFIED 2026-08-21)

The agent loop described below is **built and verified green**. The
`task_plan` text claiming "the remaining gap is the agent that calls
them" was stale relative to the tree — the agent already exists.

Shipped in code:
- `code/aiosh-cli/src/agent.ts` — Computer-Use loop
  (Observe → Think → Act → Loop), Ollama-0.22.1 backend with a
deterministic stub fallback, `classify()` preflight per tool call.
- `code/aiosh-mcp/aiosh_mcp/agent_bridge.py` — persistent MCP stdio
  client forwarding `tools/call` to the real `aiosh_mcp.server`.
- `aiosh agent <prompt>` CLI subcommand (Sprint-0 §2 stub now real).
- MCP dispatch gate (`_dispatch.py` + `server.py`) calls `classify()`
  on every tool — the ADR-0035 §D-4 boundary.

Verified 2026-08-21 (all smokes green after installing mcp/fastmcp +
npm deps and fixing a broken `node_modules/.bin/tsc` wrapper):
```
PASS: Sprint 1.5 classifier smoke (SC1..SC10 + cross-language)
PASS: aiosh-mcp smoke (TS↔Python chain, 9 tools)
PASS: aiosh-mcp Sprint 1 pentest smoke
PASS: aiosh run sandbox smoke (fail-open-with-audit)
PASS: aiosh demo smoke (D1 grant+scan · D2 no-grant refusal ·
      D3 classifier-first refusal)
```
Note: `test_demo_smoke.py` D1 reaches "attempted" but the host lacks
the `nmap` binary — the audited `outcome=refused 'nmap binary not on
PATH'` is the correct auditable answer, not a bug.

### Done in Sprint 3 (shipped 2026-08-21)

- **Audit-ring retention policy** (item 1 of the Sprint 3 queue):
  checkpointed segment rotation + per-segment bloom filters,
  implemented identically in both substrates.
  - New `audit_segments` table; rotation archives the oldest live rows
    byte-identically to `$AIOSH_HOME/audit-archive/segment-<id>.jsonl`,
    pins the file sha256, and records `{first/last row id, row_count,
    genesis_prev_hash, head_hash, bloom}`.
  - Rotation is archival, never destruction (P-2/O-4 compliant, RFC 9162
    §4.13 log-retirement pattern); the live chain re-anchors at the
    checkpoint head and the rotation event itself is an `audit.rotate`
    chain row (O-2). Rotation refuses to run on a broken chain.
  - `verify()` is anchor-aware on both substrates; `verify --full`
    replays every archive file (checksum + per-row re-hash + segment
    linkage) before the live walk.
  - `seen(hash)` answers live / maybe (bloom) / archive (exact scan) /
    no — no false negatives.
  - Surfaces: CLI `aiosh audit rotate [--keep N] [--dry-run]`,
    `audit segments`, `audit seen <hash> [--exact]`,
    `audit verify --full`; MCP `aios.audit.rotate` (PEP grant
    required), `aios.audit.segments`, `aios.audit.seen`,
    `aios.audit.verify(full)`.
  - Docs: `docs/research/AIOS-AUDIT-RING-RETENTION-2026-08-21.md`,
    `docs/SPEC-AUDIT-RETENTION.md`, ADR-0036.
  - Verified 2026-08-21: `tests/test_retention_smoke.py` R1–R7 all
    PASS (incl. TS-rotates→Python-verifies cross-substrate proof);
    all Sprint 0/1/1.5/2 smokes remain green.

### Active task (next)
**Sprint 3 — remaining hardening items.** With retention shipped, the
highest-value next steps (see Queued below) are: (2) the **`aiosh demo`
snap test** formalized into the CI suite; and (3) expanding the five
pentest wrappers toward the full Kali / MITRE ATT&CK v19 taxonomy.

### Queued (Sprint 2)
- **Sprint 3 (SHIPPED 2026-08-26): CI Smoke Orchestration observability (T-00181..T-00190)**.
  - Standardized CI health metrics via iosh ci metrics action, completing the Phase 0 integration matrix.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration security policy (T-00171..T-00180)**.
  - Documented CI orchestrator vulnerability boundaries and updated the repository knowledge index.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration automated tests (T-00161..T-00170)**.
  - Brought the legacy ci_run.py orchestrator under automated test coverage.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration configuration (T-00151..T-00160)**.
  - Implemented Twelve-Factor environment configurations for CI orchestration bounds and file paths.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration MCP/API surface (T-00141..T-00150)**.
  - Integrated ios.ci into the Rust MCP server routing table.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration CLI surface (T-00131..T-00140)**.
  - Formally verified the CLI surface integration (iosh ci) implemented preemptively during T-00128.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration core service (T-00121..T-00130)**.
  - Native Rust implementation of iosh ci check, strict JSON artifact validation, 1MB file bounds, and honest audit row emission.
- **Sprint 2 (SHIPPED + verified 2026-08-21):** Ollama-0.22.1 /
  Anthropic-Computer-Use agent loop over MCP, gated by the Sprint-1.5
  classifier. ADR-0035 §D-2 (MCP tools) was already wired; §D-4
  (classifier gate) is enforced both as an agent-loop preflight
  (`agent.ts`) and at the MCP dispatch boundary (`_dispatch.py`).
- `aiosh demo` end-to-end scripted engagement (snap test).
- **Landlock + seccomp-bpf wrapper around `aiosh run`** —
  shipped in §11 (sandbox.py + cli.ts wiring + 3-scenario smoke).
  Sprint-2 gap closed; remaining env-dependent work is hardening
  the host kernel (Landlock ≥ 5.13 + accepting new seccomp filters),
  not the sandbox code.
- **Research-note tying the shipped rule-pack classifier to the
  neuron / Dynamic Neural Topology substrate:** shipped as
  `docs/research/AIOS-CLASSIFIER-PRIMITIVE-AND-NEURAL-SUBSTRATE-2026-08-20.md`
  (8 sections, 14 sub-headings). The classifier is *deliberately
  separate* from the cognition engine — see the note's
  three-way split: deterministic safety boundary / pluggable agent
  / preserved-but-unshipped neuron substrate.
- **Audit-ring retention policy (rotation / bloom filter)** — SHIPPED
  2026-08-21 in Sprint 3 item 1. See `### Done in Sprint 3` below.
- Expand pentest set to the full Kali/Parrot tool taxonomy across
  MITRE ATT&CK v19 categories.
- Rule-pack expansion beyond `R-12` as new tools / new attack
  categories land (version-stamped via `policy_revision`).

### Research done (2026-08-21)

- **Four open research gaps closed** in
  `docs/research/AIOS-RESEARCH-GAPS-2026-08-21.md` (all anchors fetched
  live 2026-08-21): Kali/MITRE ATT&CK v19.2 taxonomy → 9 proposed new
  wrappers + namespace rule; on-device inference (llama.cpp `ggml-org`,
  OpenAI-compatible `llama serve`, Ollama local/cloud); AI ↔ desktop hook
  (KWin 6 scripting + wlr virtual input + AT-SPI2, semantic-first
  `gui.*` set); prompt-injection defense for MCP *outputs* (R-11 covers
  args only; propose tagged `scan_output_for_pi`). Each carries a
  "Decisions needed" block → becomes ledger tasks per the no-skip law.

## Immediate actions for the next agent session

1. Read `docs/SPRINT-0.md` (Sprint 0 + Sprint 1 sections) — full
   shipped contract. Sprint 3 retention is documented in
   `docs/SPEC-AUDIT-RETENTION.md` + ADR-0036.
2. Read ADR-0035 (S-rank agent architecture) and Pillar-C clauses,
   plus ADR-0036 (audit-ring retention).
3. Pick a task — retention is SHIPPED (2026-08-21, verified green).
   Next queue items: formalize the `aiosh demo` snap test into a CI
   suite, then expand the five pentest wrappers toward the full
   Kali / MITRE ATT&CK v19 taxonomy.
4. Confirm baseline is green before starting (verified 2026-08-21):
   the **Rust** stack is the primary surface —
   `bash code/aiosh-rust/ci/rust_smoke.sh` (build + 45 tests + MCP wire
   contract + CLI status). The legacy suites
   (`python code/aiosh-mcp/tests/test_demo_smoke.py` AND
   `python code/aiosh-mcp/tests/test_smoke.py` AND
   `python code/aiosh-mcp/tests/test_pentest_smoke.py` AND
   `python code/aiosh-mcp/tests/test_retention_smoke.py` AND
   `bash code/aiosh-cli/tests/smoke.sh`) still run to pin the
   cross-substrate invariant.
5. Do not start implementation before research (see
   `mostimportanAIfolder/RESEARCH_EXECUTION_PROTOCOL.md`).

## Constraints

- Do not discard unrelated user changes.
- Do not change kernel code yet; v2 ships on Linux hosts, not the
  microkernel.
- Use repository evidence. ADRs and shipped code (with tests) are
  the source of truth for "what's done".
- Preserve unresolved/deferred tasks explicitly rather than silently
  deleting them.
- Cross-substrate canonical-JSON invariant MUST hold — the Rust
  `canonical()` (sorted keys, no whitespace, floats with `.0`) produces
  byte-identical hash chains with the legacy TS/Python substrates, and
  `code/aiosh-rust/aiosh-core/src/canonical.rs` documents the shared
  contract.

## Errors Encountered (Sprint 0/1)

| Error                                              | Cause                                       | Resolution                                                              |
|---------------------------------------------------|---------------------------------------------|-------------------------------------------------------------------------|
| `read_files` tool rejecting valid string              array | Tool-runtime parameter serialization quirk | Use `cat` via `run_terminal_command` for batch reads                      |
| Chain hash mismatch Sprint 1 row 1                | TS wrote args_json with stripped undefined keys, but canonicalJson hashed with them as null. | TS now writes args_json in canonical form too (`code/aiosh-cli/src/audit.ts`) |
| Sprint 0 cross-process tool-count equality        | Sprint 1 added 4 tools; the equality `actual != expected` broke | Relaxed to subset check: `expected <= actual`                            |
| `outcome: string` not assignable to OutcomeKind | `outcome` was a 3-valued narrow union       | Compute literal `"ok"|"refused"|"error"` and assign                       |
| `pentest.aircrack-ng` audit name with hyphen    | Python identifier can't carry hyphen        | Function named `aios_pentest_aircrack_ng`; audit tool hardcoded literal  |
| `spawned Bash Exit 1` on npx tsc from wrong dir  | shell cwd reset between commands            | `cd /c…and` chains each invocation                                       |

## Current decision

**Active track: Sprint 3 — remaining hardening items. Item 1
(audit-ring retention) SHIPPED 2026-08-21.**

Sprint 1 (pentest wrappers + cross-language audit), Sprint 1.5 (rule-pack
classifier + spec), Sprint 1.5b (Landlock/seccomp sandbox), Sprint 2
(Ollama/Anthropic agent loop over MCP, classifier-gated), and Sprint 3
item 1 (checkpointed segment rotation + bloom filter retention, ADR-0036)
are all shipped and verified green (2026-08-21).

**Next queue items:** (2) formalize `aiosh demo` snap test into CI
suite; (3) expand five pentest wrappers toward full Kali / MITRE
ATT&CK v19 taxonomy.

**Task-DB caveat:** `TASK_DATABASE.json` is a NON-authoritative,
graph-derived reconstruction (`authoritative: false`,
`provenance: graph-derived-recovery`) — its "89/89 COMPLETED" is an
artifact, not a real status. Source of truth for "what's done" is
repository evidence: ADRs + shipped code with green smokes (per
README *How to keep going*). The live v2/Sprint plan is this file.
