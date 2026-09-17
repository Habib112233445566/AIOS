# T-01539: Filesystem Layout - MCP/API Surface: Documentation

## Metadata
- **Task ID:** `T-01539`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout MCP/API Surface (`aios.fs_layout.*`) and its operator/agent guide
- **Status:** Complete
- **Date:** 2026-09-18
- **Milestone:** Sub-Epic: Filesystem Layout (9/10) — MCP/API Surface Documentation
- **Dependencies:** `T-01538` (MCP/API Surface Hardening)
- **Next Task:** `T-01540` (Filesystem Layout / MCP·API surface: Verification & Evidence)

---

## 1. Documentation Updates Delivered

1. **Operator/agent guide — [`docs/filesystem_layout.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/filesystem_layout.md)**
   - **§5 was the gap.** It documented only the six *read-only* tools (§5.1–§5.6) while the four
     **mutations** that shipped in `T-01534` (`register`, `set_active`, `remove`, `import_fstab`) had
     no reference at all. §5 now covers all **ten** tools:
     - **§5.0 Surface at a Glance** — a ten-row table (effect, grant, `store_path`, audit target) plus
       the four rules that govern the mutating half (PEP grant required twice over; `store_path`
       mandatory; `scope.paths` governs the paths touched; no dry-run).
     - **§5.7–§5.10** — the four mutations, each with its arguments, its refusal shapes and its audit
       target.
     - **§5.11 Result Envelope, Refusals and Audit Rows** — the four envelope shapes (success, body
       refusal, gate refusal, unknown tool) verbatim, and how to join a call to its forensic record.
     - **§5.12 Copy-Pasteable End-to-End MCP Session** — a runnable grant → register → list session.
   - **§6 Constraints & Known Limitations** grew from 11 to **21** entries. New: write-side ceiling and
     the sealed-store consequence (§6.12), staged-residue cap and its known evasion (§6.13), bounded
     replace retry and what it does not bound (§6.14), undeclared arguments are ignored (§6.15), read
     tools are ungated (§6.16), single-writer assumption on MCP (§6.17), audit-ring durability is
     cross-cutting (§6.18), payload ceilings (§6.19), a path must mean one thing to the process that
     runs the tool (§6.20), and the `.`-is-not-an-allow-entry degeneracy (§6.21, discovered here — §5).
   - **§7** gained the **Sub-Epic 4** evidence trail (`T-01531..T-01540`), which was entirely absent —
     §7 previously stopped at Sub-Epic 3. Each entry names what the task contributed, and `T-01540` is
     listed as the still-open task that closes the sub-epic.
2. **Repository index — [`docs/README.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/README.md)**
   - §8.14 updated from "six read-only plus `register`/`set_active`/`remove`" to **ten** tools
     (`import_fstab` was missing), the evidence range now ends at `T-01539`, and the runnable block was
     rewritten so that every line actually executes (§3.3) — including isolating `AIOSH_HOME` so a
     copy-pasted block never writes to the operator's real audit ring.
3. **MCP server README — [`code/aiosh-mcp/README.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-mcp/README.md)**
   - The server README listed **no** `aios.fs_layout.*` tool at all. It now has a *Filesystem Layout
     Tools* section: the ten-tool inventory, the grant + `store_path` requirement, what `scope.paths`
     governs, a runnable stdio example, the documented gate refusal, and pointers to guide §5/§6.

---

## 2. Working Examples (copy-pasteable)

All three shipped examples were executed **verbatim** — extracted from the documents, not retyped —
in throwaway directories with `AIOSH_HOME` pointing at a temporary ring and `PATH` at the in-repo
debug binaries. No example touches the repository or the operator's real state.

### Example A — guide §5.12 (register → list over stdio)

```bash
export AIOSH_HOME="$PWD/.aios-demo"; mkdir -p "$AIOSH_HOME" demo
GRANT=$(aiosh grant create --to agent:fs-layout-demo --tools 'aios.fs_layout.*' --allow demo \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["grant_id"])')
aiosh layout show aios-container-minimal-v1 --json | python3 -c \
  'import json,sys; s=json.load(sys.stdin)["data"]; s["id"]="lab-vm-v1"; s["name"]="Lab VM"; print(json.dumps(s))' \
  > demo/layout.json
# one JSON-RPC object per line on stdin
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"aios.fs_layout.register","arguments":{"spec":"demo/layout.json","store_path":"demo/layouts.json","grant_id":"<GRANT>"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"aios.fs_layout.list","arguments":{"store_path":"demo/layouts.json"}}}
```

### Example B — `code/aiosh-mcp/README.md` (list → register → probe → refused register)

Four requests in one session: a read, a granted mutation, a probe, then the same mutation **without**
`grant_id` to show the gate refusal.

### Example C — index §8.14 (CLI lifecycle, then a granted MCP mutation)

`validate --standard` → derive a spec → `register` → `set-active` → `probe` → `import-fstab`, then a
grant scoped to `./demo` and an MCP `remove` of the just-imported layout.

---

## 3. Executed Proof

### 3.1 Example A, run as written

```
  id 1 isError= False ok= True count= None registered= True
  id 2 isError= False ok= True count= 3    registered= None
