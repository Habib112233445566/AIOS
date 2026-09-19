# T-01546 — Filesystem Layout configuration: Integration

## Metadata
- **Task ID:** `T-01546`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / configuration
- **Status:** Complete — the configuration contract is integrated into the durable test battery and
  proven at parity across both surfaces; changes left **uncommitted** on top of the pending
  T-01543/T-01544/T-01545 tree per the delivery protocol.
- **Date:** 2026-09-19
- **Depends on:** `T-01545` (the CLI wire suite this task wires into the aggregate runner).
- **Feeds:** T-01547 (configuration: Security Review) — the parity case added here (C9) is
  additional refusal-surface evidence that review can build on.

## 1. What the ledger asked, and how it reconciles with the existing surfaces

The instructions ask to (a) wire the feature into its real call path, (b) confirm cross-substrate
parity, (c) update the registration point for discoverability, and (d) run the closest existing
smoke suite. The *feature itself* — the T-01542 validation contract and the T-01544 store parse
contract — already runs inside its real production call paths: every CLI `layout` verb
(T-01531..T-01525 surfaces) and every `aios.fs_layout.*` MCP tool validates and loads through the
same `aiosh-core` code, and C1 of the FL8 suite pins that the MCP manifest advertises exactly the
ten tools' argument contracts. Nothing new needed registering on those surfaces (no new verb, no
new tool, no new env var, no new file — the T-01542 §7 additive-only rule).

The integration gaps this task closes are the two durable ones:

1. **Parity was asserted, not pinned.** The T-01543/T-01544 security audits probed the MCP surface
   by hand and found the same refusals as the CLI, but no test pinned that equality. A wording
   drift between the surfaces would have been invisible to every suite.
2. **The T-01545 suite was standalone only.** Like the matrix epic's scaffold (T-00063) before its
   integration task (T-00066), the config-validation suite was not part of the aggregate FL
   battery, so a regression would only surface if someone remembered to run it.

## 2. What was changed

| File | Change |
|---|---|
| `code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py` | New **C9 configuration-contract parity** case (below) + docstring entry + `hashlib` import + `CONTAINER_ID` constant. No assertion of C1..C8 touched. |
| `tools/test_fs_layout_suites.py` | **FL9** registered: the config-validation suite joins the aggregate runner (docstring criterion, `test_fl9_configuration_contract`, suites list, PASS line). |
| `docs/README.md` | The one living reference to the runner updated `FL1..FL8` → `FL1..FL9`. |

## 3. C9 — the parity case (FL8 suite, now C1..C9)

Driven through the real `aiosh-mcp` binary over stdio with a real PEP grant, against a real temp
store, using the CLI only as the parity oracle and audit reader:

- **E-2 parity, message-verbatim:** `aios.fs_layout.register` with an inline `mode: 0` layout is
  refused, and the refusal text must contain the *exact message* the CLI `validate` verb returns
  for the same document (`directory '/var/lib/aios' mode must be in 1..=0o7777 (octal), found 0`).
  The CLI half is fetched live, so if one surface's wording ever moves, this pair fails loudly.
- **One honest audit row at a new refusal site:** the inline-form parse refusal row carries
  `outcome=error` and `target=contract-c9-mode0` (the C2 target property, now proven at the
  configuration refusals too), and the response's `audit_id` matches it.
- **E-1 parity:** the spec-string form with an unknown top-level field is refused with the same
  serde offender wording (`unknown field \`dry_run\``), also one honest error row.
- **Fail-closed:** after both refusals the store holds exactly the warmed state, the refused ids
  never appear, and no staged `.tmp` file exists.
- **T-01544 store contract through MCP:** a store file with an unknown top-level key
  (`active_layout`) is refused by `list` (read) *and* `set_active`/`register` (mutate) with the
  same `failed to deserialize layout store from … unknown field \`active_layout\`` wording as the
  CLI, the file is byte-identical after the refusals (md5), and dropping the key recovers it —
  the documented external-recovery step, now pinned on the second surface.

