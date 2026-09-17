# T-01538: Filesystem Layout — MCP/API Surface: Hardening

## Metadata
- **Task ID:** `T-01538`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout MCP/API surface (`aios.fs_layout.*`), the persistence layer all ten
  tools share (`aiosh_core::fs_layout_service`), and the gate/envelope/audit path
  (`aiosh_core::dispatch`)
- **Status:** Complete — three defects found and fixed on the surface's persistence layer; the other
  hardening requirements verified (with evidence) rather than assumed. Residuals carried forward are
  named in §5 with owners, not implied closed.
- **Date:** 2026-09-18
- **Milestone:** Sub-Epic: Filesystem Layout (8/10) — MCP/API Surface Hardening
- **Dependencies:** `T-01537` (MCP/API Surface Security Review)
- **Next Task:** `T-01539` (Filesystem Layout / MCP/API surface: Documentation)
- **Code changed:** `code/aiosh-rust/aiosh-core/src/fs_layout_service.rs` (+195/−10),
  `code/aiosh-rust/aiosh-core/tests/test_fs_layout_service.rs` (+162)
- **Not committed:** the tree stays on `feat/user-session-bootstrap`; committing is the delivery
  step, not this task.

---

## 1. Scope & Method

**Surface under hardening:** the ten `aios.fs_layout.*` tools and the shared persistence layer behind
them. This task owns the four hardening duties the ledger states for every component's MCP/API
surface, plus the two residuals `T-01537` explicitly assigned to it (F-5's unbounded staged-file
retention, and the temp-file/resource-cleanup mandate).

**What is actually reachable here.** The fs_layout surface spawns **no external processes** — verified
by searching the whole crate for process-spawning APIs (`Command::new`, `.spawn()`, `.output()`,
`shell=True`, `os.system`): none appear on any fs_layout path, and the only `unsafe` in the crate is
confined to syscall wrappers in `sandbox.rs`/`secrets.rs`. The files it touches are the store JSON it
writes (`store_path`), the layout/fstab documents it reads (`spec`/`fstab`/`layout`), and its own
staged temp files. That is why "add timeouts" resolves to *bounded reads and no unbounded retrying*
rather than to a subprocess timeout, and why the defects worth finding were on the **write** path.

**Method — probe, don't re-read.** Every behavioural claim below was exercised through the **real
`aiosh-mcp` binary over stdio** (JSON-RPC 2.0), with the real `aiosh` CLI minting the PEP grant and
reading the audit ring back (both share one DB via `AIOSH_HOME`). Probes ran under a throwaway
`AIOSH_HOME`, a throwaway store directory under the OS temp dir, and a grant scoped to that temp
directory, so no repository file and no operator state was touched.

**One probe accident worth recording**, because it is the cleanest reproduction in this document: my
first over-ceiling run used an `aiosh-mcp.exe` that had not been rebuilt since the fix, so it
exercised the *pre-fix* binary and reproduced the defect end-to-end (§3 H-1, "Pre-fix" column). The
same script then ran against the rebuilt binary and shows the fix. Nothing else in this document
claims a pre-fix behaviour that was not observed that way.

**Environment note.** `bash ci/run_all_smokes.sh` cannot run on this host: it aborts at the first
suite (`rust_smoke`, 0/29 suites) because `tools/ci_run.py` spawns `["bash", …]` and Windows
`CreateProcess` resolves `bash` to `C:\Windows\System32\bash.exe` — the WSL stub, with no distro
installed:

```
==> [rust_smoke] starting
FAIL: rust_smoke (exit 1) — last 40 lines of /tmp/aiosh-ci-rust_smoke.log:
Windows Subsystem for Linux has no installed distributions.
== 1 SUITE(S) FAILED after 1 run (107 ms)
ci-check: FAIL (0/29 suites, 1 failed)
```

This is pre-existing and independent of this change: it fails before compiling any code this task
touched, and it reproduces identically with the change reverted. The fs_layout-relevant suites were
therefore run directly (§4), and they are green.

---

## 2. Hardening Inventory

