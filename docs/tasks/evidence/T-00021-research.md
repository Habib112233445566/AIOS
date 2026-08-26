# T-00021 — Task Ledger Control: core service Research

**Date:** 2026-08-22
**Type:** research (no code changed)
**Depends on:** T-00020 (complete)
**Artifact note:** instructions specify `T-00021-research.md`; the ledger
row's `artifacts` field declares `T-00021-core-service-research.md`
(mirrored byte-for-byte so the declared artifact exists).

---

## 1. What "core service" means in this ledger (template fact)

`tools/generate_master_tasks.py` generates every epic from a fixed
10-step component lifecycle: data model → **core service** → CLI
surface → … For the Task Ledger Control epic, the data-model step
(T-00011..T-00020) already shipped `aiosh-core/src/ledger.rs` +
the `aiosh task` CLI surface. The core-service step (T-00021..T-00030)
must therefore define and build the *service layer* of ledger control —
the question this research answers is: **what is the service, who calls
it, and through which gate?**

## 2. Internal facts (read from the tree, 2026-08-22)

| # | Fact | Anchor |
|---|---|---|
| F1 | Ledger control exists as a Rust library: `complete/block/unblock/skip/rebuild/check/load_state`, flock-guarded, atomic state writes | `code/aiosh-rust/aiosh-core/src/ledger.rs` |
| F2 | The only production surface is the CLI: `aiosh task status\|done\|block\|unblock\|skip\|rebuild\check`; every outcome writes one honest audit row (`task.ledger`) | `code/aiosh-rust/aiosh-cli/src/main.rs::cmd_task` |
| F3 | The MCP server exposes exactly **12 tools** — fs.read, process.list, audit.tail/verify/rotate/segments/seen, pentest.nmap/nikto/sqlmap/tshark/aircrack-ng. **No `aios.task.*` tools exist** | `code/aiosh-rust/aiosh-mcp/src/main.rs` (tool_manifest/call_tool) |
| F4 | The gate pattern every MCP call must use: `dispatch::dispatch` (classifier→PEP verdict) then `dispatch::commit`, or the combined `dispatch::recorded_call(ring, pep, tool, command, args, target, grant_id, require_grant, actor_id, actor, f)` | `aiosh-core/src/dispatch.rs:51,175,213` |
| F5 | ADR-0035 §D-2 binding: **MCP is the ONLY tool-call protocol AIOS exposes to external models** | `mostimportanAIfolder/ADR-0035-aios-s-rank-agent-architecture.md` |
| F6 | ADR-0035 §F-2: fail-open behavior must always write an honest audit row | same ADR |
| F7 | The wire-contract smoke asserts `len(names) == 12` — adding task tools changes that assertion (a CI contract change, not just additive code) | `code/aiosh-rust/ci/rust_smoke.sh:60` |
| F8 | Known limitations carried from the data model: L1 single-host flock single-writer; L2 Rust default `current_exe()` path resolution misses `<repo>/docs/tasks` (operators must set `AIOSH_TASKS_DIR`); L3 `rebuild` rewinds pointer onto skipped tasks (`max(completed)+1`, both substrates); L4 evidence attested not validated; L5 parity smoke covers done+block flows only | `docs/SPEC-TASK-LEDGER.md` §7 |
| F9 | The Python legacy MCP server also has no task tools (its manifest mirrors Sprint-0..3 surfaces) | `code/aiosh-mcp/aiosh_mcp/server.py` |

## 3. External authoritative facts (fetched live 2026-08-22)

Source: Model Context Protocol — Tools,
<https://modelcontextprotocol.io/docs/concepts/tools>