```

### 3.2 Example B, run as written

```
  id 1 isError= False ok= True  gate= None reason=
  id 2 isError= False ok= True  gate= None reason=
  id 3 isError= False ok= True  gate= None reason=
  id 4 isError= True  ok= False gate= pep  reason= tool 'aios.fs_layout.register' requires explicit PEP grant
--- documented refusal text check ---
  reason matches doc verbatim: True | tool 'aios.fs_layout.register' requires explicit PEP grant
```

Request 4 is the refusal the README quotes; the string matches the document character for character.

### 3.3 Example C, run as written

```
VALID: Filesystem layout 'aios-uefi-standard-v1' satisfies all FL1..FL5 invariants
SUCCESS: Registered filesystem layout 'lab-vm-v1'
SUCCESS: Active filesystem layout set to 'lab-vm-v1'
Target Disk Probe for 'lab-vm-v1' (Capacity: 200 GiB):
  Status:           VIABLE
  Required Minimum: 1 GiB
  Partition Budget: 0 GiB
SUCCESS: Imported filesystem layout 'lab-vm-v2' with 1 mounts
MCP remove: isError= False ok= True removed= True id= lab-vm-v2
audit verify ok = True
```

### 3.4 Documented claims checked against the running binary

Each row of guide §5/§6 that makes a behavioural assertion was exercised, not assumed:

| Documented claim | Observed |
|---|---|
| Success envelope carries `ok`, `tool`, `audit_id` | `{"ok":true,"tool":"aios.fs_layout.list","audit_id":3,...}` |
| Body refusal: `ok:false` + `error`, `isError:true`, active layout protected | `{"ok":false,"error":"cannot remove active layout 'lab-vm-v1'; switch active layout first"}`, `isError=true` |
| Gate refusal: `gate`+`policy_revision`+`reason`, no store write | `{"ok":false,"gate":"pep","policy_revision":"sprint-2-rule-pack-v1","reason":"tool 'aios.fs_layout.register' requires explicit PEP grant"}`, `isError=true` |
| Unknown tool: bare `ok:false`+`error`, **no row** | `{"ok":false,"error":"unknown tool: aios.fs_layout.nope"}`, `isError=true`; ring table `audit_ring` held **1** row (the following `list`), `audit_id` absent from the refusal |
| Read tools are ungated (§6.16) | `get`/`validate`/`list` succeeded with no `grant_id` |
| §6.15 undeclared arguments are ignored, mutation still happens | `register` with an extra `"dry_run": true` returned `ok:true`, and the layout was present in a later `list` |
| Oversize `store_path` (>1024) refused | `{"ok":false,"gate":"pep","reason":"path subject 'd/…/layouts.json' blocked by grant scope.paths"}` — refused by policy, before the length bound |
| The four mutations are listed in `pep::is_irreversible` (§5.0 rule 1) | `fn is_irreversible` contains all four ids |
| `aiosh audit verify` stays `ok` across the surface's probes | `data.ok = true` |

---

## 4. Corrections Made During Verification

Documentation that asserts something the binary does not do is worse than no documentation, so the
verbatim runs were allowed to overrule the prose. Four claims failed and were corrected:

1. **The index example's grant authorized nothing.** It used `aiosh grant create … --allow .` and only
   *appeared* to work because the call it showed was an ungated read. `--allow .` is accepted by the
   CLI but matches nothing (see §5 below). The example now scopes the grant to a named directory
   (`--allow demo`) with `demo/…` arguments, and the `--allow .` trap is a documented limitation
   (§6.21) rather than a silent foot-gun.
2. **The index example's MCP call was a duplicate registration.** It re-registered `lab-vm-v1`, which
   the CLI part of the same block had already created — so the example refused on a fresh run. It now
   removes the layout the CLI just imported, which is authorized, non-duplicate, and exercises the
   mutation path (`removed=true`, §3.3).
3. **§5.11 claimed "a row is still written" for an unknown tool.** False: `aiosh-mcp`'s dispatch falls
   through to a bare `{"ok": false, "error": "unknown tool: …"}` (main.rs:4262) and never reaches the
   audit ring. Direct read of the ring confirms **1** row for a two-call session (unknown tool +
   `list`), and the refusal carries no `audit_id`. The guide now states that the unknown-tool path is
   the **one** request on this surface that leaves no forensic record — and that the missing
   `audit_id` is how a client recognises it.
4. **§6.18 inherited the same error** ("success, body refusal, gate refusal and unknown-tool alike").
   Corrected to name the three cases that do write a row and to exclude the unknown-tool path. The
   equivalent footnote in `code/aiosh-mcp/README.md` was tightened the same way.

---

## 5. New Limitation Found While Documenting: a `.` Is Not a Usable `--allow` Entry

Writing a *working* example surfaced a real behaviour that neither the guide nor the tests recorded.

**Reproduced on Windows.** A grant created with `--allow .` refuses its own directory, for a relative
*and* an absolute argument:

```
dot-allow + rel store        -> DENIED: path subject 'layouts.json' blocked by grant scope.paths
dot-allow + ABS store        -> DENIED: path subject 'C:\…\t39dot-…\layouts.json' blocked by …
dot-slash-allow + ABS store  -> DENIED  (same)
abs-allow  + ABS store       -> ok
abs-allow  + rel store       -> DENIED: path subject 'layouts.json' blocked by grant scope.paths
```

**Mechanism.** `pep::normalize_path_str` drops `.` components, so the key for `.` is the **empty
string**; `path_allowed` matches only on `key == entry` or `key.starts_with("<entry>/")`. It is
fail-closed on Windows (safe, silently useless). The **mirror** behaviour is source-derived and *not*
reproduced here — no POSIX host in this task — but follows directly from the same expression: on
POSIX the comparison degenerates to `starts_with("/")`, which every absolute key satisfies, so
`--allow .` would authorize the whole filesystem. A second rule falls out of the matrix: the entry and
the argument must be written in the **same frame** (relative entry with relative argument, absolute
with absolute) or the containment test cannot fire.

**Recorded, not fixed.** Fixing this is a change to `pep::normalize_path_str` and therefore to the
`scope.paths` semantics of every tool that declares path subjects — out of scope for a documentation
task. §6.21 states the behaviour, the Windows-observed and POSIX-derived halves are labelled
separately, and the guidance is to use a named directory (`--allow demo`) or an absolute directory
with absolute arguments. Never rely on `.` to confine; never rely on `.` to deny.

---

## 6. Suites and Documentation Invariants

```
$ python3 tools/check_task_docs.py
[+] C1 spec-health   [+] C2 component sections   [+] C3 referenced paths
[+] C4 phase map     [+] C5 index health         [+] C6 no volatile counts
PASS: task docs criteria (C1..C6)                             rc=0