| # | Requirement | Verdict | Evidence |
|---|---|---|---|
| H1 | Size caps where files are involved — **read** side | already bounded (`T-01531`/`T-01534`) | doc read capped at 10 MiB *during* the read and refused by file type; inline payloads capped at 1 MiB; `store_path` ≤ 1024 chars; transport line capped at 1 MiB; layout counts 128 partitions / 128 mounts / 1024 directories |
| H2 | Size caps — **write** side | **DEFECT → fixed (§3 H-1)** | store could be written past the reader's ceiling and became permanently unreadable, while the call reported `ok=True` |
| H3 | Bounded retries | **DEFECT → fixed (§3 H-4)** | temp *creation* retried 8×; the final atomic *replace* was single-attempt, so one transient lock produced a permanent failure plus stranded residue |
| H4 | Bounded temp-file retention | **DEFECT → fixed (§3 H-2)** | `T-01537` F-5 left one preserved staged file per failed call, unbounded; now capped at `MAX_STAGED_KEEP` checked before staging |
| H5 | Cleanup on the error path is reported, never silent | **DEFECT → fixed (§3 H-3)** | a failed unlink of the partial staged file was discarded with `let _ =` |
| H6 | Errors in the standard result envelope | verified | every arm — read and mutating — returns through `dispatch::recorded_call*`; refusals observed as JSON-RPC `isError` with the reason text, never an empty read (§4) |
| H7 | Resource cleanup: DB connections | verified, nothing to change | this service opens **no** database connection (no `rusqlite`/sqlite use in `fs_layout_service`), so there is no connection to leak; the audit ring is opened and closed per call by `audit.rs`, not here |
| H8 | Resource cleanup: child processes | verified, nothing to change | no process is spawned on this surface (see §1) |
| H9 | Resource cleanup: temp files | verified + hardened | a successful mutation leaves no staged file; a failed one preserves exactly one (deliberate, documented) and now cannot accumulate past the cap (§4 C/D) |
| H10 | Fail-open writes an honest audit row (ADR-0035 §F-2) | verified for this surface | each fs_layout call writes exactly one row in every outcome, including the two new refusals, with `outcome=error` and the refusal text in `outcome_detail`; `aiosh audit verify` stayed `ok` (§4 A/C/E) |

**What was *not* done, deliberately.** No external timeouts were added, because there is nothing to
time out (H7/H8 above): inventing a watchdog around a `rename` syscall would be theatre — a
kernel-blocking `rename`/`sync_all` is not interruptible from the calling thread, and the honest
statement is the one written into the constant's doc comment ("no unbounded retrying", not "no
unbounded syscall"). No subprocess ceilings were added for the same reason.

---

## 3. Findings

### H-1 — HIGH: the write side had no ceiling, so a store could be saved into permanent unreadability

**Root cause.** `MAX_LAYOUT_DOC_BYTES` (10 MiB) is enforced by `read_bounded_text_file` on **every**
read — `load_from_path` refuses a larger store forever. Nothing bounded the **write** side, and layout
registration is not capped, so `save_to_path` would happily commit a store larger than the reader's
own ceiling. The call reported success; the next load failed with a size error. The store's data was
not lost so much as **sealed**: nothing can load it to shrink it again.

**Pre-fix (reproduced over the real MCP surface, pre-rebuild binary).** Accumulating ~1 MiB layouts
through `aios.fs_layout.register`:

```
  register#10 ok=True  store_on_disk=  10025616
  register#11 ok=True  store_on_disk=  11027499 OVER-CEILING     <-- success reported
  register#12 ok=False store_on_disk=  11027499 OVER-CEILING  err=layout store file '…\fs_layouts.json'
                                                                  exceeds 10 MiB security size limit
```

`#11` is the defect: `ok=True` while writing 11,027,499 bytes past a 10 MiB read ceiling — and the
store never loads again, so the mutation cannot even be undone through the surface.

**Post-fix (same script, rebuilt binary).** The refusal now lands on the write, before anything is
staged, and the store stays readable at its previous size:

```
  register#10 -> ok      store_on_disk=10025616
  register#11 -> REFUSED isError=True store_on_disk=10025616 (<= ceiling: True)
    error: refusing to save filesystem layout store '…\fs_layouts.json': serialized store is 11027499
           bytes, which exceeds the 10 MiB read ceiling, so the result could never be loaded again —
           remove or split registered layouts before retrying (the existing store on disk is unchanged)
     audit row: tool=aios.fs_layout.register outcome=error target=bulk-11
```

**Design note (why the refusal is checked on the serialized bytes, before staging).** The check runs
after serialization and before parent-directory creation, staging, or the parent-directory guard, so a
refusal cannot itself leave residue (verified: `staged files after refusal: []`). It also means the
refusal is a *state* error, not an I/O error: the destination is untouched and the new layout is
absent while the previous one remains (§4 B).

### H-2 — MEDIUM: failed replacements could accrete staged files without limit (closes T-01537 F-5)

