# T-01532: Filesystem Layout - MCP/API Surface: Specification

## Metadata
- **Task ID:** `T-01532`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout agent surface (`code/aiosh-rust/aiosh-mcp::main`, `aios.fs_layout.*`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (2/10) — MCP/API Surface Specification
- **Dependencies:** `T-01531` (MCP/API Surface Research)
- **Next Task:** `T-01533` (Filesystem Layout / MCP/API surface: Scaffold)
- **Code changed:** none (specification task)

---

## 1. Scope & Objective

This specification freezes the contract for the agent-facing Filesystem Layout surface: the JSON-RPC
tool names, per-tool input schemas, output shapes, failure modes, persistence effects, and audit
effects. It is reviewable without reading the implementation.

It specifies:

- the **six existing read tools**, restated as they will behave after this sub-epic (three of them
  change: `get` is widened, `probe`'s default changes, and `validate`/`fstab` lose an unhardened read);
- **four new mutation tools** that give the agent the authoring capability the CLI has had since
  `T-01524`;
- the envelope, error, and audit contracts shared by all ten.

Out of scope: any canonical default store path (owned by the configuration sub-epic,
`T-01541..T-01550`), physical partitioning/formatting, and file-writing tools (see §3, D-8).

Roadmap position — after this sub-epic the surface is:

| # | Tool | Kind | Mutates store | Grant required |
|---|---|---|---|---|
| 1 | `aios.fs_layout.get` | read (widened) | no | no |
| 2 | `aios.fs_layout.validate` | read (hardened) | no | no |
| 3 | `aios.fs_layout.fstab` | read (hardened) | no | no |
| 4 | `aios.fs_layout.list` | read | no | no |
| 5 | `aios.fs_layout.probe` | read (default changed) | no | no |
| 6 | `aios.fs_layout.diff` | read | no | no |
| 7 | `aios.fs_layout.register` | **new** mutation | yes | **yes** |
| 8 | `aios.fs_layout.set_active` | **new** mutation | yes | **yes** |
| 9 | `aios.fs_layout.remove` | **new** mutation | yes | **yes** |
| 10 | `aios.fs_layout.import_fstab` | **new** mutation | yes | **yes** |

---

## 2. Reused vs. New Interfaces

### 2.1 Reused (unchanged, already shipped)

| Interface | Location | Role in this surface |
|---|---|---|
| `dispatch::recorded_call` | `aiosh-core/src/dispatch.rs:213` | PEP verdict + exactly one audit row per call, refusal short-circuit (`:230`) |
| `dispatch::dispatch`, `Verdict::to_json` | `aiosh-core/src/dispatch.rs:36` | Gate and refusal envelope |
| `PepStore`, `grant_id` | `aiosh-core/src/pep.rs` | Authorization for mutation tools |
| `FilesystemLayoutStore::register_layout` | `fs_layout_service.rs:319` | Validates `FL1..FL5`, rejects duplicate ids, seeds the active pointer when empty |
| `FilesystemLayoutStore::set_active_layout` | `:371` | Rejects unknown ids |
| `FilesystemLayoutStore::remove_layout` | `:342` | Refuses the active layout and the two built-in presets |
| `FilesystemLayoutService::import_fstab_as_layout` | `:631` | Parses `fstab(5)` rows, ≤ 128 mounts, inherits from a base layout |
| `FilesystemLayoutService::save_to_path` / `load_from_path` | `fs_layout_service.rs` | Atomic `O_EXCL` staging + fsync + rename; 10 MiB bounded, type-checked load |
| `read_bounded_text_file`, `MAX_LAYOUT_DOC_BYTES` | `:79` | Regular-file-only, stream-bounded document reads |
| `resolve_fs_layout_service` | `aiosh-mcp/src/main.rs:4168` | `store_path` bounds (≤ 1024 chars, no control chars), missing-store fallback, corrupt-store error |
| `FilesystemLayoutSpec`, `probe_target`, `diff_layouts`, `generate_fstab` | `fs_layout.rs`, `fs_layout_service.rs` | Layout semantics |
| `MAX_LINE_BYTES = 1 MiB` transport cap | `aiosh-mcp/src/main.rs:4213` | Bounds inline payloads |

### 2.2 New (all AIOS-specific)

The Model Context Protocol is upstream; **every tool name below is AIOS-specific** and none is an
upstream-standard API. New work is limited to:

1. `aios.fs_layout.register`, `set_active`, `remove`, `import_fstab` — four AIOS-specific tools.
2. Widening of the AIOS-specific `aios.fs_layout.get` to resolve a stored layout by id.
3. Routing the `validate`/`fstab` spec reads through the existing `read_bounded_text_file`.
4. Changing `probe`'s default `layout_id` to the store's active layout.
5. An explicit 1 MiB inline-payload guard on `validate`, `fstab`, and `register`.

No new core-service method is required: every mutation is a thin adapter over an existing store
method, so the CLI and MCP cannot drift semantically.

---

## 3. Resolved Decisions (from `T-01531` §5)

| ID | Decision | Resolution |
|---|---|---|
| D-1 | Do agents get mutating layout tools? | **Yes.** The store is declarative metadata; no disk is touched. |
| D-2 | Which mutations? | **All four**: `register`, `set_active`, `remove`, `import_fstab`, mirroring the CLI 1:1. |
| D-3 | Grant requirement | **Mutations require a PEP grant** (`require_grant = true`); reads stay ungated. |
| D-4 | `get` widening | **Widen `get`** with `layout_id` (+ optional `store_path`); `profile` remains valid and keeps its current default, so existing callers are unaffected. A separate `show` tool is rejected as duplication. |
| D-5 | `probe`/`diff` defaults | **`probe` defaults to the store's active layout** (CLI parity); **`diff` keeps the two preset defaults** — this matches the CLI, whose `resolve_layout` falls back to the active layout while `diff` defaults to `aios-uefi-standard-v1` → `aios-container-minimal-v1`. |
| D-6 | Close the read-hardening divergence (F-1) | **In this sub-epic** (`T-01534`), re-proved by the hardening task. A blocking, unbounded read inside a single-threaded server loop is a transport DoS, not a style issue. |
| D-7 | Destructive-change guard | **Report, do not refuse.** `set_active` returns `previous_active`, `active`, and `destructive_transition` (the `diff_layouts` verdict of previous → new, or `null` when it cannot be computed). Hard refusal is deferred. |
| D-8 | Tools that write files | **None.** `fstab` returns content; writing reopens the symlink/`O_EXCL` surface closed in `T-01528`. |
| D-9 | Default store path | **Deliberately open.** Owned by `T-01541+`. Mutation tools therefore require an explicit `store_path` (§4.7.1). |
| D-10 | Test-contract updates | FL6's "all 6 tools" assertions, `test_mcp_fs_layout_tools`, and the MCP README are updated in the same change (implementation `T-01534`, docs `T-01539`). |

### 3.1 Consequence of D-9 for mutations: `store_path` is required

The CLI mutation commands without `--store` report success against a throw-away in-memory store
(documentation `T-01529` §6.7). For an agent that is not merely unhelpful but misleading — the call
would appear to succeed while the process exits and discards it. Therefore:

> `store_path` is a **required** parameter of the four mutation tools. Omitting it is an error:
> `"store_path is required for mutating fs_layout tools (no canonical default store is defined yet)"`.

Read tools keep `store_path` optional (missing store → seeded presets). When the configuration
sub-epic defines a canonical default store, this requirement can be relaxed to a defaulted parameter
without changing any other part of the contract.

---

## 4. Tool Specifications

Common conventions:

- Every tool accepts an optional `grant_id` (string) for PEP authorization.
- All parameters are bounded strings; `store_path` follows `resolve_fs_layout_service` (≤ 1024 chars,
  no control characters).
- Successful bodies include `ok: true` and `tool`; failures produced by the tool body include
  `ok: false`, `tool`, and `error` (string). Either way the body gains `audit_id` and
  `classifier_policy_revision`.
- No tool spawns a process, writes a file other than the layout store, or touches disk beyond reads
  of regular files.

### 4.1 `aios.fs_layout.get` (read — widened)

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `layout_id` | string | no | — | **New.** Return the layout with this id from the store. |
| `profile` | enum(`standard_uefi`, `minimal_container`) | no | `standard_uefi` | Existing behaviour, unchanged. |
| `store_path` | string | no | in-memory presets | Used only when `layout_id` is given. |

- **Precedence:** `layout_id` > `profile`.
- **Behaviour:** with `layout_id`, load the store (via `resolve_fs_layout_service`) and return that
  layout; unknown id is an error. Otherwise return the named canonical preset without consulting the
  store — byte-identical to today's output, so existing callers are unaffected.
- **Success:** `{ok, tool, layout: <FilesystemLayoutSpec>}`.
- **Errors:** `"layout with id '<id>' not found in store"`; `store_path` bound violations.

### 4.2 `aios.fs_layout.validate` (read — hardened)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `spec` | string | no | Path to a regular file **or** inline JSON. |
| `layout` | object | no | Inline layout object (takes precedence over `spec`). |
| `store_path` | string | no | Unused except for bounds parity; retained for signature stability. |

- **Behaviour:** resolve the layout (defaulting to `standard_uefi`), then validate against `FL1..FL5`.
- **Hardening (D-6):** when `spec` names an existing path, it is read with
  `read_bounded_text_file(path, MAX_LAYOUT_DOC_BYTES, "spec file")`; a directory, FIFO, or character
  device fails by type and the 10 MiB ceiling is enforced on the byte stream. Inline `spec`/`layout`
  payloads larger than 1 MiB are rejected explicitly before parsing.
- **Success:** `{ok: true, tool, id, valid: true, error: null}`.
- **Failure (invalid layout):** `{ok: false, tool, id, valid: false, error: "<FL1..FL5 violation>"}`.
- **Failure (bad input):** `{ok: false, tool, error: "..."}` for unparseable JSON, unreadable or
  non-regular files, and oversize documents.

### 4.3 `aios.fs_layout.fstab` (read — hardened)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `profile` | enum(`standard_uefi`, `minimal_container`) | no | Preset selector when `spec` is absent. |
| `spec` | string | no | Path or inline JSON; takes precedence over `profile`. |

- **Behaviour:** identical to today, except the path read goes through `read_bounded_text_file`.
- **Success:** `{ok: true, tool, id, fstab: "<6-field fstab text>"}`.
- **Failure:** `{ok: false, tool, error}` for parse/read/type/size problems. The tool never writes the
  content anywhere (D-8).

### 4.4 `aios.fs_layout.list` (read — unchanged)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `store_path` | string | no | Missing file → in-memory seeded presets. |

- **Success:** `{ok: true, tool, active_layout_id, count, layouts: [...]}`.
- **Failure:** `{ok: false, tool, error}` for a corrupt or unbounded `store_path`.

### 4.5 `aios.fs_layout.probe` (read — default changed)

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `layout_id` | string | no | **store active layout** (was the literal `aios-uefi-standard-v1`) | |
| `target_disk_bytes` | integer | no | `68719476736` (64 GiB) | |
| `store_path` | string | no | in-memory presets | |

- **Behaviour (new):** when `layout_id` is omitted, use the store's active layout. This matches the
  CLI, whose resolver falls back to the active layout. When the store is empty of custom layouts the
  active id is `aios-uefi-standard-v1`, so the previous observable behaviour is preserved in the
  default case.
- **Success:** `{ok: eval.is_viable, tool, evaluation: {...}}` (unchanged shape).
- **Failure:** unknown `layout_id` → `{ok: false, tool, error}`.

### 4.6 `aios.fs_layout.diff` (read — defaults unchanged)

| Parameter | Type | Required | Default |
|---|---|---|---|
| `source_id` | string | no | `aios-uefi-standard-v1` |
| `target_id` | string | no | `aios-container-minimal-v1` |
| `store_path` | string | no | in-memory presets |

- **Behaviour:** unchanged; defaults deliberately mirror the CLI's `diff` defaults (D-5).
- **Success:** `{ok: true, tool, diff: {..., destructive: bool}}`.
- **Failure:** unknown source/target → `{ok: false, tool, error}`.

### 4.7 Mutation tools — shared contract

#### 4.7.1 Preconditions (all four)

| Rule | Failure |
|---|---|
| `store_path` present, ≤ 1024 chars, no control characters | `"store_path is required for mutating fs_layout tools (no canonical default store is defined yet)"` / `"store_path exceeds 1024 characters or contains control characters"` |
| PEP verdict passes (`require_grant = true`, `grant_id` supplied) | verdict refusal envelope (§5.3) |
| Store loads (absent file → seeded presets; corrupt file → error) | `"failed to deserialize layout store..."` surfaced verbatim |
| Exactly one `save_to_path` call on success | a save failure is returned as the call's error and no partial state is left visible |

#### 4.7.2 `aios.fs_layout.register` (mutation)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `layout` | object | one of | Inline layout object. |
| `spec` | string | one of | Path (regular file only) or inline JSON. |
| `store_path` | string | **yes** | Persisted target. |

- **Behaviour:** parse → `FilesystemLayoutSpec::from_json`/`from_value` (which validates `FL1..FL5`) →
  `store_mut().register_layout(spec)` → `save_to_path`.
- **Success:** `{ok: true, tool, id, registered: true, active_layout_id}`.
- **Errors:** missing both inputs (`"register requires a 'layout' object or a 'spec' string"`);
  duplicate id (`"layout with id '<id>' is already registered"`); invariant violation from `FL1..FL5`;
  unreadable/non-regular/oversize `spec`; save failure.
- **Persistence:** exactly one atomic store write.

#### 4.7.3 `aios.fs_layout.set_active` (mutation)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `layout_id` | string | **yes** | Must already exist in the store. |
| `store_path` | string | **yes** | |

- **Behaviour:** load → capture `previous_active` → `set_active_layout(layout_id)` → `save_to_path` →
  compute `destructive_transition` by diffing previous → new (D-7).
- **Success:** `{ok: true, tool, previous_active, active, destructive_transition: bool|null}`.
- **Errors:** missing `layout_id`; unknown id (`"layout with id '<id>' not found in store"`); save
  failure.
- **Note:** `destructive_transition` is informational — an agent switching to a layout that would
  delete or shrink partitions sees `true` and is expected to surface it; the tool does not refuse.

#### 4.7.4 `aios.fs_layout.remove` (mutation)

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `layout_id` | string | **yes** | |
| `store_path` | string | **yes** | |

- **Behaviour:** load → `remove_layout(layout_id)` → `save_to_path`.
- **Success:** `{ok: true, tool, id, removed: true, active_layout_id}`.
- **Errors:** missing `layout_id`; active layout
  (`"cannot remove active layout '<id>'; switch active layout first"`); built-in preset
  (`"cannot remove built-in canonical layout '<id>'"`); unknown id; save failure.
- **Persistence:** exactly one atomic store write; refusals write nothing.

#### 4.7.5 `aios.fs_layout.import_fstab` (mutation)

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `layout_id` | string | **yes** | — | Id for the new profile. |
| `name` | string | **yes** | — | Human-readable profile name. |
| `fstab` | string | **yes** | — | Path (regular file only) or inline fstab content. |
| `base_layout_id` | string | no | store active layout | Partitions/directories are inherited from here. |
| `store_path` | string | **yes** | — | |

- **Behaviour:** read `fstab` (bounded/type-checked when it is a path) →
  `import_fstab_as_layout(layout_id, name, content, base_layout_id)` → `save_to_path`.
- **Success:** `{ok: true, tool, id, name, mounts: <n>, layout: <spec>}`.
- **Errors:** missing arguments; fstab with no usable rows
  (`"failed to import fstab as layout: ..."`); > 128 mount rows; unknown `base_layout_id`; duplicate
  `layout_id`; unreadable/non-regular/oversize fstab file; save failure.
- **Persistence:** exactly one atomic store write; a failed import writes nothing.

---

## 5. Result Envelopes

### 5.1 JSON-RPC wrapper (`tools/call`)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{"type": "text", "text": "<body as a JSON string>"}],
    "structuredContent": {"result": { }},
    "isError": false
  }
}
```

`isError` is `true` for both body failures and PEP refusals, so a client can branch on it alone.

### 5.2 Success body

```json
{"ok": true, "tool": "aios.fs_layout.register", "id": "lab-vm-v1",
 "registered": true, "active_layout_id": "aios-uefi-standard-v1",
 "audit_id": 4127, "classifier_policy_revision": 3}