| # | Fact |
|---|---|
| E1 | Tool names SHOULD be 1–128 chars; allowed characters include ASCII letters, digits, underscore, hyphen, **and dot** — so `aios.task.status` is spec-conformant (matches the existing `aios.audit.rotate` style) |
| E2 | Every tool MUST carry an `inputSchema` that is a valid JSON Schema object; parameter-less tools should use `{ "type": "object", "additionalProperties": false }` (recommended) or `{ "type": "object" }` |
| E3 | Servers SHOULD return tools in deterministic order (prompt-cache friendliness) |
| E4 | Two error channels: unknown/malformed tool ⇒ JSON-RPC protocol error (-32601/-32602); business-logic failures (e.g., NO-SKIP refusal) ⇒ normal result with `isError: true` so models can self-correct. Our server maps `ok:false` → `isError:true` today |
| E5 | Security: servers **MUST** validate all tool inputs, implement proper access controls, rate-limit invocations, sanitize outputs; human-in-the-loop for sensitive operations is a SHOULD |
| E6 | The current spec revision shown in the docs is `2026-07-28`; our server pins `protocolVersion "2025-06-18"` (client-driven echo). Pinning is compatible behavior but worth recording |

## 4. Gap analysis

The agent loop (`aiosh agent`) can call pentest/audit/fs tools over MCP,
but **cannot read or advance the project's own task ledger** except by
shelling out (which bypasses the MCP gate and violates the spirit of
F5). Conversely, humans have full CLI control (F2). The missing piece a
"core service" would supply is a **gated, audited service layer for
ledger control reachable from the agent loop**, sharing the exact
library code the CLI uses (F1) so there is one implementation of the
no-skip law.

### Candidates considered

| Candidate | Verdict | Reasoning |
|---|---|---|
| **A. Service module in `aiosh-core` + `aios.task.*` MCP tools** through the existing classifier→PEP→audit gate | **Recommended proposal (AIOS-specific)** | Directly satisfies F5 (MCP-only protocol), reuses F1/F4 patterns verbatim, gives the Pillar-C spine self-management of its own work queue |
| B. Standalone ledger daemon (socket/service) | Rejected | Zero prior art in the repo; contradicts the minimal stdio architecture; adds a second writer path against L1 |
| C. Status quo (CLI-only) | Rejected for this epic | Leaves the generated "core service" component without a deliverable and the gap above open |

## 5. Assumptions (clearly marked, not facts)

- A1 (assumption): exposing ledger mutation to the model is desirable at
  all. Mitigation if wrong: expose read-only `aios.task.status/check`
  first and keep mutations CLI/human-only — the decision below (D1/D2)
  resolves this before any scaffold exists.
- A2 (assumption): one grouped tool (`aios.task` with an `action`
  argument) vs seven narrow tools — either is spec-conformant (E1/E2);
  choice affects the F7 wire-count assertion and prompt footprint.

## 6. Decisions needed before Specification (T-00022)

- **D1 — Grant policy:** which of
  `status/check/done/block/unblock/skip/rebuild` are read-only
  (`require_grant=false`) vs consequential (`require_grant=true`,
  mirroring `aios.audit.rotate`)?
- **D2 — Surface shape:** seven `aios.task.<sub>` tools vs one grouped
  `aios.task` tool with `action` arg (E2/A2); updates F7 assertion and
  `MASTER_TASK_LEDGER` docs accordingly.
- **D3 — Scope of L2 fix:** fold the `current_exe()` path-resolution
  repair into this epic's Implementation task (code change) or keep it
  as a documented limitation?
- **D4 — Scope of L3 fix:** should `rebuild` replay `pointer_reset`
  events so skips survive rebuilds (behavior change, needs cross-substrate
  parity work), or remain rewind-by-design?
- **D5 — Protocol pin:** confirm `"2025-06-18"` remains the pinned
  protocolVersion for this epic despite E6.
- **D6 — Rate limiting (E5 MUST):** record how the requirement is met
  for local stdio task tools (e.g., document that the single-writer
  flock serializes mutations and the CLI/MCP host owns throttling, or
  add an explicit cap).
- **D7 — Human-in-the-loop (E5 SHOULD):** whether `done`/`skip` require
  an operator-side confirmation channel beyond the PEP grant.

## 7. Acceptance check

- [x] Evidence file separates facts (F1–F9 internal, E1–E6 external)
      from assumptions (A1–A2) and proposals (Candidate A, marked
      AIOS-specific).
- [x] Citations given for external sources with fetch date.
- [x] Unknowns and decisions needed listed explicitly (D1–D7).
- [x] No code changed in this task.
