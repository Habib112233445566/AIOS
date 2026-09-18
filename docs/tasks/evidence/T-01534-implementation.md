# T-01534: Filesystem Layout - MCP/API Surface: Implementation

## Metadata
- **Task ID:** `T-01534`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout agent surface (`code/aiosh-rust/aiosh-mcp::main`, `aios.fs_layout.*`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (4/10) — MCP/API Surface Implementation
- **Dependencies:** `T-01533` (MCP/API Surface Scaffold)
- **Next Task:** `T-01535` (Filesystem Layout / MCP/API surface: Unit Test)
- **Code changed:** `code/aiosh-rust/aiosh-mcp/src/main.rs`,
  `code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py`
- **Spec:** `docs/tasks/evidence/T-01532-spec.md`

---

## 1. Scope & Objective

`T-01533` declared the four mutation tool interfaces; this task implements them and closes the three
read-side divergences `T-01531` recorded. The result is a ten-tool `aios.fs_layout.*` surface that
mirrors the operator CLI one-for-one.

Delivered:

1. `aios.fs_layout.register`, `set_active`, `remove`, `import_fstab` — thin adapters over the
   **existing** store methods, `require_grant = true`, `store_path` **required** (spec §3.1, D-9).
2. `aios.fs_layout.get` widened with `layout_id` (+ optional `store_path`); the `profile` preset
   path is byte-identical to before (spec D-4).
3. `aios.fs_layout.probe` defaults to the **store's active layout** instead of the literal
   `aios-uefi-standard-v1` (spec D-5).
4. `aios.fs_layout.validate` / `.fstab` (and the two file-consuming mutations) route every `spec`
   / `fstab` path through `aiosh_core::fs_layout_service::read_bounded_text_file`, with an explicit
   1 MiB inline-payload guard — closing finding **F-1** (spec D-6).
5. The FL6 cross-surface suite extended from 6 tools / 5 groups to **10 tools / 9 groups**,
   exercising every tool through the real `aiosh-mcp` binary over stdio, including a
   CLI↔MCP parity check and a granted-versus-ungranted mutation check.

No new core API was invented: every mutation calls an existing
`FilesystemLayoutStore` / `FilesystemLayoutService` method, so the two surfaces cannot drift
semantically.

## 2. Implementation Detail

### 2.1 Shared helpers (new, private to `main.rs`)

| Helper | Purpose |
|---|---|
| `MAX_INLINE_LAYOUT_BYTES = 1 MiB` | Inline payload ceiling (matches the session tools and the transport line cap) |
| `read_layout_document_input(value, label, inline_label)` | Path → `read_bounded_text_file(path, MAX_LAYOUT_DOC_BYTES, label)`; inline → explicit 1 MiB guard. A directory/FIFO/device is refused **by type**; the 10 MiB ceiling is enforced on the byte stream |
| `ensure_inline_payload_bounded(value, label)` | Bounds an inline JSON `layout` object that never touches the filesystem |
| `require_fs_layout_store_path(args)` | Enforces the required, bounded (≤ 1024 chars, no control chars) mutation `store_path` |

### 2.2 Mutation adapters (spec §4.7)

| Tool | Adapter call | Persistence | Reported fields |
|---|---|---|---|
| `register` | `store_mut().register_layout(spec)` (validates `FL1..FL5`, refuses duplicate ids) | one `save_to_path` | `id`, `registered`, `active_layout_id` |
| `set_active` | `store_mut().set_active_layout(id)` (refuses unknown ids) | one `save_to_path` | `previous_active`, `active`, `destructive_transition` |
| `remove` | `store_mut().remove_layout(id)` (refuses active + built-in presets) | one `save_to_path` | `id`, `removed`, `active_layout_id` |
| `import_fstab` | `import_fstab_as_layout(id, name, content, base)` (≤ 128 mounts, no-rows rejection) | one `save_to_path` | `id`, `name`, `mounts`, `layout` |

`destructive_transition` is the `diff_layouts(previous → new).destructive` verdict, or `null` when
the transition cannot be diffed (e.g. an empty previous active id). It is **informational only** —
the tool reports, it does not refuse (spec D-7).

Every arm routes through `dispatch::recorded_call(..., require_grant = true, ...)` with the layout id
as the audit `target` where one is known, so a refusal is written to the audit ring before returning
and never falls through to the body (ADR-0035 §F-2).

### 2.3 F-1: the single-threaded stall is closed

Before (`aiosh.fs_layout.validate` / `.fstab`):

```rust
let content = std::fs::read_to_string(p).map_err(|e| format!("failed to read layout file: {}", e))?;
```

`read_to_string` is type-blind and unbounded: a FIFO named by `spec` blocks the whole request loop
(the MCP server is single-threaded), and `/dev/zero` streams until memory is exhausted. After
T-01534, all four file-consuming tool arms go through `read_bounded_text_file`, which stats first,
refuses anything that is not a regular file, and enforces the cap during the read. No
`std::fs::read_to_string` call remains anywhere in the `fs_layout` code path (the three remaining
call sites belong to `aios.evidence.*`, `aios.fs.read`, and `aios.process.list`):

```
$ grep -n "std::fs::read_to_string" code/aiosh-rust/aiosh-mcp/src/main.rs
3740:  (aios.evidence manifest)      3882:  (aios.fs.read)      3955:  (/proc/<pid>/comm)
```

### 2.4 Test contract updated (spec D-10)

- `test_mcp_fs_layout_tools` (in-tree) now covers: mutation discovery + `store_path` required;
  ungranted refusal with no store written; missing `store_path` with a grant; granted
  register → duplicate refusal (store byte-identical) → disk reload → `get` by id (and unknown id) →
  `set_active` (`previous_active`, `active`, `destructive_transition`) → `probe` default following
  the new active layout → `import_fstab` → active/built-in removal refusals → `remove`; a directory
  named by `spec` refused by type; and an oversize inline spec rejected by the 1 MiB guard.
- `code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py` (FL6) widened to 10 tools and four new groups
  (gating, round-trip, CLI↔MCP mutation parity, read hardening).

## 3. Verified Evidence

Build:

```
$ cargo build -p aiosh-mcp
   Compiling aiosh-mcp v0.1.0 (...\code\aiosh-rust\aiosh-mcp)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 26.68s
```

Targeted in-tree test:

```
$ cargo test -p aiosh-mcp
running 12 tests
test tests::test_mcp_fs_layout_tools ... ok
... (11 more) ...
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.25s
```

No regression on the neighbouring crates:

```
$ cargo test -p aiosh-core   →  aiosh-core passed=582 failed=0
$ cargo test -p aiosh-cli    →  aiosh-cli  passed=24  failed=0
```

Aggregate Filesystem Layout criteria (unchanged FL1..FL7, now with FL6 covering mutations):

```
$ python tools/test_fs_layout_suites.py
[+] FL1 filesystem layout data model integrity & invariants (FL1..FL5)
[+] FL2 filesystem layout core service (store, probe, diff, fstab, persistence)
[+] FL3 filesystem layout CLI surface smoke & boundaries (cmd_fs_layout)
[+] FL4 filesystem layout CLI audit emission & escape-injection security proof
[+] FL5 filesystem layout CLI in-tree unit test
[+] FL5 filesystem layout MCP in-tree unit test
[+] FL6 filesystem layout cross-surface CLI <-> MCP integration parity
[+] FL7 filesystem layout CLI hardening (non-regular paths, bounded reads, atomic persistence)

PASS: fs_layout_suites criteria (FL1..FL7)
```

Every one of the ten tools exercised through the real `aiosh-mcp` binary over stdio:

```
$ python code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py
=== RUNNING FILESYSTEM LAYOUT CROSS-SURFACE INTEGRATION SMOKE TESTS ===
PASS: tools/list exposes all 10 aios.fs_layout.* tools; mutations require store_path
PASS: aiosh layout is advertised in root help and reachable via the fs-layout alias
PASS: CLI <-> MCP parity on fstab, validate, diff and probe
PASS: shared canonical store - CLI register/set-active visible to MCP and vice versa
PASS: store_path semantics parity (fallback, corrupt, oversized, control chars)
PASS: mutations require BOTH a PEP grant and an explicit store_path; reads stay ungated
PASS: granted register/get/list/set_active/probe/import_fstab/remove round-trip persisted
PASS: CLI <-> MCP mutation parity (identical layouts, mutual pointer visibility)
SKIP: FIFO stall case needs POSIX mkfifo (Windows host) - covered on POSIX CI
PASS: spec/fstab reads are type-checked, bounded and non-blocking (F-1 closed)

ALL FILESYSTEM LAYOUT CROSS-SURFACE INTEGRATION TESTS PASSED!
```

What the new groups assert, in detail:

| Group | Assertion |
|---|---|
| Gating (granted vs ungranted) | each of the four mutations without `grant_id` returns `ok:false` with `gate:"pep"` and an `audit_id`, and **no store file is created**; with a grant but no `store_path` each returns `store_path is required...`; a grant scoped to `aios.session.*` does not authorize `fs_layout`; reads still need no grant |
| Round-trip | granted `register` (from a regular-file spec) → `get` by id returns the exact stored body → `list` count 3 → `set_active` reports `previous_active`/`active`/`destructive_transition:false` → `probe` **without** `layout_id` evaluates the new active layout → `import_fstab` → `remove`; the store file is re-read from disk after each step; duplicate register, active removal, built-in removal, unknown id and a no-rows fstab all leave the store **byte-identical** |
| CLI↔MCP parity | the same spec registered through the CLI and through MCP is field-for-field identical and equals the source JSON; CLI `set-active` is observed by MCP (including the new `probe` default) and MCP `set-active` is observed by the CLI in both directions; fstab text stays byte-identical |
| Read hardening | a directory named by `spec`/`fstab` is refused by type on `validate`, `fstab`, `register` and `import_fstab` with `is a directory`, in < 10 s, writing nothing; a > 1 MiB request line is refused by the transport with `-32700` |

### 3.1 Acceptance mapping

| Acceptance criterion (`T-01534`) | Result |
|---|---|
| Targeted test passes | `test_mcp_fs_layout_tools` (12-test `aiosh-mcp` suite) and the widened FL6 suite both green |
| No regression in existing smoke suites for touched modules | FL1..FL7 aggregate green; `aiosh-core` 582/0 and `aiosh-cli` 24/0 unchanged from the `T-01530` baseline |

## 4. Honest Limitations

1. **The FIFO stall case is not executed on this Windows host** — `os.mkfifo` is POSIX-only, so the
   suite prints an explicit SKIP rather than a false PASS. The type refusal is nonetheless proven
   cross-platform on a *directory* (same `read_bounded_text_file` code path: `metadata()` →
   `is_file()` → refuse), and the FIFO case runs on POSIX CI.
2. **The inline 1 MiB guard is unreachable over stdio**: the transport line cap is also 1 MiB, so a
   request carrying an oversize inline payload is answered with `-32700` before the tool body runs.
   The guard therefore matters for in-process callers only and is pinned by the in-tree unit test;
   the smoke test asserts the transport cap instead.
3. **The in-memory default store is still the behaviour for reads without `store_path`** — a
   deliberate consequence of D-9, unchanged here. The canonical default path remains owned by the
   configuration sub-epic (`T-01541..T-01550`).
4. **Documentation is stale by design**: `docs/filesystem_layout.md` §5 still describes six
   read-only MCP tools, and `code/aiosh-mcp/README.md` has no `fs_layout` section. Both are owned by
   the documentation task `T-01539` (spec D-10); nothing in this change claims otherwise.
5. **A mutation without a grant is refused before the `store_path` check runs**, so the
   `store_path is required` message is only observable with a valid grant. This is the intended gate
   ordering (classifier → PEP → body), and the test asserts both branches separately.
6. `destructive_transition` is reported, never enforced (D-7); an agent may activate a layout that
   would delete or shrink partitions, and is only *informed*.

## 5. Citations

1. `docs/tasks/evidence/T-01532-spec.md` — frozen contract (§3.1, §4.1–4.7, §6, §8, §10).
2. `docs/tasks/evidence/T-01531-mcp-api-surface-research.md` — findings F-1..F-4, decisions D-1..D-10.
3. `code/aiosh-rust/aiosh-core/src/fs_layout_service.rs` — `read_bounded_text_file`, `MAX_LAYOUT_DOC_BYTES`,
   `register_layout`, `set_active_layout`, `remove_layout`, `import_fstab_as_layout`, `save_to_path`.
4. `code/aiosh-rust/aiosh-core/src/dispatch.rs:213` — `recorded_call` gate + single audit row.
5. `code/aiosh-rust/aiosh-cli/src/main.rs` — `cmd_fs_layout_*` mutation semantics mirrored 1:1.
6. `code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py`, `tools/test_fs_layout_suites.py` — FL6 / FL1..FL7.
