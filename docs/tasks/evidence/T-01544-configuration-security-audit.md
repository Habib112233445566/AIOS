# T-01544 — Filesystem Layout / configuration: Security Audit of the Pending Diff

**Date:** 2026-09-19
**Scope:** the **uncommitted** working tree that this delivery would commit — the T-01543 change set
(`fs_layout.rs`, CLI/MCP strings, tests, docs) plus the T-01544 store-parse change
(`fs_layout_service.rs`, tests, docs). `HEAD` is `65a6f1b`; nothing in this audit was verified
against a later commit.
**Surfaces:** `aiosh-core::fs_layout` + `fs_layout_service`, the CLI `aiosh layout …` surface, the
ten `aios.fs_layout.*` MCP tools, the audit ring's write and **render** paths, and the on-disk
store document format.
**Environment:** Windows host; every probe ran a **freshly built** `target/debug/aiosh{,-mcp}.exe`
against a throwaway `AIOSH_HOME` and throwaway stores under a temp directory, so no real state was
touched. No `sudo`, no network, nothing outside those temp directories.
**Status:** delivered. Findings: **0 blocking, 2 Low, 2 Info**. The diff is accepted for commit with
the recommendations below recorded; none of them is a policy bypass.

---

## 1. What the diff changes, security-wise

1. The spec document's parse contract was tightened in T-01543: unknown fields refused at every
   entry (`from_json`, `serde_json::from_value` for inline `layout`, store load), plus seven new
   validation rules (E-1..E-7) and the FL6 invariant.
2. T-01544 extends the same refusal to the store **document** — previously the one parse path that
   silently ignored a field it did not know.
3. Two new **echo sites** therefore carry attacker-chosen bytes into human-readable output and into
   the audit ring: the *unknown field name* is named in the refusal message.

Points 1 and 2 reduce attack surface (a document that means something else is refused instead of
applied). Point 3 is what the new findings below are about.

## 2. Coverage — what was and was not audited

**Performed, with live reproduction against the real binaries**

| Check | Method | Result |
|---|---|---|
| Misparse protection at every entry | crafted documents through CLI `--spec`, MCP `spec`, MCP inline `layout`, and store load | refused, offender named (§3.1) |
| Refusal is fail-closed | exit codes, store md5 before/after, store contents, staged-file listing | never partially applied (§3.2) |
| Audit invariants on the new refusal paths | row count vs. invocation count, per-row `tool/command/outcome/outcome_detail` | exactly one row per call, honest outcome (§3.3) |
| Injection into terminal via the new echo sites | ESC, C1 CSI (`U+009B`), BEL, bidi override (`U+202E`), zero-width (`U+200B`) in field names and values | Cc neutralized, **Cf passes** → F-2 |
| Ring render path | `aiosh audit tail` over a ring whose row carries raw ESC | no raw control byte in the tool's output (§3.4) |
| Availability / lockout of the new rule | all `fs_layout` verbs against an unknown-field store | refused everywhere → F-1 |
| Regression of the enum-level attribute | `{"custom": …}` for `FsType` and `PartitionType`, built-ins, `import-fstab` incl. its E-6 case | no regression (§3.5) |
| Documentation accuracy | tree-wide sweep for stale `FL1..FL5` claims outside frozen evidence | one stale comment, fixed (§F-4) |

**Not performed — declared, not silently skipped**

* **Linux-only runtime paths** (Landlock/seccomp, POSIX rename semantics): host is Windows. The
  diff is pure deserialization policy and touches none of them, but that is a source-level
  conclusion, not an executed one.
* **Concurrency of the audit ring** (forked chain, lost row after a mutation, panic on a busy
  ring — F-02/F-06/F-16 of the 2026-09-18 whole-repo audit): cross-cutting `dispatch`/`audit`
  defects shared by ~130 tools, **unchanged and still open elsewhere**; this audit confirms only
  the single-writer case (§3.4). Not fixed here, not claimed fixed.
* **The `store_path` authorization model** (F-01/F-03 of the same report): other components'
  surface; this diff does not widen it (it can only turn *more* store files into refusals).
* **Advisory/CVE matching, SAST tooling, fuzzing, secret scanning:** unavailable in this
  environment, exactly as the 2026-09-18 report recorded. No new dependency was added by this
  diff, so the inventory is unchanged.
* **Adversarial LLM session** against `aiosh agent` using these messages as an injection carrier:
  not run. The classifier's static rules are the only prompt-injection control examined.

---

## 3. What held up under probe (the acceptance evidence)

### 3.1 Misparse protection is complete and names the offender

* CLI `--spec` with a stray `dry_run`: `SPEC_PARSE_FAILED`, exit 1, message names the field and
  lists the schema.