**Root cause.** A failed replace deliberately preserves its staged file — it may be the only complete
copy of the state the caller asked to persist — and `T-01537` F-5 recorded that this is reachable (a
read-only or locked destination fails *after* a successful stage) and that "bounding the retention …
belongs to `T-01538`".

**Fix.** `MAX_STAGED_KEEP` (8) refuses to stage another file once that many preserved staged files of
*this* destination already sit beside it. Three deliberate properties:

- checked **before** staging, so the refusal adds no residue of its own;
- prefix-scoped to `.<store-name>.tmp.`, so two stores sharing one `.aios/` directory cannot trip each
  other's cap (asserted in-tree: an `other-store` staged file is neither counted nor deleted);
- **nothing is ever deleted.** A staged file beside a different writer's store is that writer's data,
  and this code cannot tell the writers apart. The cap bounds *new* residue and names the oldest file
  so the operator can act; it does not garbage-collect.

A directory listing failure is reported rather than treated as "no residue" — a guard that silently
reported zero would be the same class of dishonesty the cap exists to prevent.

**Live proof over MCP** (`set_active`, 8 preserved staged files present):

```
  pre-existing staged residue: 8 (cap 8)
  set_active -> ok=False isError=True
    error: refusing to stage another store file: 8 preserved temporary file(s) already sit beside
           '…\store\fs_layouts.json' (cap 8); resolve the failed replacements named by the earlier
           errors (for example '…\.fs_layouts.json.tmp.…') before retrying
  staged after refusal: 8  (added none: True)     store still readable: True
  audit row: tool=aios.fs_layout.set_active outcome=error
```

And with the residue removed, normal operation resumes — the cap is a bound, not a lockout (asserted
in-tree as well).

### H-3 — MEDIUM: cleanup failure on the write path was silent

`save_to_path`'s partial-write branch did `let _ = fs::remove_file(&tmp_path);` and reported only the
write error. If that unlink failed, a **truncated** staged store was left on disk and the caller was
never told — the caller sees "write failed" and would reasonably assume no artefact remains, then
finds a file that *looks* like a store and is not one. The error now carries both failures and names
the file to delete by hand. Only one branch: the branch that matters (the one that returns `Err`
anyway) — the success path still consumes its staged file through the rename, and the failure path
still preserves it.

### H-4 — MEDIUM: a single transient rename failure became a permanent, residue-producing failure

**Root cause.** The final `fs::rename` was attempted once. On a live system a replace fails
*transiently* — on Windows a scanner/indexer/backup agent or a plain reader holding the destination
open surfaces as `ERROR_ACCESS_DENIED` (os error 5, which Rust reports as `PermissionDenied`), which
this host reproduces; POSIX has `EBUSY`/`ETXTBSY`/`EINTR`. One attempt converts a momentary condition
into a hard failure that also strands a staged file, turning cosmetic contention into operator work.

**Fix.** A retry loop bounded on **both** a count (`MAX_REPLACE_ATTEMPTS = 5`) and a wall-clock budget
(`REPLACE_RETRY_BUDGET_MS = 5_000`), with a doubling backoff from 20 ms, retrying only errors a retry
can plausibly clear (`PermissionDenied`/`WouldBlock` and raw os errors 5/32/33 on Windows, 16/26/11/4
on POSIX — the legacy `Access is denied. (os error 5)` above is exactly why 5 is in that set).
Permanent failures (missing parent, destination is a directory, read-only volume) still fail on the
first attempt. Retries reuse the one staged file, so they cannot multiply residue, and the error
message now reports the attempt count and still names the preserved file.

The bounds are exported (`pub const`) so the tests assert against the real constants instead of magic
numbers, and the doc comment states the limit of the claim: this bounds **our retry loop**, not a
single syscall blocked inside the kernel.

---

## 4. Verified Evidence

### A. Write/read cap symmetry and the audit row (`aios.fs_layout.register`, real binary over stdio)

```
read ceiling = 10485760 bytes
  register#01..#10 -> ok      store_on_disk 1008693 .. 10025616
  register#11 -> REFUSED isError=True store_on_disk=10025616 (<= ceiling: True)
  audit rows added by the refused call: 1
    tool=aios.fs_layout.register outcome=error target=bulk-11
    outcome_detail=refusing to save filesystem layout store '…': serialized store is 11027499 bytes,
                   which exceeds the 10 MiB read ceiling, …
  staged files after refusal: []
```

### B. The refused write changed nothing that matters

```
  loads normally: 12 layouts; bulk-11 present: False; bulk-10 present: True
```

