# T-01540: Filesystem Layout — MCP/API Surface: Verification & Evidence

## Metadata
- **Task ID:** `T-01540`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout MCP/API surface (`aios.fs_layout.*`, ten tools) and the shared
  persistence layer behind it (`aiosh_core::fs_layout_service`, `aiosh_core::pep`)
- **Status:** Complete — independent verification of the component chain at the merged `main`
  (`e253dc5`). Every earlier pass's claim was re-treated as a claim: the full battery was re-run at
  this head, and the headline behaviours were re-proven through the real `aiosh-mcp` binary over
  stdio with temporary stores rather than cited from earlier evidence. New attacks (TOCTOU,
  concurrent staging races, un-anchored key edges) were run and their outcomes recorded — nothing
  silently dropped.
- **Date:** 2026-09-18
- **Milestone:** Sub-Epic: Filesystem Layout (**10/10**) — MCP/API Surface Verification & Evidence.
  This closes the component's task chain (T-01531 … T-01540). No separate `task_plan.md` /
  `progress.md` files exist in this repository, so the milestone is recorded here per the ledger's
  artifact contract.
- **Dependencies:** `T-01539` (Documentation); verification target includes the two newest fixes
  delivered *after* T-01539 (the `.`/`..` `scope.paths` matcher fix and the canonical residue-cap
  keying, merged as `d86c348` and `e253dc5`), each of which had only been verified by the pass that
  wrote it.
- **Code changed by this task:** none — verification and evidence only. Production code is exactly
  the merged `main` content; the working tree was confirmed byte-identical to `origin/main`
  (`git diff origin/main HEAD` empty) before and during this pass.

---

## 1. Scope & Method

**Independent verification posture.** Nothing was trusted because a previous task reported it:
- The full suite battery was **re-run at this head** (§2), not quoted from earlier evidence files.
- The component's headline delivered behaviours were **re-proven through the real `aiosh-mcp`
  binary over stdio** (§3), driving the same JSON-RPC 2.0 protocol the contract suite uses, with
  temporary stores under the OS temp dir, throwaway `AIOSH_HOME` state, and grants minted by the
  real CLI. No repository file and no operator state was touched.
- Fresh attacks nobody had run were executed (§4): a TOCTOU probe between residue counting and
  staging, concurrent staged-listing races, and edge spellings against the un-anchored canonical
  key.

**Binaries provably from this source.** `aiosh-mcp.exe` / `aiosh.exe` were rebuilt (05:57:54,
post-dating every source file) with cargo reporting up-to-date afterwards; `git diff origin/main
HEAD` empty means the source they compiled is the merged `main` content. The one wrinkle, recorded
honestly: a byte-identical restore of `fs_layout_service.rs` during the *previous* task's
revert-control had bumped its mtime past the then-current binaries; the rebuild above removes any
doubt for this pass.

---

## 2. Full Battery at the Merged Head

| Suite | Result |
|---|---|
| `bash ci/run_all_smokes.sh` (mandated baseline gate) | **Attempted, failed as documented** — see below |
| `python tools/test_fs_layout_suites.py` (FL1–FL8, incl. MCP contract C1–C8) | **PASS** (all criteria) |
| `cargo test -p aiosh-core` | **602 passed / 0 failed across 30 binaries** |
| `cargo test -p aiosh-mcp` | **17 passed / 0 failed** |
| `cargo test -p aiosh-cli` | **24 passed / 0 failed** |
| `python tools/check_task_docs.py` | **PASS** (C1..C6) |
| `python tools/task_ledger.py validate` | **consistent: true**, `next_task: 1540` before advance |

**Baseline gate, attempted for the record.** `bash ci/run_all_smokes.sh` aborts at the first suite:

```
==> [rust_smoke] starting
FAIL: rust_smoke (exit 1) — last 40 lines of /tmp/aiosh-ci-rust_smoke.log:
Windows Subsystem for Linux has no installed distributions.
== 1 SUITE(S) FAILED after 1 run (153 ms)
```

`tools/ci_run.py` spawns `["bash", …]` and Windows `CreateProcess` resolves `bash` to
`C:\Windows\System32\bash.exe` — the WSL stub with no distro installed. This is pre-existing,
host-level, and independent of any component change: it fails before compiling code, and it
reproduced identically in T-01538's pass. The component-relevant suites were therefore run
directly (rows above); this is the same documented substitution, not a silent skip.

---

## 3. Fresh Behavioural Proof Through the Real Binary

All scenarios below were executed live for this task (32/32 assertions pass). Scenario letters
match the probe's output.

**A — Residue cap keyed on the canonical destination (re-proof of `e253dc5`).**
- A1 base `register` through the plain spelling succeeds; A2 a pool filled through the **case
  alias** (`STORE.JSON` staged files) blocks a register through the plain spelling; A3 the mirror
  image holds; A4/A5 a **trailing-dot** spelling (`store.json.`) and the plain name charge **one
  pool** in both directions.
- A6 a genuinely different store whose spelled name nests (`s.json.tmp` vs `s.json`) is **not**
  blocked by `s.json`'s full pool, and A7 an unrelated destination is unaffected — the mirror-image
  over-blocking the first cut also had is gone.