* MCP inline `layout` with a stray field: `isError:true`, body
  `{"ok":false,"error":"invalid layout JSON: unknown field `pwn\u001b[31m`, expected one of `id`, …"}`
  (JSON-escaped — see F-3).
* MCP `spec` string: `failed to deserialize layout JSON: unknown field …`.
* Store document, unknown **top-level** field: `LOAD_STORE_FAILED` (CLI) /
  `isError:true` (MCP), `failed to deserialize layout store from '<path>': unknown field
  `active_layout`, expected `active_layout_id` or `layouts`` — i.e. the typo that used to silently
  keep the default active layout is now the *first* thing reported.
* Store document, unknown **nested** field inside a stored layout: same refusal, naming the nested
  offender.

### 3.2 Refusal is fail-closed and side-effect free

* Refused registration: store file **md5 unchanged**, exit 1, **no staged `.tmp.` residue** created,
  the refused layout id absent from the store afterwards.
* MCP refusal: the store's layout keys were byte-for-byte the pre-call set (`mcp-probe-bad` never
  appeared) while the positive control in the same run did register.
* Live-then-refused pairs across CLI `list`/`register`/`set-active` and MCP
  `register`/`set_active` all returned the documented codes and wrote nothing.

### 3.3 Audit invariants hold on every new refusal path

Row count equals invocation count with no double-logging, in three independent probe rings:

| Ring | Invocations | Rows | Attribution |
|---|---|---|---|
| CLI battery | 11 | 11 | each row `tool=fs_layout`, `command` = the verb, `outcome=failure` for refusals, `outcome_detail` = the failing stage |
| CLI battery (sanitization run) | 7 | 7 | idem (`Failed to parse layout spec JSON`, `Failed to import fstab as layout`, …) |
| MCP battery | 6 (incl. the `grant create` and a positive control) | 6 | each `aios.fs_layout.*` row carries `outcome=ok/error` matching the envelope, and the `audit_id` in the response equals the row id |

The envelope and the row never disagreed in any probe, and no refusal was swallowed. This is the
invariant the ledger's Implementation instructions single out, and it survives the change.

### 3.4 Ring integrity and rendering

* `aiosh audit verify` over a ring containing the new refusal rows: `ok: true, checked: 3,
  broken_at: null` (single writer; concurrency out of scope, §2).
* `aiosh audit tail` renders JSON-escaped text: a row whose `outcome_detail` contains a **raw ESC
  byte** (see F-3) produced **no raw control character** in the tool's stdout or stderr — the
  escape survives only as the six characters `\u001b`.
* Human diagnostics neutralize Cc controls with U+FFFD: an ESC in an unknown field name renders as
  `pwn<U+FFFD>[31m`; the C1 CSI is likewise replaced (`U+009B` → `U+FFFD`), so the single-byte CSI
  introducer cannot reach a terminal either.

### 3.5 Compatibility (additive-only) confirmed live

* Both built-ins: `VALID … FL1..FL6`, exit 0.
* `{"custom": "zfs"}` as `fs_type` and a custom partition-type GUID both register — the enum-level
  `deny_unknown_fields` does not break the documented `Custom` payloads.
* A store written by the tool still loads, and a save that changes no state reproduces **identical
  bytes** (md5 compared); a real state change does alter them.
* `import-fstab` unaffected; an fstab row with `dump=2` is refused with E-6 through the import path.

---

## 4. Findings

### F-1 — An unknown-field store is unloadable and not repairable in-tool (Low, residual by design)

**What:** after T-01544, a store file carrying any key the schema does not declare is refused by
**every** `fs_layout` verb, and because each verb loads before it writes, **no tool can repair it**.
Reproduced: `layout list`, `register`, `set-active` each refused against the same crafted store.
**Why it is Low and not a vulnerability:** fail-closed (nothing is applied, nothing is lost), and
reachable only from a store this tool did not write — the write path emits exactly the two known
keys, verified byte-identical. It is the exact analogue of the sealed-store residual (guide §6.12)
and is now documented as **§6.22**, including the external recovery step.
**Accepted risk:** the same choice makes a *newer* version's store unreadable by an older binary.
That is deliberate (loud refusal beats silent misreading — the D1 rationale) and is stated in
§6.22 rather than implied.
**Recommendation:** none required now. If mixed-version fleets become real, the honest fix is a
documented migration step, not a tolerance switch — and a store *schema version key* would itself
be an unknown field to older readers, so it is not a fix by itself.

### F-2 — Human-path sanitizer covers Unicode Cc but not Cf; the new echo sites carry a field name into it (Low)

**What:** `sanitize_terminal` (CLI, T-01527) replaces characters for which `char::is_control()` is
true — Unicode general category **Cc**: C0, DEL, and **C1** (the C1 case is confirmed:
`U+009B` → `U+FFFD`). It does **not** touch category **Cf**, and the new refusals echo an
attacker-chosen *field name* into that renderer. Reproduced codepoint-exact through the fresh
binary:

| Payload in the unknown field name | Rendered in human stderr |
|---|---|
| `pwn\x1b[31m` | `pwn<U+FFFD>[31m` |
| `pwn\u009b31m` (single-char CSI) | `pwn<U+FFFD>31m` |
| `a\u202eb` (bidi override) | `a\u202eb` — **passed through verbatim** |
| `a\u200bb` (zero-width space) | `a\u200bb` — **passed through verbatim** |

Same result through the T-01544 store-refusal path (`a\u202eb` preserved, rc=1).
**Impact:** no escape *execution* (no ESC/C1/BEL survives, so a terminal cannot be driven), but the
operator-facing error text can be **visually reordered or concealed** by a hostile document — the
message can display a misleading fragment. It also composes with the pre-existing behavior that
any echoed untrusted text (a layout id in `layout with id '…' not found`, an argv echo) can shape a
message an operator or script keys on; scripts should key on exit codes and `error.code`, which
this diff leaves intact.
**Attribution:** the renderer's coverage boundary is pre-existing (T-01527); T-01543 added one echo
site and T-01544 a second, which is what makes it worth recording now.
**Recommendation (for a later hardening task, e.g. this sub-epic's Hardening slot):** treat the
echoed *identifier* — the field name — as untrusted and render it either through an allow-list
(`[A-Za-z0-9_-]`, else escape) or extend the sanitizer from Cc to Cc ∪ Cf. Do not weaken it into
"strip everything non-ASCII": the messages are observability surface and must stay faithful enough
to debug against.

### F-3 — Refusal text with attacker bytes is persisted verbatim in the audit ring (Info)

**What:** the MCP surface writes the whole refusal message into `outcome_detail`, so a field name
containing a raw ESC byte is stored as a raw ESC byte in the tamper-evident ring (reproduced by
querying `audit_ring`). The CLI's store path stores only the **stage** string
(`Failed to load layout store`), so it is unaffected.
**Why it is Info:** fidelity in the ring is by design and the tool's own renderers escape it (§3.4):
`aiosh audit tail` emitted no raw control byte. This is the known "no redaction on audit payloads"
residual of the 2026-09-18 report (F-10), now reachable from one more source; the only way to turn
it into terminal injection is to read the DB with something other than the tool and print the
column raw, which is outside the tool's control.
**Recommendation:** none now. If audit-payload redaction/encoding is ever taken up, this row class
should be covered by the same change (and the record must not be *silently* altered — the ring's
value is that it is faithful).

### F-4 — Stale `FL1..FL5` claim in the CLI smoke suite (Info, **fixed in this pass**)

A tree-wide sweep for `FL1..FL5` outside frozen evidence found one remaining occurrence:
`code/aiosh-cli/tests/test_fs_layout_cli_smoke.py:153` (`# Valid: shipped preset satisfies
FL1..FL5.`). Comment only, no assertion depended on it. Corrected to `FL1..FL6` in this pass; no
test behavior changed (FL1–FL8 still PASS).

---

## 5. Residuals and cross-references (unchanged by this diff, restated so nothing is implied closed)

* Ring-level concurrency defects (forked chain / lost row after mutation / panic on a busy ring) —
  open elsewhere, §6.18 of the guide; not this component's surface and not fixed here.
* `store_path` remains a caller-controlled path that is not itself a `scope.paths` subject — the
  global audit's F-01/F-03; the T-01542 D5 contract deliberately keeps store location explicit, and
  this diff neither widens nor narrows it.
* FL6 is a *floor* (at least one `required` mount), not a readiness proof — no boot-verification
  hook exists in this component (spec D8). A caller can satisfy FL6 with a junk `required: true`;
  that is the specified semantics, not a bypass of it.
* Sealed-store recovery (guide §6.12) and the new §6.22 lockout both require external recovery
  steps. No repair verb was invented here.

## 6. Verdict

**Accepted for commit.** The change strictly reduces what a malformed document can do (no silent
misreading anywhere on the parse path), all new refusals are fail-closed, audited exactly once,
and leave no residue; compatibility is proven additive against built-ins, custom enum payloads,
`import-fstab` and byte-identical store round-trips. The two Low findings are a documented
availability trade and a pre-existing renderer boundary that the diff newly exercises with an
attacker-controlled identifier; both have concrete recommendations and neither is a policy bypass.

**Reproducing:** the probes are short `python` + `subprocess` harnesses against
`target/debug/aiosh.exe` / `aiosh-mcp.exe` with `AIOSH_HOME` and `cwd` inside a temp directory
(same shape as the 2026-09-18 report's appendix: one JSON-RPC `tools/call` per process, grants
minted by the real CLI, stores and crafted documents under that temp directory). Probe scripts were
not committed — the durable forms of the checks are the two Rust tests added by T-01544 and the
FL1–FL8 suites, which run these paths through the real binaries in CI.