**Harness honesty:** three bring-up defects were mine, in the test, not the product, and are
recorded: (1) the audit-row map was snapshotted before the E-1 refusal, so the new row's lookup
failed (`21663` not found) — fixed by re-reading rows after the call; (2) a missing
`CONTAINER_ID` constant; (3) the docstring edit initially missed its anchor and was re-applied.
No product behavior was changed by this task, so no revert-proof applies; the parity claims rest
on the live pair checks above, and every probe ran against binaries freshly built in T-01545
(zero-warning build, unchanged since).

## 4. Cross-substrate parity, stated precisely

The ledger's clause names "the shared SQLite DB or canonical JSON". This component's store is a
JSON document, not the shared SQLite audit/PEP DB, and both surfaces share **one** Rust substrate
(`aiosh-core`); the MCP wrapper reads and writes the same store bytes the CLI does. The honest
reading, recorded rather than silently narrowed: the substrate-parity clause is vacuous here, and
the parity that *is* meaningful — same validator, same refusal wording, same fail-closed store
semantics on both surfaces — is exactly what C9 pins, complementing FL6's existing CLI↔MCP
behavioral parity and the canonical-JSON cross-substrate invariant that lives in the task-ledger
chain (rust_smoke parity steps).

## 5. Discoverability

- The production surfaces were already registered and advertised (CLI help lists every verb; MCP
  `tools/list` advertises all ten `aios.fs_layout.*` tools with `additionalProperties: false`,
  pinned by C1).
- The **suite** is now discoverable the same way every sibling subsystem's is: as a criterion of
  the aggregate runner `tools/test_fs_layout_suites.py` (FL9), with the runner's docstring, test
  function, suites list, and PASS line all extended, and the `docs/README.md` quick-start line
  updated to `FL1..FL9` (the only living stale reference; frozen evidence files keep their
  historical counts by convention).

## 6. Battery at the final state

| Check | Result |
|---|---|
| `cargo build` (all three crates) | zero errors, zero warnings (unchanged from T-01545) |
| `cargo test -p aiosh-core -p aiosh-mcp -p aiosh-cli` | **655 passed / 0 failed** |
| `python tools/test_fs_layout_suites.py` | **PASS — FL1..FL9** (FL8 now runs C1..C9) |
| `python code/aiosh-cli/tests/test_fs_layout_config_validation.py` standalone | **U1..U12 PASS** |
| `python tools/check_task_docs.py` | **PASS** (C1..C6) |
| `python tools/task_ledger.py validate` | **consistent** (pointer 1546, pre-advance) |
| `bash ci/run_all_smokes.sh` (baseline gate, attempted for the record) | **FAIL as documented** — `rust_smoke` host-level WSL-stub error, identical to the T-01538/T-01540/T-01541/T-01542/T-01543 record |
| Working tree | the pre-existing T-01543/T-01544/T-01545 set + this task's three files; no source changes |

## 7. Honest boundaries

1. **C9 pins two representative refusal classes plus the store contract on MCP, not all seven
   E-rules.** The full E-1..E-7 matrix remains pinned on the CLI (U2..U8) and in-tree
   (data-model tests); the MCP surface shares the validator, so a per-rule MCP matrix would
   duplicate the in-tree coverage. What C9 adds is the envelope/audit/store plumbing parity that
   only a wire test can see.
2. **The aggregate CI gate cannot run on this host** (WSL-stub). The FL suites — including FL9 —
   run green individually, which is the same evidence shape every task since T-01538 recorded.
3. Windows-host evidence; the Linux-only runtime paths are untouched (test/discovery-only task).
4. No new dependencies; stdlib only.

## 8. Acceptance check against the ledger

- *Feature reachable through its production surface* — C9's positive controls drive
  `register`/`set_active`/`list` through the real `aiosh-mcp` binary with PEP grants against a
  real store (§3), and the CLI surface through the U-suite; both surfaces already advertise the
  feature (§5).
- *Integration smoke passes end-to-end* — `tools/test_fs_layout_suites.py` FL1..FL9 all PASS,
  including the new parity case and the newly wired FL9 (§6).

**Defects/blockers: none outstanding.** Three harness defects found and fixed during bring-up are
recorded in §3. Changes uncommitted pending the chain's audit pass; the tree now contains exactly
the T-01543/T-01544/T-01545 set plus `test_fs_layout_mcp_contract.py`,
`tools/test_fs_layout_suites.py`, and `docs/README.md`.
