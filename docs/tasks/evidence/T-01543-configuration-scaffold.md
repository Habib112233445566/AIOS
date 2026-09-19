# T-01543 — Filesystem Layout configuration: Scaffold

## Metadata
- **Task ID:** `T-01543`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / configuration
- **Status:** Complete — the T-01542 contract's N-1..N-8 declarations implemented and proven;
  all changes left **uncommitted** for the security-audit pass per the delivery protocol.
- **Date:** 2026-09-18
- **Dependencies:** `T-01542` (the specification, merged via PR #6 as `65a6f1b`).
- **Feeds:** T-01544 (configuration: Implementation) — this pass *is* the interface
  realization; the ledger's next tasks harden/document/verify it.

## 1. What the ledger asked, and how it was reconciled with the contract

The T-01543 entry is the generic scaffold template: "Define typed function signatures /
interfaces only; bodies fail loudly (throw/NotImplementedError)" and "New interfaces exist and
are referenced by at least one call site or test stub." Taken literally, stub-bodied validators
would contradict the user's authoritative directive to *implement N-1..N-8 and V-1..V-7* — and
stub bodies that "fail loudly" would break every existing caller (`from_json` runs the
validator; every CLI verb and MCP tool validates). **Reconciliation (recorded, not silent):**
this pass treats "scaffold" as *interfaces + their behavior, wired into every surface*, which
satisfies the acceptance criteria exactly — the project builds with zero errors (criterion 1)
and every new interface is referenced by call sites and tests (criterion 2). Nothing was left
as a throwing stub; the two acceptance criteria are met in both letter and spirit.

No instructions/artifacts path discrepancy this time: the artifacts path is
`docs/tasks/evidence/T-01543-configuration-scaffold.md`, which is this file.

## 2. What was implemented (contract N-1..N-8, exactly)

All in `code/aiosh-rust/aiosh-core/src/fs_layout.rs` unless noted; signatures of every
frozen [REUSED] interface unchanged (spec §7).

- **N-1 (D1, E-1, V-1):** `#[serde(deny_unknown_fields)]` added to all five spec serde
  representations — `FsType`, `PartitionType` (both alongside the existing
  `rename_all = "snake_case"`), and `MountPointSpec`, `PartitionSpec`, `DirectorySpec`,
  `FilesystemLayoutSpec`. Unknown fields now fail deserialization naming the first offender
  (`unknown field \`dry_run\`, expected one of …`), at every entry: `from_json` (CLI `--spec`,
  MCP `spec`), `serde_json::from_value` (MCP inline `layout`), and store load. **Scope note
  per spec V-1:** `FilesystemLayoutStore` (the store file) was deliberately left tolerant —
  spec §5 V-1 makes that "recommended, not required", and spec §7 freezes the store format
  ("no store-format change"); T-01544 may align it.
- **N-2 (D2, E-2, V-2):** `validate_directory_spec` rejects `mode == 0` and `mode > 0o7777`
  with exactly `directory '<path>' mode must be in 1..=0o7777 (octal), found <value>`
  (unnumbered, per the spec's two-class message convention V-7).
- **N-3 (D3, E-3, V-3):** `validate_mount_point`'s FL4 block gains the exact-token `noexec`
  check for `/tmp` and `/dev/shm`: `FL4 violation: mount '<path>' missing mandatory security
  option 'noexec'`.
- **N-4 (D4, E-4, V-4):** `validate_directory_spec` constrains a present `symlink_target` to
  the UsrMerge shape — not starting with `/`, no `.`/`..` segment, first segment exactly
  `usr`: `directory '<path>' symlink_target '<t>' must be a relative path under 'usr'
  (UsrMerge)`. `None` stays exempt.
- **N-5 (D7, E-5/E-6, V-5):** (a) `validate_filesystem_layout` (top-level shape section,
  before FL1) requires `created_at` to parse as RFC 3339 **and** end in `Z` — a valid
  offset form (`+02:00`) is refused per the contract's UTC-only wording; the round-trip
  equality first cut was caught by the contract test and corrected. Message:
  `layout 'created_at' must be an RFC 3339 UTC timestamp, found '<value>'`. (b)
  `validate_mount_point` (after the device checks) bounds `dump ∈ {0,1}`:
  `mount '<path>' dump must be 0 or 1, found <n>`.
- **N-6 (D8, E-7, V-6):** FL6 in `validate_filesystem_layout`, immediately after FL1 per the
  spec's ordering rule: `FL6 violation: at least one mount must be marked required`.
- **N-7 (O1):** the validate success string is now `VALID: Filesystem layout '<id>' satisfies
  all FL1..FL6 invariants` (CLI `aiosh-cli/src/main.rs:4350`); MCP tool description and
  dispatch summary updated to `FL1..FL6` (`aiosh-mcp/src/main.rs:1011,3127`); stale FL1..FL5
  comments updated. D5's wording update applied: the four mutation-tool `store_path`
  descriptions drop "…yet" (`(required: no default store is defined)`).
- **N-8 (docs):** `docs/filesystem_layout.md` — §3.1 retitled `FL1..FL6` with FL4's `noexec`
  added, FL6 documented, and the unnumbered per-element rules named; §4 register text gains
  the unknown-field refusal and FL6; the §6.15 residual retitled to its honest two-part
  state ("Undeclared *Tool* Arguments Are Ignored…; Unknown *Spec* Fields Are Refused") —
  the MCP tool-argument boundary stands, the spec-document half is closed; remaining
  `FL1..FL5` strings updated. `docs/README.md` (2), `code/aiosh-mcp/README.md` (2) likewise.

The sealed-store residual (§6.12) and every other documented finding are untouched.

## 3. Compatibility proof (built-ins stay valid — spec §9's requirement)

Probe-captured before coding, re-proven after: both `aios-uefi-standard-v1` and
`aios-container-minimal-v1` validate under the full new rule set through the real binary —
`VALID: Filesystem layout 'aios-uefi-standard-v1' satisfies all FL1..FL6 invariants`, exit 0
(container ditto). Unit-test form: `test_both_builtins_remain_valid_under_new_rules`.

## 4. Revert-controlled proof (component convention)

The nine new tests in `aiosh-core/tests/test_fs_layout_data_model.rs`
(`test_unknown_json_field_rejected_{top_level,nested}`, `test_directory_mode_range_enforced`,
`test_fl4_noexec_required_on_tmp_and_dev_shm`, `test_symlink_target_usrmerge_shape_enforced`,
`test_created_at_must_be_rfc3339_utc`, `test_dump_bounded_to_zero_or_one`,
`test_fl6_at_least_one_required_mount`, `test_error_ordering_matches_spec_section_4`) pin
every E-1..E-7 message and the §4 ordering rule. Procedure: with all seven code additions
stripped from `fs_layout.rs` (simulated revert), the run gives **9 failed / 20 passed —
exactly the new tests fail, every pre-existing test passes**; the fix was restored
md5-identical (`615fe1db00c75b2e06fb420dd7bc5d9a` both sides) and the file then runs
**29/0**. The pre-existing suite never guarded these rules — which is why they were gaps.

One pre-existing test was updated, not to pass a regression but because the new contract
made its fixture illegal: `test_fl3_duplicate_mount_paths` planted a duplicate `/tmp` mount
carrying `nodev,nosuid` but no `noexec`; E-3 now correctly fires before FL3. The duplicate
gained `noexec` (comment documents why) so the test keeps pinning what it was written to pin.

## 5. Real-binary surface proof (26/26 assertions)

`aiosh-mcp.exe` over stdio (grants minted by the real CLI, temp `AIOSH_HOME`, temp
`store_path`) and `aiosh.exe` directly:

- Positive control: a valid custom layout registers via inline `layout` +
  `grant_id`; `ok:true`, `registered:true`, `audit_id` present; its row is
  `outcome=ok`.
- E-1 (inline, unknown field `dry_run`): `isError:true`, wording names the
  offender; exactly **one** register row attributed to the refused layout id,
  `outcome=error`, and the response's `audit_id` matches that row. Nothing staged.
- E-3 via the **spec-file** form (`noexec` stripped from `/tmp`): refused with the
  contract-exact message — the same rule through the second input path.
- E-7 (all `required:false`), E-6 (`dump:2`), E-2 (`mode:0`), E-4 (absolute
  symlink target), E-5 (`+02:00` offset form): each refused with the exact
  contract message.
- Store integrity after all refusals: exactly the successful registration is
  persisted, no `t01543-e*` id present, no staged `.tmp` residue. Never deleted,
  per-store scoping preserved.
- CLI: success string is `FL1..FL6` for both built-ins; `--spec` refusal
  (E-2) exits 1 with the contract message; `import-fstab` works unchanged — its
  hardcoded `2026-09-16T00:00:00Z` passes E-5, so fstab ingestion is unaffected.

One process lesson recorded honestly: the first E-5 probe returned VALID from a **stale
binary** (`cargo test` had rebuilt test binaries but not the shipped ones); after an explicit
`cargo build`, the fresh binary refuses the offset form. The probe claims rest on the fresh
build.

One collateral fix found by the battery (FL4 security suite): `_malicious_store()` embedded
its ESC payload in `created_at` (`20260101<ESC>[2J`), which E-5 now refuses at parse time —
the injection-hardening rule eliminated that route. The fixture's payloads moved to fields
the contract leaves free-text (`directories[0].description` added; `created_at` becomes a
valid timestamp, whose verbatim fstab-header echo the test still asserts), and the suite
passes for the right reason: rendering still neutralizes, JSON and store stay faithful.

## 6. Battery at the final state

| Check | Result |
|---|---|
| `cargo build -p aiosh-core -p aiosh-cli -p aiosh-mcp` | zero errors, zero warnings |
| `cargo test -p aiosh-core -p aiosh-mcp -p aiosh-cli` | **653 passed / 0 failed** |
| `python tools/test_fs_layout_suites.py` (FL1–FL8; MCP contract C1–C8 inside FL8) | **PASS**, all criteria |
| Real-binary surface probe (§5) | **26/26 assertions** |
| `python tools/check_task_docs.py` | **PASS** (C1..C6) |
| `python tools/task_ledger.py validate` | **consistent**, pointer 1543 (pre-advance) |
| `ci/run_all_smokes.sh` (mandated baseline gate, attempted for the record) | **FAIL as documented** — `rust_smoke` fails with the host-level WSL-stub error, identical to T-01538/T-01540/T-01541/T-01542's record |
| Working tree | only this task's intended files; probe script deleted before commit time |

## 7. Acceptance check against the ledger

- *Project builds/imports with zero errors* — cargo build clean (§6, row 1).
- *New interfaces exist and are referenced by at least one call site or test stub* — every
  new rule lives inside the existing called validators (call sites: all CLI verbs, all MCP
  tools, `from_json`, store load) and is pinned by the nine new tests (§4).

**Defects/blockers: none outstanding.** Two honest notes: the scaffold-vs-implement
reconciliation in §1, and the N-8 flag in the spec pointed at a "§6.13 mode claim" that did
not exist — the actual stale claims (§3.1, §6.15, mcp/README) are the ones corrected in §2.
The stale-binary lesson and the FL4 fixture re-route are in §5. Changes are left
uncommitted for the security-audit pass; the pending tree contains exactly:
`fs_layout.rs`, `aiosh-cli/src/main.rs`, `aiosh-mcp/src/main.rs`,
`test_fs_layout_data_model.rs`, `test_fs_layout_audit_security.py`, the three doc files,
the two ledger-state files, and this evidence file (+ the tool-generated completion note).