### C. Residue cap through the MCP surface — see §3 H-2 for the transcript
(`ok=False`, `isError=True`, one `outcome=error` row, **no** residue added, store still readable).

### D. Success path leaves nothing behind

```
  seed register ok: True   staged: []
  set_active ok=True       staged left: []
```

### E. Audit chain

```
  aiosh audit verify --json  ->  ok: True   data.ok: True
```

### F. Test suites (all green, at this head)

```
$ python tools/test_fs_layout_suites.py
[+] FL1..FL8
PASS: fs_layout_suites criteria (FL1..FL8)

$ python code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py
PASS: C1 advertised inputSchema == accepted arguments for all 10 tools
PASS: C2 audit target for both register input forms, success and body refusal
PASS: C3 destructive_transition true on shrink, false on grow
PASS: C4 negative cases (ungranted, missing/oversize store_path, unknown tool)
PASS: C5 grant scope.paths confinement + in-scope/unscoped positive controls
PASS: C6 prompt-injection refused nested or flat
PASS: C7 scope.paths canonical matching (case, 8.3, trailing dot/space)
PASS: C8 device/extended-length spellings refused

$ python code/aiosh-cli/tests/test_fs_layout_hardening.py
ALL FILESYSTEM LAYOUT CLI HARDENING PROOFS PASSED!   (H-2/H-4b/H-5 SKIP: need POSIX)

$ python code/aiosh-cli/tests/test_fs_layout_audit_security.py
ALL FILESYSTEM LAYOUT AUDIT & INJECTION SECURITY PROOFS PASSED!

$ cargo test -p aiosh-mcp     -> test result: ok. 17 passed; 0 failed
$ cargo test -p aiosh-core    -> 30 test binaries, all ok; 591 tests passed; 0 failed
$ cargo test -p aiosh-cli     -> test result: ok. 24 passed; 0 failed
$ python code/aiosh-mcp/tests/test_fs_layout_mcp_smoke.py   (via FL5/FL6 in the suite runner)
```

### G. New regression coverage

`code/aiosh-rust/aiosh-core/tests/test_fs_layout_service.rs` (14 → 17 tests):

- `test_fs_layout_save_refuses_store_larger_than_reader_ceiling` — builds a store that exceeds the
  ceiling *and still validates* (a `DirectorySpec::description` is unbounded free text, so one
  registered layout is enough), asserts the refusal names the read ceiling and the unchanged store,
  asserts the pre-existing store is byte-identical and still loadable, and asserts no staged file was
  created. This is the pin for H-1.
- `test_fs_layout_save_refuses_to_stage_past_the_residue_cap` — pins H-2, including that a *different*
  store's staged file is neither counted nor deleted, and that removing the residue restores operation.
- `test_fs_layout_replace_retry_is_bounded_and_reported` — pins H-4's *contract* rather than a
  platform-specific count: ≥1 and ≤`MAX_REPLACE_ATTEMPTS` attempts parsed out of the message, the
  preserved staged file named, exactly one staged file left (retries do not multiply residue), and the
  whole envelope inside its stated budget. Platform behaviour legitimately differs (POSIX reports
  `EISDIR`, Windows reports transient `os error 5`), which is why the bound is what is asserted.

**Negative control (H-1/H-2/H-4 are load-bearing, not decorative).** Reverting
`fs_layout_service.rs` to `HEAD` while keeping the new tests fails exactly the three new tests and
leaves the pre-existing 14 green:

```
test test_fs_layout_replace_retry_is_bounded_and_reported ... FAILED
test test_fs_layout_save_refuses_to_stage_past_the_residue_cap ... FAILED
test test_fs_layout_save_refuses_store_larger_than_reader_ceiling ... FAILED
test result: FAILED. 14 passed; 3 failed
  (H-4 failure: "error must report its attempt count: … Access is denied. (os error 5)
                 (the fully-written new store was preserved at …)"  <- single attempt, no retry)
```

The reverted file was then restored and compared byte-for-byte (`cmp`) against the hardened copy: no
control marker remains in the tree (`git diff` reviewed).

---

## 5. Residual Limitations (stated, not omitted)

