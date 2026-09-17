# T-01533: Filesystem Layout - MCP/API Surface: Scaffold

## Metadata
- **Task ID:** `T-01533`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout agent surface (`code/aiosh-rust/aiosh-mcp::main`, `aios.fs_layout.*`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (3/10) — MCP/API Surface Scaffold
- **Dependencies:** `T-01532` (MCP/API Surface Specification)
- **Next Task:** `T-01534` (Filesystem Layout / MCP/API surface: Implementation)
- **Code changed:** `code/aiosh-rust/aiosh-mcp/src/main.rs` (interface declarations only)

---

## 1. Scope & Objective

`T-01532` froze the contract for ten `aios.fs_layout.*` tools, four of them new mutations. This
scaffold declares the **new interface** only — the four mutation tools — and wires it into the
server's discovery and dispatch paths so that the surface is reachable and fails loudly. Behavioural
changes to the existing read tools (`get` widening, `probe` default, `validate`/`fstab` hardening)
and the bodies of the four mutations are deliberately **out of scope here** and are delivered by
`T-01534`.

The scaffold is intentionally thin: it proves the interface exists, is advertised, is gated, and
compiles — nothing more.

## 2. What Changed

### 2.1 Manifest (`tool_manifest`, `code/aiosh-rust/aiosh-mcp/src/main.rs`)

Four entries appended after `aios.fs_layout.diff`, each with `additionalProperties: false` and
`grant_id` advertised as an optional PEP parameter:

| Tool | `required` | Notes |
|---|---|---|
| `aios.fs_layout.register` | `["store_path"]` | `layout` (object) xor `spec` (string) declared as optional alternatives; the body owns the "neither supplied" error |
| `aios.fs_layout.set_active` | `["layout_id", "store_path"]` | |
| `aios.fs_layout.remove` | `["layout_id", "store_path"]` | |
| `aios.fs_layout.import_fstab` | `["layout_id", "name", "fstab", "store_path"]` | `base_layout_id` optional (defaults to the store active layout) |

`store_path` is advertised as **required** on every mutation, exactly as `T-01532` §3.1 requires:
no canonical default store exists yet (that default belongs to the configuration sub-epic,
`T-01541..T-01550`), so a mutation without one would appear to succeed against a throw-away
in-memory store.

### 2.2 Dispatch (`call_tool`)

Four match arms were added after the `aios.fs_layout.diff` arm. Each routes through the house gate
`dispatch::recorded_call` with **`require_grant = true`** (matching `aios.session.action` /
`aios.session.create`) and a body that fails loudly:

```rust
"aios.fs_layout.register" => {
    let f = move || -> Result<Value, String> {
        Err("aios.fs_layout.register is not implemented yet (T-01533 scaffold)".to_string())
    };
    dispatch::recorded_call(
        &mut self.ring, &self.pep,
        "aios.fs_layout.register", "Register Filesystem Layout profile", arguments,
        None, grant_id, true, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
    )
}
```

Because the stub is wrapped in `recorded_call`, the scaffold already proves the two invariants
`T-01532` §8 depends on:

1. an ungranted call is refused by the PEP gate **before** the body runs, with one audit row written
   by the refusal path;
2. a granted call reaches the body and still writes exactly one audit row (the body failure is the
   call's `error`, and the row's `outcome` is `error`).

### 2.3 Test stub

`test_mcp_fs_layout_tools` (in-tree, `#[cfg(test)] mod tests`) gained a step 10 that:

- asserts all four mutation tools are present in `tool_manifest()`;
- asserts each advertises `store_path` in `inputSchema.required`;
- asserts an ungranted `aios.fs_layout.remove` returns `ok: false` with `gate == "pep"`;
- mints a real grant through `server.pep.create(...)` and asserts the granted call reaches the stub
  and returns the loud `not implemented` error.

## 3. Verified Evidence

Build (compiles, zero errors):

```
$ cargo build -p aiosh-mcp
   Compiling aiosh-mcp v0.1.0 (...\code\aiosh-rust\aiosh-mcp)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.12s
```

Targeted in-tree interface test:

```
$ cargo test -p aiosh-mcp --bin aiosh-mcp test_mcp_fs_layout_tools
running 1 test
test tests::test_mcp_fs_layout_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.16s
```

Acceptance mapping:

| Acceptance criterion | Result |
|---|---|
| Project builds/imports with zero errors | `cargo build -p aiosh-mcp` finished with zero errors |
| New interfaces exist and are referenced by at least one call site or test stub | four manifest entries + four `call_tool` arms + step 10 of `test_mcp_fs_layout_tools` |

## 4. Honest Limitations Carried Into `T-01534`

- The four mutation bodies are stubs; `register`/`set_active`/`remove`/`import_fstab` do nothing.
- `store_path` is advertised as required but not yet enforced in a body (`T-01534` adds the
  explicit `"store_path is required for mutating fs_layout tools"` error).
- The existing read-tool divergences recorded in `T-01531` §2.3 remain open:
  `validate`/`fstab` still read a `spec` path with `std::fs::read_to_string`, `get` cannot reach
  store layouts by id, and `probe` still defaults to the literal `aios-uefi-standard-v1`.
- The FL6 cross-surface suite still asserts "all 6" tools and passes unchanged, because the new
  tools are additive; the suite must be widened in `T-01534` (spec `D-10`).

## 5. Citations

1. `docs/tasks/evidence/T-01532-spec.md` — tool schemas, §3.1 (`store_path` required), §4.7, §6.
2. `code/aiosh-rust/aiosh-mcp/src/main.rs` — `tool_manifest`, `call_tool`, `test_mcp_fs_layout_tools`.
3. `code/aiosh-rust/aiosh-core/src/dispatch.rs:213` — `recorded_call` refusal short-circuit.
4. Prior evidence: `T-01531-mcp-api-surface-research.md`, `T-01532-mcp-api-surface-specification.md`.
