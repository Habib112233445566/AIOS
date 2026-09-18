# T-01531: Filesystem Layout - MCP/API Surface: Research

## Metadata
- **Task ID:** `T-01531`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout agent surface (`code/aiosh-rust/aiosh-mcp::main`, `aios.fs_layout.*`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (1/10) — MCP/API Surface Research
- **Dependencies:** `T-01530` (CLI Surface Verification & Evidence)
- **Next Task:** `T-01532` (Filesystem Layout / MCP/API surface: Specification)
- **Code changed:** none (research task)

---

## 1. Executive Summary & Problem Context

The Filesystem Layout data model (`T-01501..T-01510`), core service (`T-01511..T-01520`), and operator
CLI surface (`T-01521..T-01530`) are closed and verified. Task `T-01531` opens the **MCP/API Surface
Sub-Epic** (`T-01531..T-01540`).

The agent-facing surface is **not greenfield**: six `aios.fs_layout.*` tools already exist and are
covered by an in-tree unit test and a cross-surface parity smoke suite. The research question is
therefore not "what should we build?" but "what is already there, where does it diverge from the
hardened CLI it shares a store with, and what is genuinely missing?".

Three findings shape the rest of this sub-epic:

1. **The agent can read but not write layouts.** All six tools are non-mutating; the four store
   mutations shipped to the CLI in `T-01521..T-01530` (`register`, `set-active`, `remove`,
   `import-fstab`) have no MCP counterpart. An operator can create a layout that the agent can
   observe (proved by the FL6 parity suite) but the agent cannot author or activate one.
2. **The `T-01528` read-hardening was applied to the CLI only.** `aios.fs_layout.validate` and
   `aios.fs_layout.fstab` still resolve a spec path with `std::fs::read_to_string`, bypassing
   `read_bounded_text_file` — so the "regular file only / cap enforced during the read" defect class
   fixed on the CLI survives in the MCP server.
3. **The surface is undocumented.** `code/aiosh-mcp/README.md` documents `aios.session.*`,
   `aios.service.*`, and `aios.package.*` but contains no `fs_layout` mention at all.

---

## 2. Current Implementation Audit (verified facts)

### 2.1 Tool manifest — six tools, all non-mutating

| Tool | File:line | Mutates store? | PEP grant required? |
|---|---|---|---|
| `aios.fs_layout.get` | `aiosh-mcp/src/main.rs:996` (manifest), `:2991` (dispatch) | no | no (`require_grant = false`) |
| `aios.fs_layout.validate` | `:1008` / `:3011` | no | no |
| `aios.fs_layout.fstab` | `:1021` / `:3046` | no | no |
| `aios.fs_layout.list` | `:1034` / `:3080` | no | no |
| `aios.fs_layout.probe` | `:1046` / `:3100` | no | no |
| `aios.fs_layout.diff` | `:1060` / `:3128` | no | no |

Contrast with the same server's mutating session tools, which pass `require_grant = true`:
`aios.session.action` (`:2766`) and `aios.session.create` (`:2823`). Every fs_layout tool passes
`false`, matching its read-only behaviour.

### 2.2 Parameter surface

- `store_path` is advertised and honoured by `list`, `probe`, and `diff` (the three store-backed read
  tools). `resolve_fs_layout_service` (`:4168`) bounds it to ≤ 1024 characters, rejects control
  characters, falls back to the seeded presets when the file is absent, and returns an explicit error
  for a corrupt store — parity with the CLI, asserted by the FL6 smoke suite.
- `get` does **not** consult the store at all: its `profile` enum is
  `["standard_uefi", "minimal_container"]` (`:996`), so there is no way to fetch an arbitrary stored
  layout by id. The CLI equivalent is `aiosh layout show <ID> [--store <path>]`.
- `validate` and `fstab` accept `spec` as a path **or** inline JSON, and `validate` also accepts an
  inline `layout` object. `fstab` additionally accepts a `profile` selector.

### 2.3 Divergences from the hardened CLI

**F-1 — Spec reads are not hardened (`validate` `:3021`, `fstab` `:3053`).**

```rust
let content = std::fs::read_to_string(p).map_err(|e| format!("failed to read layout file: {}", e))?;
```

`T-01528` fixed exactly this class of defect on the CLI (findings H-4 and H-5): a directory, FIFO, or
character device named by `--spec` was refused by *type* through `read_bounded_text_file`
(`aiosh-core/src/fs_layout_service.rs:79`) instead of being read, and the 10 MiB ceiling was moved
from a metadata snapshot into the byte stream. The MCP server still uses the unbounded, type-blind
read, so:

- a FIFO named by `spec` blocks the MCP server's request loop indefinitely (an unauthenticated DoS on
  the agent transport, not just on one CLI invocation);
- `/dev/zero` streams until memory is exhausted;
- a document larger than the cap is read in full before parsing.

The server's 1 MiB request-line cap (`MAX_LINE_BYTES`, `:4213`) bounds *inline* payloads but not a
path handed to `read_to_string`.