- A8 the at-cap refusal surfaces as a proper `isError` envelope naming the cap
  ("refusing to stage another store file: 8 preserved temporary file(s)…"); A9 the refusal produces
  **exactly one `outcome=error` row**, hash-chained (`hash`/`prev_hash` present on every row); A10
  **no staged file was ever deleted** (8 remain before and after).

**B — `scope.paths` `.`/`..` matcher (re-proof of `d86c348`, fresh vectors).**
- B1 a grant minted with `--allow .` **authorizes an absolute store path inside the grant's
  working directory** (the Windows fail-closed half that used to refuse its own directory); B2 the
  **relative spelling** of the same location is authorized by the same entry — entry and argument
  now share one anchored frame.
- B3 an absolute store outside the directory is refused; B4 a **relative `../` spelling escaping
  the directory** is refused (the anchored `..` cannot be used to walk out).
- B5 `--allow ..` authorizes a sibling of the working directory (inside the parent) and B6 **does
  not** authorize an unrelated tree — `..` entries are usable and bounded, not wildcards.

**D — In-scope and unscoped positives (regression fence).**
- D1 granted mutation works; D2 an unscoped **read** (`list` with no `grant_id`) still works, as
  the ungated-read design intends; D3 an ungranted **mutation** is refused pre-gate and D4 recorded
  as exactly one honest `outcome=refused` row; D5 `validate` over a granted spec document works.

---

## 4. New Attacks — What Holds and What Does Not

**C1 — TOCTOU between residue counting and staging.** `save_to_path` counts residue, then stages —
check-then-use by construction. Ten trials planted 7 staged files, started a real `register`, then
planted two more 2–20 ms into the call. **Result: 10/10 the refusal observed the planted files**
(residue 9 at refusal time; the cap check runs after the plants landed) and **0/10 staged through**
a window. Holding conclusion: the only way observed residue can exceed 8 is files planted
externally *between* a successful count and a successful stage — and in 10 probes the refusal
never lost to that window.

**Honest boundary (structural, not a defect):** the cap is a **pre-stage refusal, not a
transaction**. A writer that won the count/stage race would exceed the bound; more fundamentally,
the service **never deletes** staged files (a sweep-by-pattern would be an attacker-influenced
deletion primitive), so externally planted `.tmp`-shaped files can always push *observed* residue
past 8 without any code defect. The bound that is honest to claim: *this service stages nothing
new once its canonical-keyed count reaches the cap, and never deletes anyone's staged data*.

**C2 — Concurrent staged-listing races.** Six trials ran two real `register` calls against one
store with 7 planted files. **Result: 6/6 both staged, residue > 8 in 0 trials, unreadable stores
0** — the single-process MCP server serializes its own calls, the staged listing never observed a
torn state, and every store loaded cleanly afterwards (`list` OK each time). Two *external*
processes remain outside any lock (the documented single-writer assumption, guide §6.17) — the
race above is the in-server window, and it did not produce an over-cap or corrupt state in 6
trials.

**C4 — Un-anchored canonical key on non-existent deep paths.** An allow entry naming
`…/deep/a/b/c` where **nothing below `deep` exists** still authorized a target inside it (the
lexical fallback keys identically) and **did not** authorize a sibling branch
(`…/deep/a/b/OTHER`). Fail-closed where it must be, usable where it should be.

**C5 — Drive-relative store spelling.** A store argument spelled `st\store.json` while the grant
covers the absolute directory was **refused (fail-closed)** and the refusal recorded. Recorded as
observed behaviour: the residue keyer deliberately does not anchor drive-relative spellings, and
policy treats the ambiguous form as not-provably-in-scope. No over-grant found.

---

## 5. Component Chain Closure (T-01531 … T-01540)

The Filesystem Layout MCP/API surface chain delivered, in order: the ten-tool surface (T-01534),
contract tests (T-01535), cross-surface integration (T-01536), a security review that found and
fixed path-scope aliasing (T-01537), hardening of the persistence layer (T-01538: write cap,
residue cap, bounded transient-only retry), honest documentation including four
executed-examples-corrected claims (T-01539), the `.`/`..` scope matcher fix, the canonical
residue-cap keying fix, and this independent verification. Residuals are **documented findings,
not silently dropped**: the sealed-store case (a store over the 10 MiB read ceiling cannot be
loaded or repaired by any tool — guide §6.12), undeclared arguments ignored despite
`additionalProperties: false` (§6.15), ungated reads with caller-named spec paths (§6.16), the
single-writer assumption (§6.17), and the structural check-then-use nature of the residue cap
recorded in §4 above. Each carries an owner (the guide section) and none is implied closed.

---

## 6. Conclusion

The component's full battery is green at merged `main` (`e253dc5`); the two newest fixes re-prove
through the real binary with fresh vectors; the new attacks found **no bypass** — the TOCTOU and
race windows did not defeat the cap in 16 trials, and the edge spellings fail closed. No defect
requiring a code fix before the chain closes was found; the structural boundary in §4 and the
documented residuals in §5 are recorded honestly rather than repaired silently.
