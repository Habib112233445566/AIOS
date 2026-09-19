# T-01544 — Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / configuration: Implementation

## Metadata
- **Task ID:** `T-01544`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / configuration
- **Status:** Complete — the one contract item T-01543 deliberately deferred is now implemented,
  tested and documented; changes left **uncommitted** for the security-audit pass (see
  `T-01544-configuration-security-audit.md`, the audit the delivery protocol requires before commit).
- **Date:** 2026-09-19
- **Depends on:** `T-01543` (the scaffold pass that realized N-1..N-8).
- **Feeds:** T-01545 (configuration: Unit Test) — the two tests added here are the targeted proof
  this task requires; T-01545 deepens them.

## 1. Scope decision (recorded, not silent)

The T-01544 entry is the generic implementation template ("write a failing test first", "implement
the smallest change that passes"). T-01543 had already realized the whole T-01542 contract
(N-1..N-8) and reconciled that with the scaffold template in its own §1. The **one** contract item
left open by that pass is T-01542 §5 **V-1's second half**: unknown-field rejection on the store
*document*, which the spec marks "recommended, not required" and T-01543 recorded as
"**T-01544 may align it**". That is this task's change: the last deserialization entry that
tolerated a field it did not know.

Nothing else was invented: no new CLI verb, no new MCP tool, no new env var, no new file, **no
store-format change** (the store's field set and bytes are unchanged — only the tolerance is gone).

## 2. What was changed

| File | Change |
|---|---|
| `code/aiosh-rust/aiosh-core/src/fs_layout_service.rs` | `#[serde(deny_unknown_fields)]` on `FilesystemLayoutStore`, with a doc comment stating the contract and the failure it closes. This is the entire behavioral change. |
| `code/aiosh-rust/aiosh-core/tests/test_fs_layout_service.rs` | Two new tests (below), written **before** the attribute was added. |
| `docs/filesystem_layout.md` | §2.1 bullet on the store's parse contract; §6.15 extended (the store document joined the closure); new **§6.22** recording the rule *and* its honest boundary. |
| `code/aiosh-cli/tests/test_fs_layout_cli_smoke.py` | One stale comment `FL1..FL5` → `FL1..FL6` (comment only, no assertion changed) — found by the audit pass, recorded as its F-4. |

The defect the change closes, live for one transaction before it: a store spelling
`active_layout` instead of `active_layout_id` **loaded successfully** — the unknown key was
ignored and the built-in default stayed active, so the operator mutated a store that meant
something other than what it said. Every other parse entry (all five spec structs, `from_json`,
MCP inline `layout`, MCP `spec`) had failed loudly on the same typo since T-01543; the store file
was the exception.

## 3. Failing test first (as the ledger instructs)

Both tests were added and run before the implementation:

```
test test_fs_layout_store_rejects_unknown_top_level_field ... FAILED   (Result::unwrap_err() on an Ok value)
test test_fs_layout_store_rejects_unknown_nested_field     ... ok
test result: FAILED. 20 passed; 1 failed
```

The top-level test failed for exactly the right reason: the crafted store *loaded*, printing the
whole store as the `Ok` payload. The nested test passed pre- and post-change, and that is recorded
honestly rather than dressed up: the nested rule is T-01543's (`deny_unknown_fields` on the spec
structs), and the test's value is that it pins the rule **through the store path**, not that it is
new behavior.

## 4. Revert-controlled proof (component convention)

Simulated revert = the one added attribute stripped from `fs_layout_service.rs`, rebuilt:

| State | Result |
|---|---|
| Attribute stripped (revert) | **20 passed / 1 failed** — only `..._rejects_unknown_top_level_field` fails; every pre-existing test passes |
| Attribute restored | **21 passed / 0 failed**; file restored **md5-identical** (`1f57eaa89631e56cb79680f3f9d103d0` both sides) |

So the new test is load-bearing for the new rule, and the rule is the only thing it guards.

## 5. Compatibility proof (the spec's additive-only requirement)

- A store written by this tool still loads unchanged (`test_fs_layout_store_rejects_unknown_top_level_field`
  loads the freshly saved store before corrupting it; the write path emits only `active_layout_id`
  and `layouts`).
- **Byte fidelity:** re-saving a store whose state did not change reproduces the previous bytes
  exactly (md5 compared before/after a `set-active` that returns to the same value; a real state
  change does change the bytes). No serialization drift from the attribute.
- **Enum variants:** the T-01543 change also put `deny_unknown_fields` on `FsType`/`PartitionType`;
  a live probe registered layouts using `{"custom": "zfs"}` and a custom partition-type GUID
  successfully, so the enum-level attribute rejects unknown *variant* spellings without breaking
  the documented `Custom` payloads.
- **fstab ingestion unaffected:** `import-fstab` on a 2-line sample registers successfully (its
  hardcoded `created_at` passes E-5), and an fstab whose dump field is `2` is refused with the
  E-6 message (`mount '/' dump must be 0 or 1, found 2`) — the new rules gate the import path
  rather than bypassing it.
- Both built-ins still validate: `VALID: … 'aios-uefi-standard-v1'/'aios-container-minimal-v1'
  satisfies all FL1..FL6 invariants`, exit 0, from the freshly built binary.

## 6. Battery at the final state

| Check | Result |
|---|---|
| `cargo build -p aiosh-core -p aiosh-cli -p aiosh-mcp` | zero errors, zero warnings |
| `cargo test -p aiosh-core -p aiosh-mcp -p aiosh-cli` | **655 passed / 0 failed** (T-01543 recorded 653; +2 = these tests) |
| `python tools/test_fs_layout_suites.py` (FL1–FL8; MCP contract C1–C8 inside FL8) | **PASS**, all criteria |
| Real-binary refusals (CLI `register`/`list`/`set-active`, MCP `register`/`set_active`) | `LOAD_STORE_FAILED` / `isError:true`, exact serde wording naming the field, exit/rc as documented |
| `python tools/check_task_docs.py` | **PASS** (C1..C6) |
| `python tools/task_ledger.py validate` | **consistent** (pointer 1544, pre-advance) |
| Working tree | only this task's intended files (plus the still-pending T-01543 tree it builds on) |

**Binaries were rebuilt before every live probe** (`cargo build`, not just `cargo test`) — the
stale-binary lesson recorded in T-01543 §5 is applied, not repeated.

## 7. Honest boundaries carried forward

1. **An unknown-field store is unloadable and not repairable in-tool** — the §6.12 sealed-store
   analogue, now documented as **§6.22**: every verb loads before it writes, so recovery is
   external (drop the unknown key). Reachable only from a hand-edited store or a cross-version
   writer; this tool's own writes are always clean. Fail-closed, so nothing is lost — a store
   that says something else is refused, not partially applied.
2. **Forward compatibility is deliberately traded for loud failure**: a future version that adds a
   store key would write stores this version refuses. That is the same choice D1 made for specs
   (a silent misreading is worse than a refusal) and is recorded in §6.22 rather than implied.
3. The verification here is Windows-host evidence; the Linux-only paths are unaffected by this
   change (it is pure deserialization policy) and their own proofs remain as T-01528 recorded them.

## 8. Acceptance check against the ledger

- *Targeted test passes* — two targeted tests in `aiosh-core/tests/test_fs_layout_service.rs`,
  21/0 in isolation, with a revert-controlled proof that they guard the change (§4).
- *No regression in existing smoke suites for touched modules* — 655/0 across the three crates and
  FL1–FL8 PASS through the real binaries (§6).
- *Failing test first / smallest change / reuse helpers / no new dependencies / one audit row per
  consequential call* — §§3, 2, 6; the audit pass independently confirmed exactly-one-row on every
  refusal path this change can produce (its §3, F-verified).

**Defects/blockers: none outstanding.** The two findings the audit raised on this diff are Low/Info
and are recorded in `T-01544-configuration-security-audit.md` (§F-1, §F-2, §F-3; §F-4 is the
one-line stale-comment fix already applied). Changes remain uncommitted pending that audit's
acceptance.