**F-2 — No explicit payload ceiling on inline specs.** `aios.session.validate` and
`aios.session.create` perform an explicit 1 MiB spec check (`:2579`, `:2780`, `:2785`). The fs_layout
tools rely entirely on the transport line cap.

**F-3 — `probe` defaults to a hard-coded preset, not the active layout.** `layout_id` defaults to the
literal `"aios-uefi-standard-v1"` (`:3104`), and `diff`'s `source_id` defaults to the same literal
(`:3132`). After an operator calls `set-active` on a custom layout, an agent that probes without an
explicit `layout_id` silently evaluates the *preset* rather than the layout the operator just
activated. The CLI resolver's precedence (`T-01529` §4.0) ends at "the store's active layout"; the MCP
defaults diverge from it.

**F-4 — `get` cannot reach store layouts** (see §2.2), so the agent's only way to inspect a custom
layout's full body is `list` (all layouts, possibly large) or `diff`.

### 2.4 Audit, gating, and existing coverage

- Every tool body is wrapped in `dispatch::recorded_call` (`aiosh-core/src/dispatch.rs:213`), which
  runs the PEP verdict first and writes one audit row per call; a PEP refusal short-circuits the body
  (`:230`), so refused calls are still recorded.
- In-tree unit test `test_mcp_fs_layout_tools` (`aiosh-mcp/src/main.rs:5449+`) asserts the manifest and
  exercises `get`, `validate`, `fstab`, `list`, and the store-backed tools.
- Cross-surface suite `code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py` (331 lines, 5 groups,
  criterion **FL6**) proves: manifest discoverability incl. `store_path`; `fs-layout` alias parity
  with the CLI; byte-identical fstab text across surfaces; identical validate/diff/probe verdicts;
  shared-store visibility (CLI register + `set-active` observed by MCP, and the CLI observing the same
  store); and `store_path` failure-mode parity (missing → seeded presets, corrupt → explicit error,
  > 1024 chars and control characters → explicit error).
- `cargo test -p aiosh-mcp` is green (12 tests, verified in `T-01530`).

### 2.5 Documentation state

`grep -c fs_layout code/aiosh-mcp/README.md` → **0**. The README documents the session, service, and
package tool surfaces with request examples; the fs_layout surface has none.

---

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Technical Assumption |
|---|---|---|
| Tool inventory | Six `aios.fs_layout.*` tools exist (manifest `:996..:1073`); all dispatch through `recorded_call`. | These six are the intended *read* half of the surface; the mutation half was deferred to this sub-epic. |
| Gating | All six pass `require_grant = false`; `aios.session.action`/`create` pass `true`. | Read tools should stay ungated; mutation tools should require a grant (matching session precedent). |
| Store sharing | `list`/`probe`/`diff` accept `store_path` and read the same JSON the CLI writes; parity asserted by FL6. | Mutation tools must read-modify-write the *same* file with the same atomic-persistence guarantees the CLI relies on. |
| Default store | With no `store_path`, MCP operates on an in-memory store seeded with the two presets; nothing is persisted. | A future configuration task (sub-epic `T-01541..T-01550`) owns any canonical default path; this sub-epic should not invent one. |
| Persistence safety | `FilesystemLayoutService::save_to_path` stages with `O_CREAT \| O_EXCL`, fsyncs, then renames (T-01528 H-1..H-3, H-6). | Mutation tools should persist exclusively through that method — never write the store themselves. |
| Read hardening | CLI reads go through `read_bounded_text_file`; MCP `validate`/`fstab` use `read_to_string` (`:3021`, `:3053`). | F-1 is a real, exploitable divergence to be closed in this sub-epic, not merely a style difference. |
| Destructive-change awareness | `diff` returns `destructive: true` for partition deletion, shrink, or format change. | Mutation tools that can activate a destructive layout should surface that verdict rather than silently switching. |
| Test contract | FL6 asserts the tool set is discoverable and that the read tools require no grant. | Adding tools requires updating the FL6 manifest assertions and `test_mcp_fs_layout_tools`. |

---

## 4. Upstream Standards & Prior Art

1. **Model Context Protocol (JSON-RPC 2.0)**: tools are advertised through `tools/list` with a JSON
   Schema `inputSchema` and invoked through `tools/call`; the server in this repo additionally caps a
   request line at 1 MiB (`MAX_LINE_BYTES`, `:4213`) and answers oversize with `-32700`.
2. **AIOS `dispatch::recorded_call`**: the house pattern for a tool call — PEP verdict, then the body,
   then exactly one audit row; refusal is recorded and never fails open (ADR-0035 §F-2).
3. **Peer surfaces in this server**: `aios.session.*` is the closest precedent for a surface that owns
   both queries and mutations, with mutations gated by `require_grant = true` and `store_path`
   bounded to 1024 characters with control-character rejection.
4. **The CLI contract settled in `T-01529`**: the operator counterpart defines the mutation semantics
   (`register` validates FL1..FL5 before insertion, duplicate ids refused; `set-active` refuses
   unknown ids; `remove` refuses the active layout and the built-in presets; `import-fstab` parses
   `fstab(5)` rows, caps at 128 mounts, rejects content with no usable rows). The MCP mutation tools
   should be thin adapters over the *same* store methods (`register_layout`, `set_active_layout`,
   `remove_layout`, `import_fstab_as_layout`) so the two surfaces cannot drift.