1. **Read-tool gating (`T-01537` F-4) is unchanged and is a spec-level decision.** `get`/`validate`/
   `fstab`/`list`/`probe`/`diff` still run ungated and still accept a caller-named `spec` path. The
   read is bounded and type-checked, and egress is limited to documents that parse as a layout — but a
   grant cannot currently constrain it, because an ungated call consults no grant. Options (gate the
   reads, or honour `spec` against a supplied grant's `scope.paths`) are recorded in `T-01537` §3 F-4
   and belong to the spec (`docs/filesystem_layout.md` §8.1) before they belong to code. **Not
   changed here.**
2. **The classifier/CLI-provenance residual (`T-01537` F-2, read side) is unchanged.** A store written
   by the operator CLI can carry hostile text that `get`/`list` echo; rendering neutralises control
   characters but JSON output and the store stay faithful *by design*, and the in-tree security proof
   asserts that faithfulness. Closing it means deciding how stored text is marked on read — a
   behaviour change for read tools, and `T-01539`'s neighbourhood. **Not changed here.**
3. **`DirectorySpec::description` and `DirectorySpec`-adjacent free text have no length bound of their
   own.** This is what let a single validated layout exceed the 10 MiB ceiling in the H-1 reproducer.
   The *consequence* is now closed (H-1 refuses the write), but the field itself is still unbounded —
   bounding it is a data-model change (`fs_layout.rs`, `T-01531…T-01535`'s surface) and would alter
   which documents validate. The cap is the load-bearing invariant: any store the service writes can
   be read back, whatever the field bounds are.
4. **A kernel-blocking `rename`/`sync_all` is still not interruptible.** `REPLACE_RETRY_BUDGET_MS`
   bounds our loop and the backoff, not a syscall that never returns (an unresponsive network mount).
   The server is single-threaded, so that would stall the request loop; no portable fix exists at this
   layer, and inventing a watchdog thread would not make the syscall cancellable. Stated rather than
   papered over.
5. **The audit ring's own durability defects are out of this task's surface and remain open.** The
   forking chain under concurrency, the unbounded `busy_timeout`-less write, and the panic on a busy
   ring (F-02/F-06/F-16 of `docs/SECURITY-AUDIT-2026-09-18.md`) are properties of `dispatch`/`audit`,
   shared by ~130 tools, not of the fs_layout surface. This task verified only that *this* surface
   writes one honest row per call and that the chain verifies afterwards on a single-writer run. They
   must not be read as fixed by this evidence file.
6. **Platform coverage.** This host is Windows. The POSIX-only proofs (`mkfifo` blocking read,
   failed-save preservation on a read-only directory, symlink at the destination) print an explicit
   SKIP in `test_fs_layout_hardening.py` and were not executed; the Windows `os error 5` transient
   class *was* executed (§3 H-4). The POSIX raw-error numbers in `is_transient_replace_error` are
   reasoned from the documented errno values, and the retry behaviour itself is asserted
   platform-independently.
7. **The CI gate is unusable on this host** (§1). The suites this surface is governed by were run
   directly and are green; the 0/29 abort is the WSL `bash.exe` shim, pre-existing and unrelated.

---

## 6. Acceptance

- **"Failure modes produce explicit, auditable errors."** Every refusal observed in §3/§4 is an
  explicit error naming the offending value and the limit it crossed, returned in the standard
  envelope (`isError`), and recorded as exactly one `outcome=error` audit row whose `outcome_detail`
  carries the same reason. The two failure modes that used to be *inexplicit* — an oversized store
  written into unreadability while reporting success (H-1) and a silent cleanup failure (H-3) — no
  longer exist.
- **"No temp/connection leaks on the error path."** A successful mutation leaves no staged file
  (§4 D); a failed one preserves exactly one by design and cannot accrete past `MAX_STAGED_KEEP`
  (§3 H-2); a refused oversized save stages nothing (§4 A); this service opens no DB connection and
  spawns no process (§2 H7/H8).
- **Reconciliation with `T-01537`'s residuals.** Of the two residuals that review assigned to this
  task, **F-5's unbounded staged retention is closed** (H-2, bounded and proven over MCP), and the
  **temp-file cleanup mandate is addressed** (H-2/H-3/H-9, including that the preserved file is now
  bounded and reported rather than accumulating silently). F-2's read side and F-4's read gating are
  **not closed** and are named in §5 with the reason they are spec-level rather than code-level — the
  same disposition `T-01537` gave them, now with an owner and a stated blocker instead of being left
  to implication.
- **Verification:** `FL1..FL8` PASS; MCP contract `C1..C8` PASS; CLI hardening and audit/injection
  proofs PASS; `cargo test -p aiosh-mcp` 17/0, `cargo test -p aiosh-core` 591/0 across 30 test
  binaries, `cargo test -p aiosh-cli` 24/0; three new tests negative-controlled (fail without the fix,
  pass with it, no marker left); `aiosh audit verify` ok. Nothing committed.
