# T-01535: Filesystem Layout - MCP/API Surface: Unit Test

## Metadata
- **Task ID:** `T-01535`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout agent surface (`code/aiosh-rust/aiosh-mcp::main`, `aios.fs_layout.*`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (5/10) — MCP/API Surface Unit Test
- **Dependencies:** `T-01534` (MCP/API Surface Implementation)
- **Next Task:** `T-01536` (Filesystem Layout / MCP/API surface: Integration)
- **Code changed:** `code/aiosh-rust/aiosh-mcp/src/main.rs`,
  `code/aiosh-rust/aiosh-core/src/dispatch.rs`,
  `code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py` (new)

---

## 1. Scope & Objective

A read-only audit of the `T-01534` surface probed three defects against the real `aiosh-mcp`
binary. This task closes all three and adds the focused tests that should have caught them.

| # | Defect (audit finding) | Fix | Test that catches it |
|---|---|---|---|
| 1 | `aios.fs_layout.get` advertised only `profile` + `grant_id`, so the `layout_id` support added by `T-01534` was undiscoverable (and any client honouring `additionalProperties: false` would reject it); `validate` likewise omitted `store_path` | Both schemas now advertise exactly what each arm reads | `test_mcp_fs_layout_manifest_matches_accepted_arguments` (in-tree) + C1 (contract file) |
| 2 | `register` wrote two different audit rows for the same operation — a `spec` path logged `target = None`, an inline `layout` logged the layout id — contradicting spec §9 | The layout id is recorded for **both** accepted forms | `test_mcp_fs_layout_register_audit_target_is_layout_id` + C2 |
| 3 | `destructive_transition: true` was only proven by the audit's throwaway probe; both the in-tree test and FL6 asserted only `false` | A shrink is pinned as `true`, a grow as `false` | `test_mcp_fs_layout_set_active_reports_destructive_transition` + C3 |

## 2. Fix Detail

### 2.1 Manifest schemas match the arms (defect 1)

`aios.fs_layout.get` now advertises `layout_id`, `profile`, `store_path`, `grant_id`; its description
no longer claims to return only a built-in preset. `aios.fs_layout.validate` now advertises
`store_path` as well, and **the arm enforces its bounds** (`check_fs_layout_store_path_bounds`), so
the advertised parameter is genuinely accepted — spec §4.2 keeps `store_path` on `validate` for
signature/bounds parity without reading the store. `fstab`, `list`, `probe`, `diff` and the four
mutations already matched their arms and were left alone.

The predicate `< 1024 chars, no control characters` now has a single owner in the MCP server:
`require_fs_layout_store_path` (mutations) and `validate` both call
`check_fs_layout_store_path_bounds`, instead of the mutation path re-implementing it.

### 2.2 One audit target per operation (defect 2)

The layout id is only knowable *after* parsing the spec, and that parse must stay behind the gate:
reading the file pre-gate would let an unauthorized caller trigger the read, and would let a FIFO
named by `spec` stall the single-threaded request loop before policy was consulted. So the fix is an
explicit dispatch variant rather than a pre-gate read:

```rust
// aiosh-core/src/dispatch.rs
pub fn recorded_call<F>(...)            // unchanged signature, delegates with a no-op resolver
pub fn recorded_call_with_resolved_target<F>(..., resolve_target: &dyn Fn(&Value) -> Option<String>, f: F)
```

The body's successful result may now resolve the row target; `None` falls back to the pre-gate
target. The pre-gate target is still what the classifier and PEP see and what a **refusal** row
records — only the post-gate outcome row can be enriched, so the row never claims authorization it
did not receive. `aios.fs_layout.register` resolves it from the parsed spec id:

```rust
&|body: &Value| body.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
```

Verified against the audit ring: `spec` path → `target = 'contract-spec-v1'`, inline →
`target = 'contract-inline-v1'`.

### 2.3 New standalone contract test (the task's required test file)

`code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py` (≈330 lines) drives the real binary over
stdio and asserts four criteria: **C1** advertised `inputSchema.properties` equals the declared
accepted-argument set for all ten tools (plus `additionalProperties: false` preserved and the
mutations' `store_path` requirement), **C2** `register` audit target is the layout id for both input
forms, read back through the CLI's audit tail, **C3** `destructive_transition` is `true` on a shrink,
`false` on a grow, and agrees with the shared `diff` verdict, **C4** negative cases (ungranted
mutation, missing `store_path`, oversize `store_path` on both a mutation and `validate`, unknown tool
name). The same accepted-argument table lives in the in-tree test as
`FS_LAYOUT_TOOL_ARGUMENTS`, so the schema cannot drift again.

## 3. Verified Evidence

```
$ cargo build -p aiosh-mcp -p aiosh-cli
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 51.65s

$ cargo test -p aiosh-mcp
running 15 tests
test tests::test_mcp_fs_layout_manifest_matches_accepted_arguments ... ok
test tests::test_mcp_fs_layout_register_audit_target_is_layout_id ... ok
test tests::test_mcp_fs_layout_set_active_reports_destructive_transition ... ok
test tests::test_mcp_fs_layout_tools ... ok
... (11 more) ...
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.22s

$ cargo test -p aiosh-core   →  aiosh-core passed=582 failed=0

$ python code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py
=== RUNNING FILESYSTEM LAYOUT MCP CONTRACT UNIT TESTS ===
PASS: C1 advertised inputSchema == accepted arguments for all 10 tools
PASS: C2 register records the layout id as the audit target for both input forms
PASS: C3 destructive_transition true on shrink, false on grow, matches diff
PASS: C4 negative cases (ungranted, missing/oversize store_path, unknown tool)

ALL FILESYSTEM LAYOUT MCP CONTRACT CRITERIA PASSED!
```

The earlier cross-surface suite is unchanged and still green (9/9 groups). The pre-existing
`FL6 ... PASS` output is recorded in `T-01534-mcp-api-surface-implementation.md`; the integration
task `T-01536` extends it and re-runs it.

### 3.1 The new test fails when the feature is broken (task acceptance)

Each criterion was negative-controlled by injecting the pre-fix behaviour and confirming a loud
failure:

```
OK  C1 schema contract: fails when broken -> aios.fs_layout.get: manifest advertises
      ['grant_id', 'layout_id', 'profile', 'store_path'] but ...           (pre-fix expectation)
OK  C2 audit target: fails when broken -> spec path: audit target is None, expected the layout id
      'contract-spec-v1'
OK  C3 destructive verdict: fails when broken -> shrink must report destructive_transition=true
```

### 3.2 Acceptance mapping

| Acceptance criterion (`T-01535`) | Result |
|---|---|
| New test file runs standalone and passes | `test_fs_layout_mcp_contract.py` runs standalone, C1..C4 pass |
| Negative cases are asserted, not just happy path | C4 (ungranted, missing/oversize `store_path`, unknown tool) plus the three negative controls above |
| (ledger) cover valid input, invalid input, boundary values, primary failure mode | C1 valid schema set; C2 both input forms; C3 true/false verdicts; C4 boundary (1025-char path) and failure modes |

## 4. Honest Limitations

1. **Defect 2's fix touches shared core dispatch.** `recorded_call_with_resolved_target` is a new
   public function in `aiosh-core/src/dispatch.rs`; `recorded_call` keeps its signature and behaviour
   (it delegates with a no-op resolver), and all ~120 existing call sites are untouched. The new
   function is used by exactly one tool today (`register`).
2. **The resolver reads the tool's own response key (`id`)** rather than a dedicated field, so a
   future `register`-like tool must be explicit about which key carries the target. A dedicated
   reserved key was rejected as hidden magic in an audit path; a second gate variant was rejected as
   duplication.
3. **C1 asserts the declared table, not the source.** The in-tree table and the contract file's
   table are two copies of the same list; a parameter added to an arm but to neither table would pass.
   Closing that fully needs schema derivation from the arms rather than a test-side declaration.
4. **The audit-target check reads rows via the CLI's audit tail** (the DB is shared), so it depends on
   the default audit DB being writable; C2 is skipped-by-failure rather than skipped-by-design if it
   is not.
5. **`destructive_transition` remains informational only** (spec D-7): `set_active` reports `true`
   and still switches.

## 5. Citations

1. `docs/tasks/evidence/T-01534-mcp-api-surface-implementation.md` — the surface under test.
2. `docs/tasks/evidence/T-01532-spec.md` — §4.1 (`get` parameters), §4.2 (`validate` `store_path`),
   §4.7, §8.1, §9 (audit target).
3. `code/aiosh-rust/aiosh-core/src/dispatch.rs` — `recorded_call`,
   `recorded_call_with_resolved_target`, `commit`.
4. `code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py` — C1..C4.