5. **Linux `/etc/fstab`**: `fstab(5)` six-field rows; parsing/serialisation already lives in
   `aiosh-core::fs_layout` and is invariant-checked under `FL1..FL5`.

---

## 5. Unknowns and Decisions Needed

These are listed explicitly, as this task's acceptance requires; each is to be resolved by `T-01532`.

1. **D-1 — Does the agent get mutating layout tools at all?**
   *Options:* read-only forever; mutations exposed and gated; mutations exposed behind a separate
   capability. *Recommendation:* expose them, gated — the store is declarative metadata, never physical
   disk mutation, and the CLI already provides the semantics.
2. **D-2 — Which mutations ship in this sub-epic?**
   *Candidates:* `register`, `set-active`, `remove`, `import-fstab` (the CLI's `T-01521..T-01530` set).
   *Recommendation:* all four, as one tool each, mirroring the CLI 1:1.
3. **D-3 — Should mutation tools require a PEP grant?**
   *Recommendation:* `require_grant = true` for all four, matching `aios.session.action`/`create`;
   reads stay ungated so observation never needs a grant.
4. **D-4 — Does `get` widen to store layouts by id?**
   *Options:* widen `get` (additive `layout_id` + optional `store_path`); add a new `show` tool;
   leave as-is. *Recommendation:* widen `get` — a new tool would duplicate it and inflate the
   manifest, and the `profile` enum remains valid for the preset case.
5. **D-5 — What should `probe`/`diff` default to when no id is given?**
   *Recommendation:* the store's active layout (CLI parity), with the canonical preset only as the
   empty-store fallback. This is a behaviour change (F-3) and needs a test asserting it.
6. **D-6 — Do we close the read-hardening divergence (F-1) here or in the hardening task?**
   *Recommendation:* fix it as part of implementation (`T-01534`) and re-prove it in `T-01537`/`T-01538`
   — leaving a known unbounded, blocking read in a long-lived server process is worse than the
   scoped churn of routing both call sites through `read_bounded_text_file`.
7. **D-7 — Do mutations need a destructive-change guard?**
   *Recommendation:* `set-active` should return the `destructive` verdict of the implied transition
   (or at minimum the previous/new ids) so the agent's audit row records what it switched to; hard
   refusal is deferred unless the spec review asks for it.
8. **D-8 — Should any tool write files (e.g. render fstab to a path)?**
   *Recommendation:* no. `fstab` returns content; writing is the operator's decision, and a
   server-side write would reopen exactly the symlink/`O_EXCL` surface `T-01528` closed.
9. **D-9 — Default store path for the agent surface.** Left open deliberately: `T-01541+`
   (configuration) owns defaults. Until then, omit `store_path` and the MCP surface operates on
   in-memory presets (current, documented behaviour).
10. **D-10 — Test-contract updates.** Adding tools invalidates the FL6 "all 6" assertions and
    `test_mcp_fs_layout_tools`; both must be extended in the same change, and the MCP README must
    document the new surface (F-3 documentation gap).

---

## 6. Constraints Carried Into the Sub-Epic

- Declarative metadata only: no partitioning, formatting, or mount execution.
- All persistence goes through `FilesystemLayoutService::save_to_path` (atomic, `O_EXCL` staging,
  fsync, staged-file preservation on rename failure).
- Every tool call must produce exactly one audit row, including PEP refusals.
- Inline payloads are bound by the 1 MiB request line; file-backed reads must be bound by
  `read_bounded_text_file` with `MAX_LAYOUT_DOC_BYTES` (10 MiB).
- `store_path` bounds (≤ 1024 characters, no control characters) are already enforced by
  `resolve_fs_layout_service` and must be preserved by every new tool.
- No new API is invented: all mutation semantics reuse existing `aiosh-core` store methods.

---

## 7. Citations & Bibliography

1. Model Context Protocol specification — tools (`tools/list`, `tools/call`), JSON-RPC 2.0 transport.
2. `fstab(5)`, `lsblk(8)`, `findmnt(8)`, `fdisk(8)` — `util-linux` man pages.
3. Repository sources audited: `code/aiosh-rust/aiosh-mcp/src/main.rs` (`:996..:1073`, `:2766`,
   `:2823`, `:2991..:3160`, `:4168`, `:4213`), `code/aiosh-rust/aiosh-core/src/fs_layout_service.rs`
   (`:79`), `code/aiosh-rust/aiosh-core/src/dispatch.rs` (`:213`),
   `code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py`.
4. Prior evidence: `T-01528-cli-surface-hardening.md` (H-1..H-7, platform skips),
   `T-01529-cli-surface-documentation.md` (mutation semantics, exit-code contract),
   `T-01530-cli-surface-verification-evidenc.md` (baseline green).
5. ADR-0035 — audit invariants and fail-open recording (`§F-2`).