$ python3 tools/test_fs_layout_suites.py
[+] FL1..FL8 … PASS: fs_layout_suites criteria (FL1..FL8)      rc=0

$ python3 code/aiosh-mcp/tests/test_fs_layout_mcp_contract.py
PASS: C1 … PASS: C8   ALL FILESYSTEM LAYOUT MCP CONTRACT CRITERIA PASSED!   rc=0

$ cargo test -p aiosh-mcp      -> 17 passed; 0 failed
$ cargo test -p aiosh-core     -> 591 passed; 0 failed across 30 test binaries
                                  (incl. test_fs_layout_data_model 19/0, test_fs_layout_service 17/0)
```

The doc checker's C3 (referenced paths) and C5 (index links) cover the new prose and links; C1/C2/C4/C6
are unaffected. The new §6.21 cross-references were checked to resolve (§5.0 rule 3, §5.12, §7,
`code/aiosh-mcp/README.md`, `docs/README.md`).

---

## 7. Acceptance Verification

- [x] **Docs updated with working example.** Three examples, each executed verbatim in an isolated
      directory (§3.1–§3.3). §5.12 and index §8.14 were *corrected* by their own runs (§4.1–§4.2), so
      the shipped text is the version that was executed.
- [x] **Limitations are stated, not omitted.** §6 stands at 21 entries; the session's own run added
      §6.21, and two previously-wrong claims about audit rows were corrected rather than left standing
      (§4.3–§4.4).
- [x] **`README`/spec updated with what shipped and how to invoke it.** Guide §5 (all ten tools),
      index §8.14, and `code/aiosh-mcp/README.md` (which previously listed none of them).
- [x] **Constraints and known limitations recorded honestly.** Every behavioural assertion in the new
      prose is backed by an observed result (§3.4); the one consequence that could not be reproduced
      without a POSIX host (§6.21's fail-open half) is labelled source-derived.
- [x] **Task evidence files linked from the doc.** Guide §7 now carries the complete
      `T-01531..T-01540` Sub-Epic 4 trail; `docs/README.md` §8.14 links the range through `T-01539`.

---

## 8. Changes in This Task

| File | Change |
|---|---|
| `docs/filesystem_layout.md` | §5.0 + §5.7–§5.12 added; §6 grown to 21 entries; §7 Sub-Epic 4 trail; corrections per §4 |
| `docs/README.md` | §8.14 tool count, evidence range and runnable block corrected |
| `code/aiosh-mcp/README.md` | *Filesystem Layout Tools* section added; audit-row footnote tightened |

Documentation only — no production code was changed. Changes are **uncommitted** (not requested).