```

### 5.3 PEP refusal envelope

```json
{"ok": false, "tool": "aios.fs_layout.remove", "audit_id": 4128,
 "reason": "<policy reason>", "gate": "<gate>", "policy_revision": 3}
```

A refusal is written to the audit ring before returning and never falls through to the body.

### 5.4 Body failure envelope

```json
{"ok": false, "tool": "aios.fs_layout.remove",
 "error": "cannot remove active layout 'lab-vm-v1'; switch active layout first",
 "audit_id": 4129}
```

---

## 6. Error Catalogue

| Condition | Tool(s) | Message fragment | Envelope |
|---|---|---|---|
| Missing required argument | all mutations | `"... requires ..."` | §5.4 |
| `store_path` absent on a mutation | mutations | `store_path is required for mutating fs_layout tools` | §5.4 |
| `store_path` > 1024 chars / control chars | all store-backed | `store_path exceeds 1024 characters or contains control characters` | §5.4 |
| Corrupt store file | all store-backed | deserialize error surfaced verbatim | §5.4 |
| Unknown layout id | `get`, `probe`, `set_active` | `layout with id '<id>' not found in store` | §5.4 |
| Duplicate id | `register`, `import_fstab` | `... already registered` | §5.4 |
| `FL1..FL5` violation | `register`, `import_fstab` | validated error text | §5.4 |
| Spec path is a directory / FIFO / device | `validate`, `fstab`, `register`, `import_fstab` | `... is a directory` / `not a regular file` | §5.4 |
| Document above 10 MiB | same | size-limit error from `LayoutDocReadError::TooLarge` | §5.4 |
| Inline payload above 1 MiB | `validate`, `fstab`, `register` | `... exceeds 1 MiB limit` | §5.4 |
| Not-UTF-8 document | same | UTF-8 error from `LayoutDocReadError::NotUtf8` | §5.4 |
| Remove active / built-in layout | `remove` | see §4.7.4 | §5.4 |
| Missing auth | mutations | policy reason | §5.3 |
| Request line > 1 MiB | transport | `-32700 request line exceeds 1048576 bytes` | JSON-RPC error |

---

## 7. Persistence Effects

| Tool | Store read | Store write |
|---|---|---|
| `get` / `probe` / `diff` / `list` | yes (`store_path` or presets) | **no** |
| `validate` | no (unless `store_path` bounds are checked) | **no** |
| `fstab` | no | **no** |
| `register` | yes | one atomic `save_to_path` |
| `set_active` | yes | one atomic `save_to_path` |
| `remove` | yes | one atomic `save_to_path` |
| `import_fstab` | yes | one atomic `save_to_path` |

`save_to_path` remains the only writer: it stages into `.<name>.tmp.<pid>.<nanos>.<n>` with
`O_CREAT | O_EXCL`, fsyncs, then renames over the store, preserving the staged file when only the
rename fails (`T-01528` H-1..H-3, H-6). A refusal (PEP or semantic) performs no write.

---

## 8. Security & Invariant Guarantees

1. **Authorization:** all four mutations require a passing PEP verdict in addition to the payload
   checks; reads remain ungated because they are non-mutating.
2. **Path hygiene:** `store_path` ≤ 1024 characters, no control characters; spec/fstab paths must be
   regular files; symlinks to regular files stay readable, and no write ever follows a pre-existing
   link.
3. **Bounded reads:** file documents ≤ 10 MiB enforced during the read; inline payloads ≤ 1 MiB
   enforced explicitly; the transport line cap stays at 1 MiB.
4. **Invariant enforcement before mutation:** `register`/`import_fstab` cannot insert a layout that
   violates `FL1..FL5`; `remove` cannot orphan the active pointer; built-in presets are immutable.
5. **Audit immutability:** one hash-chained SQLite WAL row per call, including refusals and body
   failures, with `outcome` and the failure detail.
6. **No privilege escalation:** no process spawning, no disk formatting, no writing outside the store
   file named by `store_path`.

---

## 9. Audit Effects

| Field | Value |
|---|---|
| `tool` | e.g. `aios.fs_layout.set_active` |
| `command` | Short human description, e.g. `"Set active Filesystem Layout"` |
| `target` | The layout id for per-layout operations; `None` for `validate`/`fstab`/`list` |
| `grant_id` | Echoed when supplied |
| `actor_id` / `actor` | `agent:mcp@aiosh-mcp` / `agent` (unchanged) |
| `outcome` | `ok` or `error`; refusals are recorded by the gate path |
| `detail` | The error string on failure, `null` on success |

---

## 10. Acceptance & Test Plan (for `T-01534`/`T-01535`/`T-01536`)

1. **Manifest:** `tools/list` exposes all ten `aios.fs_layout.*` tools; every mutation advertises
   `store_path` as required and no read tool does.
2. **Happy path per tool:** register → list shows it → get by id returns it → set_active switches the
   pointer (with `destructive_transition`) → probe/diff default to the new active layout →
   import_fstab adds a second profile → remove deletes it; the store file is re-read from disk after
   each step to prove persistence.
3. **Failure path per tool:** every row of §6 exercised, asserting `ok: false` and the message
   fragment; refusals (active remove, built-in remove, duplicate register, invalid FL) must leave the
   store byte-identical on disk.
4. **Gating:** each mutation without `grant_id` returns the §5.3 refusal envelope; reads succeed with
   no grant.
5. **Hardening:** a directory named by `spec`/`fstab` is refused by type on the MCP path (the F-1
   regression), and an oversize document is refused within the bounded budget.
6. **Parity:** CLI `register`/`set-active` observed by MCP and vice versa (extends the existing FL6
   suite); `fstab` text remains byte-identical across surfaces.
7. **Cross-surface mutation equivalence:** a layout registered through the CLI is identical
   (field-for-field) to one registered through MCP with the same input.
