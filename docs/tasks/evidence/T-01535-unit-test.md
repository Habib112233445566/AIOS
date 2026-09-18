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
| 2 | `register` wrote two different audit rows for the same operation — a `spec` path logged `target = None`, an inline `layout` logged the layout id — contradicting spec §9 | The layout id is recorded for **both** accepted forms on every row the tool body writes — the success row **and** the body-refusal row. The first pass closed only the success half; **§6 records the correction** | `test_mcp_fs_layout_register_audit_target_is_layout_id` + C2 |
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
pub fn recorded_call<F>(...)                        // unchanged signature; delegates with a no-op body target
pub fn recorded_call_with_body_target<F>(..., f: F) // F: FnMut() -> (Result<Value, String>, Option<String>)
```

The body returns its outcome **and** the target it managed to resolve, so the row target has exactly
one resolution point — `body_target.or(pre_gate)` — and that one value is used on every row written
past the gate: the success row and the body-error/refusal row alike. The pre-gate target is still
what the classifier and PEP see, and it is the fallback only when the body produced no target (and
the target of the gate's own refusal row, which is written before the body runs). `aios.fs_layout.register`
reports the id it parsed from the spec on both outcome paths:

```rust
let (outcome, resolved) = /* body: resolve `resolved = Some(id)` right after parsing */;
(outcome, resolved)
```

Verified against the audit ring after the §6 correction: `spec` path success →
`target = 'contract-spec-v1'`, inline success → `target = 'contract-inline-v1'`, and both duplicate
**refusals** → the same layout ids.

> **Correction — see §6.** As first shipped, this section's variant took
> `resolve_target: &dyn Fn(&Value) -> Option<String>` and consulted it **only on `Ok`**, so a
> `spec`-form refusal still fell back to the pre-gate `None` and left a duplicate-id attempt with no
> layout-queryable row. The description above is the corrected behaviour; §6 records the fix and the
> test that now catches it.

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
PASS: C2 register records the layout id as the audit target for both input forms, on success and on body refusal (pre-gate boundary asserted)
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
      ['grant_id', 'layout_id', 'profile', 'store_path'] but ...           (re-injected pre-fix manifest)
OK  C2 audit target: fails when broken -> spec path: audit target is None, expected the layout id
      'contract-spec-v1'
OK  C3 destructive verdict: fails when broken -> shrink must report destructive_transition=true
```

This first-pass control only injected the pre-fix **success** path; §6.3 repeats it as a real
binary-level control over the **refusal** path, where the original defect actually survived.

### 3.2 Acceptance mapping

| Acceptance criterion (`T-01535`) | Result |
|---|---|
| New test file runs standalone and passes | `test_fs_layout_mcp_contract.py` runs standalone, C1..C4 pass |
| Negative cases are asserted, not just happy path | C4 (ungranted, missing/oversize `store_path`, unknown tool) plus the three negative controls above |
| (ledger) cover valid input, invalid input, boundary values, primary failure mode | C1 valid schema set; C2 both input forms; C3 true/false verdicts; C4 boundary (1025-char path) and failure modes |

## 4. Honest Limitations

1. **Defect 2's fix touches shared core dispatch.** `recorded_call_with_body_target` is a new
   public function in `aiosh-core/src/dispatch.rs`; `recorded_call` keeps its signature and behaviour
   (it delegates with a no-op body target), and all ~120 existing call sites are untouched. The new
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
   `recorded_call_with_body_target`, `commit`.
4. `code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py` — C1..C4.

---

## 6. Post-Audit Correction (second pass over defect 2)

A second read-only audit of the pushed head (`b67b5bb`) probed the same surface and found defect 2
**half-landed**: on the failure path the two input forms still differed. The audit's live probe is
reproduced below, together with the root cause and the correction.

### 6.1 What was still wrong

