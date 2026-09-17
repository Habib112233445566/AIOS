# T-01526: Filesystem Layout - CLI Surface: Integration

## Metadata
- **Task ID:** `T-01526`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface integration (`code/aiosh-rust/aiosh-cli::cmd_fs_layout` ↔ `code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (6/10) — CLI Surface Integration
- **Dependencies:** `T-01525` (CLI Surface Unit Test)
- **Next Task:** `T-01527` (Filesystem Layout / CLI surface: Security Review)

---

## 1. Production Surfaces Integrated

### 1.1 Root Command Dispatcher (`code/aiosh-rust/aiosh-cli/src/main.rs`)
- Top-level routing (verified present, covers both the canonical command and the compatibility alias):

  ```rust
  Some("layout") | Some("fs-layout") => cmd_fs_layout(&args[1..]),
  ```

- The root usage banner advertises the complete subcommand taxonomy:
  ```text
  aiosh layout <list|show|validate|check|probe|diff|fstab|register|set-active|remove|import-fstab>  Filesystem Layout & Target Partitioning Manager
  ```
- `aiosh layout --help` documents all 10 subcommands plus the `--store` / `--json` / `--spec`
  options.

### 1.2 Agent MCP Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)

**Parity gap found and fixed.** The six `aios.fs_layout.*` tools previously constructed
`FilesystemLayoutService::new()` unconditionally, so an autonomous agent could only ever see the two
seeded built-in presets. A layout registered by the operator through
`aiosh layout register --store <path>` was invisible to the agent surface, and vice versa — the
shared canonical JSON store was write-only from the CLI's perspective.

Added `resolve_fs_layout_service(&Option<String>)` and wired it into the three store-backed read
tools, mirroring the existing `resolve_service_store` pattern:

- `aios.fs_layout.list` — enumerates the layouts in the supplied store (active pointer included).
- `aios.fs_layout.probe` — probes any layout in the supplied store, including operator-registered ones.
- `aios.fs_layout.diff` — diffs any two layouts in the supplied store.

The `store_path` parameter was added to each tool's `inputSchema.properties` (all three schemas use
`"additionalProperties": false`, so an undocumented parameter would have been rejected outright).

Resolution semantics deliberately mirror the CLI's `load_fs_layout_service` so both surfaces observe
identical state: an existing file is loaded and validated via
`FilesystemLayoutService::load_from_path`; a path that does not exist yet yields the seeded default
store rather than an error; paths exceeding 1024 characters or containing control characters are
rejected.

### 1.3 Shared Substrate
The shared substrate is the canonical layout store JSON (`fs_layouts.json`: `active_layout_id` +
`layouts` map), not SQLite. As of this task both the operator CLI and the agent MCP surface read and
write that same file, so state changes propagate in both directions.

---

## 2. Integration Test & Verification

New cross-surface integration smoke suite: **`code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py`**
(5 groups), following the `test_session_mcp_smoke.py` convention of driving the `aiosh-mcp` binary
over stdio JSON-RPC 2.0 and the `aiosh` binary as a subprocess.

```
$ python code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py
=== RUNNING FILESYSTEM LAYOUT CROSS-SURFACE INTEGRATION SMOKE TESTS ===
PASS: tools/list exposes all 6 aios.fs_layout.* tools incl. store_path
PASS: aiosh layout is advertised in root help and reachable via the fs-layout alias
PASS: CLI <-> MCP parity on fstab, validate, diff and probe
PASS: shared canonical store - CLI register/set-active visible to MCP and vice versa
PASS: store_path semantics parity (fallback, corrupt, oversized, control chars)

ALL FILESYSTEM LAYOUT CROSS-SURFACE INTEGRATION TESTS PASSED!
```

What the suite proves:

| Claim | Assertion |
|---|---|
| Feature reachable through production surfaces | root help advertises `aiosh layout <...>`; both `layout` and `fs-layout` routes return identical canonical data |
| Tool discoverability | `tools/list` returns all 6 `aios.fs_layout.*` tools and advertises `store_path` on the store-backed three |
| Cross-substrate parity (presets) | `fstab` text is **byte-identical**; `validate` verdict, `diff` destructive flag + delta counts, and `probe` viability/budget arithmetic all agree |
| Shared canonical store (CLI → MCP) | a layout registered by the CLI is enumerated by MCP with `store_path`, and its active pointer is observed |
| Isolation | without `store_path`, MCP still sees only the 2 seeded presets (no implicit leakage of operator state) |
| Bidirectional state sharing | after the CLI switches the active pointer, MCP reports the same `active_layout_id`; the CLI then observes the identical layout set |
| Failure modes | missing store falls back to seeded presets on both surfaces; a corrupt store fails loudly on both (MCP `ok: false`, CLI exit 1) |

### 2.1 Regression suites

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.81s

$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.26s

$ python code/aiosh-cli/tests/test_fs_layout_cli_smoke.py
ALL FILESYSTEM LAYOUT CLI SMOKE TESTS PASSED!

$ python code/aiosh-mcp/tests/test_session_mcp_smoke.py
ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!
```

---

## 3. Acceptance Verification

- [x] Feature reachable through its production surface — `aiosh layout` (and the `fs-layout`
      alias) for operators, `aios.fs_layout.*` for agents, both reading the same canonical store.
- [x] Integration smoke passes end-to-end with zero regressions.

---

## 4. Known Limitations

- `aios.fs_layout.get` / `validate` / `fstab` still operate on the seeded presets or an inline
  `spec`/`profile` argument and do not accept `store_path`; store-backed reads go through
  `list`/`probe`/`diff`. Extending `get` to resolve an arbitrary stored layout ID is a natural
  follow-up but was not required to close the parity gap.
- The MCP `diff`/`probe` tools remain ungated (`require_grant = false`) as read-only surfaces; PEP
  gating and audit-row emission for the CLI surface are audited in `T-01527` (security review).
- The shared store has no file locking; concurrent CLI and MCP writes to the same store path are
  last-writer-wins. Single-writer usage is assumed, as elsewhere in the system.
