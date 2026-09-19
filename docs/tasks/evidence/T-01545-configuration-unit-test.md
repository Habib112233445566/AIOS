# T-01545 — Filesystem Layout configuration: Unit Test

## Metadata
- **Task ID:** `T-01545`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / configuration
- **Status:** Complete — new standalone wire-level unit-test suite delivered; **no source files
  changed** (test-only task); changes left **uncommitted** on top of the pending T-01543/T-01544
  tree per the delivery protocol.
- **Date:** 2026-09-19
- **Depends on:** `T-01544` (the implementation this suite pins).
- **Feeds:** T-01546 (configuration: Integration) — wiring this suite into the FL runner and any
  further surface integration is that task's scope (same split the matrix epic used: scaffold
  T-00063 deferred CI wiring to integration T-00066).

## 1. What the ledger asked, and what was built

The entry instructs: cover *valid input, invalid input, boundary values, and the primary failure
mode*; use the existing smoke-test style; assert observable behavior, not implementation details;
run standalone; and confirm the file **fails when the feature is broken**.

The T-01542 contract (E-1..E-7, ordering rule) was already pinned at the *library* level in-tree
by the T-01543 tests (`test_fs_layout_data_model.rs`) and the store-parse half by T-01544
(`test_fs_layout_service.rs`). What no suite pinned is the **operator surface**: exit codes, JSON
envelope codes, the fail-closed store-integrity guarantee on refusal paths, and the human
(non-JSON) output path. This task's artifact is
`code/aiosh-cli/tests/test_fs_layout_config_validation.py` — 12 test groups driven through the
real `aiosh` binary, same harness style as `test_fs_layout_cli_smoke.py` (fixtures fetched from
the binary itself, never hand-written).

| Group | Covers |
|---|---|
| U1 | valid: both built-ins, boundary values (`mode 0o7777`, `dump 1`, `Z` form), compat `{"custom":"zfs"}` payloads |
| U2 | E-1 unknown fields: top-level, nested, unknown enum variant — offender named, parse-stage `id: "unknown"` |
| U3 | E-2 mode `0` and `4096` refused with exact wording; `0o7777` legal (pinned in U1) |
| U4 | E-3 FL4 `noexec` on `/tmp` **and** `/dev/shm`; exact-token boundary (`noexec2`, `noexec=1` do not satisfy) |
| U5 | E-4 symlink targets (`/usr/bin`, `bin/usr`, `usr/../etc`, `usr/./bin`) refused; `usr/bin` legal |
| U6 | E-5 `created_at`: junk, naive, and `+02:00` offset refused; offending value echoed verbatim |
| U7 | E-6 `dump 2` refused with the exact path and value |
| U8 | E-7 FL6 floor (one required mount suffices) **+ the §4 ordering rule** (E-5 > FL6 > E-2) |
| U9 | the same contract wording through the spec-**file** input form |
| U10 | register-path refusals: `SPEC_PARSE_FAILED` envelope + fail-closed store (md5-identical, refused ids absent, no staged residue) |
| U11 | **T-01544 store contract**: unknown top-level and nested fields in the store file refused by read *and* mutate verbs (`LOAD_STORE_FAILED`), store byte-identical, recovery external (drop the key → loads again) |
| U12 | human (non-JSON) path: `INVALID: …` / load-failure lines on stderr, empty stdout, exit 1 |

## 2. Empirical-first procedure

Every expected value in the suite was captured from a **freshly built** binary *before* the file
was written (`cargo build` of all three crates, zero warnings; the T-01543 stale-binary lesson
applied from the start): the exact `LOAD_STORE_FAILED` / `VALIDATION_FAILED` / `SPEC_PARSE_FAILED`
codes, the serde offender wording, the `INVALID: ` human prefix, and the md5-unchanged/no-residue
behavior on refusal. No assertion encodes a guessed string.

## 3. Standalone run (acceptance 1)

`python code/aiosh-cli/tests/test_fs_layout_config_validation.py` → **U1..U12 all PASS**, exit 0.

## 4. Broken-feature proof (acceptance 2; component convention)

Snapshot md5s matched the audited state exactly before sabotaging (`fs_layout.rs`
`615fe1db00c75b2e06fb420dd7bc5d9a` — the T-01543 evidence value; `fs_layout_service.rs`
`1f57eaa89631e56cb79680f3f9d103d0` — the T-01544 evidence value). Three *independent* rules were
sabotaged in one build:

1. E-3 `noexec` check disabled (`if false && …` in `validate_mount_point`),
2. E-7 FL6 check disabled (same treatment in `validate_filesystem_layout`),
3. T-01544's `deny_unknown_fields` stripped from `FilesystemLayoutStore`.

Result on the sabotaged build: **exactly U4, U8, U11, U12 failed; the other 8 passed** —
- U4 died on the disabled noexec rule (`envelope code 0 != 1`);
- U8 died on FL6 (its ordering half depends on FL6 existing, so one sabotage kills both halves);
- U11 died on the store attribute (top-level refusal gone) — and U12's store leg with it;
- every test guarding a rule that was *not* sabotaged kept passing, so the failures are
  attributable to the exact broken features, not to collateral breakage.

Both files restored **md5-identical** (same two values, verified), full rebuild, suite green
12/12 again.

**One honest process note (the stale-binary trap, hit and corrected mid-proof):** the first
sabotage attempt replaced the attribute with a non-existent `deny_unknown_fields_off` — serde
rejects it, `cargo build` **failed**, and the suite (run against the stale binary) passed
everything, which would have been a false "feature is not broken" conclusion. The build output
was checked before believing the result; the revert was redone with a compile-valid removal and
§4's numbers come from that corrected run. The suite itself always runs against whatever binary
is current; the evidence only counts on a fresh build.

## 5. Battery at the final state

| Check | Result |
|---|---|
| `cargo build -p aiosh-core -p aiosh-cli -p aiosh-mcp` | zero errors, zero warnings |
| `cargo test -p aiosh-core -p aiosh-mcp -p aiosh-cli` | **655 passed / 0 failed** (unchanged from T-01544 — test-only task) |
| `python code/aiosh-cli/tests/test_fs_layout_config_validation.py` | **U1..U12 PASS** |
| `python tools/test_fs_layout_suites.py` | **PASS** — FL1..FL8 all criteria (incl. MCP contract C1..C8 in FL8) |
| `python tools/check_task_docs.py` | **PASS** (C1..C6) |
| `python tools/task_ledger.py validate` | **consistent** (pointer 1545, pre-advance; the `evidence` warning is the pre-existing historical backfill list, unchanged by this task) |
| Working tree | this suite file + the pre-existing uncommitted T-01543/T-01544 tree it pins; no source changes |

## 6. Honest boundaries

1. **CLI surface only.** The MCP tools (`aios.fs_layout.*`) share the same validator and store
   loader in `aiosh-core`, but their wire envelopes, PEP gating, and audit rows are deliberately
   out of scope here — they remain covered by FL6 (cross-surface parity) and FL8 (contract
   C1..C8). A refusal wording change would be caught by this suite through the CLI and by FL1
   in-tree; only MCP-specific plumbing would need FL8.
2. **Runner wiring deferred by convention.** The file is standalone per the ledger's Unit Test
   acceptance; adding it to `tools/test_fs_layout_suites.py` (or CI) is T-01546's discoverability
   work, mirroring the scaffold→integration split the matrix epic used (T-00063 → T-00066).
3. **Sabotage coverage is 3 of the 9 rules.** Every E-1..E-7 rule has at least one exact-wording
   assertion (U2..U8), but the broken-feature proof disabled three representative rules spanning
   both files and both spec/store contracts; a per-rule revert matrix was not run. The in-tree
   Rust suites provide the second, independent guard for each rule.
4. Windows-host evidence; the suite spawns the real binary and touches only temp directories.
5. No new dependencies; Python stdlib only, same harness shape as the neighboring smoke suites.

## 7. Acceptance check against the ledger

- *New test file runs standalone and passes* — §3 (12/12, exit 0).
- *Negative cases are asserted, not just happy path* — U2..U12 are refusals/envelope codes/
  integrity checks; exact contract wording asserted throughout.
- *Valid input, invalid input, boundary values, primary failure mode* — U1/U3-U8 boundaries,
  U10/U11 primary store failure modes, §1 table.
- *Matching tests/ directory, existing smoke-test style* — `code/aiosh-cli/tests/`, same helpers
  and fixture discipline as `test_fs_layout_cli_smoke.py`.
- *Fails when the feature is broken* — §4, three-rule sabotage with predicted-and-observed
  failure matrix and md5-identical restore.

**Defects/blockers: none outstanding.** One process lesson recorded (§4 stale-binary false
sabotage). Changes uncommitted pending the chain's audit pass; the tree now contains exactly
this file plus the T-01543/T-01544 set.