| Probe (real `aiosh-mcp` over stdio) | Before §6 | After §6 |
|---|---|---|
| `register(spec)` duplicate → refusal row | `target = None` | `target = 'contract-spec-v1'` |
| `register(layout)` duplicate → refusal row | `target = 'contract-inline-v1'` | `target = 'contract-inline-v1'` |
| `register(spec)` success row | `target = 'contract-spec-v1'` | unchanged |
| `register(layout)` success row | `target = 'contract-inline-v1'` | unchanged |

The refusal text itself already named the id (`layout with id 'contract-spec-v1' is already
registered`), so the row was the only place the id was lost — and a duplicate-id attempt is exactly
the row an operator wants to find **by layout**. Spec §9 was therefore still violated on the failure
path, and this file's earlier claim ("the layout id is recorded for **both** accepted forms",
unscoped) overstated what had shipped. That claim is now scoped in §1 and corrected above.

### 6.2 Root cause and fix

The first variant resolved the target **only on `Ok`**, and fell back to the pre-gate parameter on
the error branch — two owners for one field, differing by code path. The correction gives the row
target a single resolution point used for every post-gate row:

```rust
// aiosh-core/src/dispatch.rs — recorded_call_with_body_target
let (outcome, body_target) = f();
// The single resolution point for every row written past the gate.
let row_target = body_target.or_else(|| pre_gate.map(|s| s.to_string()));
// ... same `row_target` is committed on Ok and on Err
```

`recorded_call` retains its exact signature and behaviour (it delegates with `move || (f(), None)`),
so all ~120 existing call sites are untouched, and the pre-gate target still governs the
classifier/PEP and the gate's own refusal row — so a row never claims authorization the call did not
receive. `aios.fs_layout.register` now reports the parsed id from its body on **both** outcome paths.

### 6.3 Coverage added, and the control that proves it

* **In-tree** (`test_mcp_fs_layout_register_audit_target_is_layout_id`): after the two success
  registrations, the same layout is registered again in both forms and each refusal row is asserted
  to carry the layout id with `outcome = "error"`.
* **Contract file C2**: extended from success rows only to the two refusal rows, plus the pre-gate
  boundary below. C2 asserts `outcome` as well as `target`, so a row that is merely *present* is not
  enough.
* **Negative control (real binary)**: the pre-fix behaviour was re-injected (`Err` branch commits
  `pre_gate` instead of `row_target`), the binary rebuilt, and C2 failed exactly as intended:

```
AssertionError: spec path (duplicate refusal): audit target is None, expected 'contract-spec-v1'
```

The injection was then reverted (verified: 0 occurrences of the control marker) and the binary
rebuilt before the final green run.

### 6.4 Documented boundary — the pre-gate refusal row

A **pre-gate** refusal (e.g. no PEP grant) is written before the body runs, so it can carry only a
target that needed no I/O. For `register`:

* inline `layout` → the id is read from the argument, so the refusal row names the layout;
* `spec` path → the id is only knowable by parsing the caller-supplied file, and that parse is
  deliberately **not** performed before authorization: an ungranted caller must not be able to
  trigger a read of an attacker-chosen path, and a FIFO named by `spec` must not be able to stall the
  single-threaded loop before policy is consulted (the F-1 hazard closed by `T-01531`).

So the spec-form pre-gate refusal row records `target = None` **by design**, while every row the tool
body writes (success or refusal) records the layout id. C2 asserts this boundary explicitly (it is
not left to an untested accident), and it is the one residual asymmetry: closing it would require
reopening the unauthorized-read hazard, so it is recorded rather than silently accepted.

### 6.5 Ledger placement

The defect being corrected belongs to this task's own scope (its test file and this evidence), and
`T-01535`/`T-01536` are already `done`. The ledger no-skip law forbids re-completing a completed id,
and the pointer's next task (`T-01537`, *Security Review*) was **not** performed by this pass, so the
pointer is left at `1537` rather than advanced on a claim this pass cannot support.
