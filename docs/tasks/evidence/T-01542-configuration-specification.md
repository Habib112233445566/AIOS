# T-01542 — Filesystem Layout configuration: Specification

**Date:** 2026-09-18
**Type:** specification (no code changed in this task)
**Depends on:** T-01541 research (`docs/tasks/evidence/T-01541-configuration-research.md`)
**Feeds:** T-01543 "configuration: Scaffold" — the implementation tasks must realize exactly
what is written here, no more.
**Artifact path note (same ledger quirk as T-01541):** the `instructions` line says
`docs/tasks/evidence/T-01542-spec.md`; the binding `artifacts` entry — the path the evidence
checker validates against — is `docs/tasks/evidence/T-01542-configuration-specification.md`.
This file is written at the **artifacts** path, as T-01541 did.
**Status:** SPECIFIED — D1–D9 resolved below. Every normative rule is marked **[NEW]** or
**[REUSED]**; anything not marked and not quoted is out of scope. All NEW rules are
**AIOS-specific** decisions, not upstream standard requirements; where an upstream source
anchors a rule it is cited.

---

## 0. Method and authority

The contract below is grounded in the T-01541 research facts F1–F11, re-verified for this
specification by reading the merged-`main` source and by real-binary probes at
`aiosh.exe layout …` (isolated `AIOSH_HOME`, throwaway stores): the built-in layouts' exact
mount options, modes, `dump`/`pass`/`required` values and `created_at` shape were captured from
`layout list`/`layout show --json` output, not from source-reading alone. Compatibility of every
NEW validation rule against both built-ins was checked against that captured data (§9).
This document is self-contained: a reviewer needs the source only to *implement*, not to
*review*.

---

## 1. Resolved decisions (D1–D9)

Each decision locks a T-01541 open question. Rationale cites the research fact that motivates
it. "Compat" = verified against both built-in layouts' captured data (§9).

| # | Decision | Rationale |
|---|---|---|
| **D1** | **Reject unknown JSON fields** at every spec deserialization entry. Error prefix `invalid layout JSON: unknown field '<name>' in <struct>`; the first unknown field is named, parsing stops there. | F2: typos validate silently today — the worst failure mode for a security-relevant contract. `deny_unknown_fields` is additive: every document that parses today and carries only known fields still parses (compat holds). |
| **D2** | **Range-validate `mode`**: legal iff `mode ≤ 0o7777`. Reject `mode == 0`. Do **not** reject any bit combination inside that range (setuid/setgid/sticky are legitimate operator choices). | F4: `mode=0` is legal today and means an unusable directory; there is no defensible reading of "POSIX permissions mode" where 0 is intended. Capping at `0o7777` bounds the value to the 12 permission bits; `u32` beyond that is a unit error (bytes-as-mode). Built-ins use `0o755/0o750/0o1777/0o777` — all in range (compat). |
| **D3** | **Extend FL4**: `/tmp` and `/dev/shm` mounts must carry `noexec` in addition to the existing `nodev` + `nosuid`. Error message gains a sibling: `FL4 violation: mount '/tmp' missing mandatory security option 'noexec'`. | CIS rationale (research §4) recommends all three on these mounts; both built-ins already carry `noexec` (probe-captured §9), so the tightening is backward-compatible and catches only layouts *worse* than the defaults. |
| **D4** | **Constrain `symlink_target` when present**: must be a *relative* UsrMerge-shaped path — non-empty, ≤1024 chars, no control chars (existing checks [REUSED]), **and** [NEW] must not start with `/`, must not contain a `.` or `..` segment, and its first segment must be `usr`. | F5 + A3: the doc-comment intent is UsrMerge; `usrmerge` entries in both built-ins are exactly `usr/bin`, `usr/sbin`, `usr/lib` (compat). Absolute targets would defeat the merge; relative-but-escaping targets would be path traversal. Applies only when `symlink_target` is `Some` — plain directories are untouched. |
| **D5** | **Keep "explicit store location only" as the permanent contract.** No discovered default store path is added. `AIOSH_HOME` continues to scope audit/PEP state only. The MCP `store_path` description's "(required: no default store is defined yet)" wording is updated to drop "yet" when touched. | F9: the absence of a default store is a deliberate security property (agents cannot mutate layout state they were not pointed at); a discovered default would silently widen every grant's blast radius. |
| **D6** | **Keep mount options free-form** beyond the existing per-option hygiene checks. No per-`FsType` vocabulary is introduced. FL4's mandatory-option checks remain the only content rules. | The option space is kernel/ext-tooling defined and version-dependent; a hardcoded vocabulary would reject valid configurations while the FL2-style hygiene checks (non-empty, no control chars/whitespace) already block injection into fstab lines. Built-ins use free-form options like `size=4G`, `fmask=0077` that a narrow vocabulary would threaten (compat). |
| **D7** | **Validate `created_at` as RFC 3339 UTC**: `Z`-suffixed, parseable by an RFC 3339 parser. [NEW] Validate `dump ∈ {0,1}` per fstab(5) ("dump frequency… 0 = no dump, 1 = daily backup" — fstab(5) field 5 semantics). `pass` bounds are already FL1-governed [REUSED]. | F10: both are unvalidated free strings/ints today. The built-in's `created_at` (`2026-09-16T00:00:00Z`) is RFC 3339 UTC (probe-captured), and all built-in `dump` values are `0` (compat), so both rules are additive-only. |
| **D8** | **Give `required` real semantics**: [NEW] a layout whose active-or-registered `mounts` contain **no** `required == true` mount fails validation with `FL6 violation: at least one mount must be marked required`. Individual `required` values are otherwise free; no boot-verification hook is specified (that is a future component's surface). | F10: the field is dead weight today; the minimal honest semantics is a sanity floor (a layout with no required mounts cannot be "operationally ready"). Both built-ins mark all six mounts required (compat). Naming it FL6 keeps FL1–FL5's documented meanings frozen (guide §3.1) and extends the sequence. |
| **D9** | **Keep default sizing as built-in constants** (`aios-uefi-standard-v1`: 512 MiB ESP / 50 GiB root / 4 GiB swap / 64 GiB floor; `aios-container-minimal-v1` as captured). No profile system in this component; `probe_target`'s `target_disk_bytes` parameter remains the operator's sizing lever, unchanged. | A2: no authority mandates these values; making them profile-dependent would invent a config surface this component has not been asked for. The existing min-disk + budget checks (research §1(6)) already gate operator-supplied sizes. |

**Consistency of the decision set:** D1–D4, D7–D8 are all *additive* validation rules — every
built-in layout and every document accepted by today's validator that carries only known fields
and sane values still passes (verified against captured built-in data, §9). D5–D6, D9 are
*preservation* decisions — they lock current behavior in as contract so later tasks cannot
weaken it silently.

---

## 2. Inputs (configuration surface)

**I1 — Layout spec document (JSON).** The unit of configuration. Deserialized by
`FilesystemLayoutSpec::from_json` (CLI `--spec`, MCP `spec` string) or `serde_json::from_value`
(MCP inline `layout` object). Schema = research F1: top-level `id, name, description,
target_disk_min_bytes, partitions[], mounts[], directories[], created_at` — **all mandatory**
[REUSED]; nested shapes per §5–§7. Unknown fields rejected [NEW per D1].

**I2 — Store location.** Always explicit [REUSED per D5]: CLI `--store <path>` (mutations
persist there; absent ⇒ in-memory store of built-ins only, nothing read or written — F9);
MCP mutations require `store_path` argument (MCP `register`/`set_active`/`remove`/
`import_fstab` schemas mark it required; `additionalProperties: false`). `AIOSH_HOME` scopes
audit/PEP state only [REUSED].

**I3 — fstab ingestion text.** `import_fstab`: 4–6-field fstab(5) lines, ≤128 mounts [REUSED],
auto-set `required = (path == "/")`, source bound ≤ `MAX_LAYOUT_DOC_BYTES` (10 MiB) [REUSED].

**I4 — Sizing input.** `probe_target(layout_id, target_disk_bytes)` — operator-supplied; no
new sizing knobs [REUSED per D9].

---

## 3. Outputs

**O1 — Validation verdict.** CLI: `VALID: Filesystem layout '<id>' satisfies all FL1..FL5 invariants`
(exit 0) or `INVALID: <first error>` (exit 1). **[NEW]** once FL6 lands, the success string
becomes `… all FL1..FL6 invariants` — the strings are observability surface, and the suite
asserts on the `VALID`/`INVALID:` prefixes, not the invariant count. MCP: `aios.fs_layout.validate`
returns the verdict inside the standard tool envelope (§6).

**O2 — Registration result.** CLI `layout register --spec …`: success
`SUCCESS: Registered filesystem layout '<id>'` / JSON `{"code":0,"data":{"id":…,"registered":true},"error":null}`
[REUSED, code-cited]. MCP `aios.fs_layout.register`: `ok:true` body carries `tool`, `id`,
`registered: true`, `active_layout_id` [REUSED, code-cited].

**O3 — Generated fstab.** `layout fstab` / `aios.fs_layout.fstab`: 6-field lines in fstab(5)
order — `fs_spec, fs_file, fs_vfstype, fs_mntops, fs_dump, fs_passno` — root pass 1, others 0
or 2 [REUSED; probe-verified against fstab(5) in T-01541]. This ordering is frozen contract.

**O4 — Read-only outputs.** `list` (sorted by id, built-ins seeded), `show [id]`, `diff`,
`probe` (min-disk + partition-budget verdicts with <10 % slack warning), `import-fstab`
(returns the synthesized spec) [REUSED].

---

## 4. Error cases (the failure matrix)

Every error below is surfaced as: CLI — `exit 1`, human message via `sanitize_terminal`
(control chars → U+FFFD [REUSED]), `--json` envelope `{"code":1,"data":null,"error":{"code":…,"message":…}}`;
MCP — `isError:true` result with exactly one honest `outcome` audit row; CLI subcommands write
exactly one audit row each [REUSED]. Codes are the existing ones where they exist; NEW codes
are AIOS-specific (marked).

| # | Trigger | Error surface | Status |
|---|---|---|---|
| E-1 | Unknown JSON field anywhere in the spec document | flows through the existing deserialization error path: `failed to deserialize layout JSON: unknown field `<name>`, expected one of …` (serde `deny_unknown_fields` wording; first offender named, parse stops) | **[NEW]** (D1) |
| E-2 | `mode == 0` or `mode > 0o7777` | `directory '<path>' mode must be in 1..=0o7777 (octal), found <value>` — unnumbered, matching `validate_directory_spec`'s existing per-element style (`directory owner cannot be empty`) | **[NEW]** (D2) |
| E-3 | `/tmp` or `/dev/shm` mount missing `noexec` | `FL4 violation: mount '<path>' missing mandatory security option 'noexec'` | **[NEW]** (D3) |
| E-4 | Absolute, `.`/`..`-segmented, or non-`usr`-rooted `symlink_target` | `directory '<path>' symlink_target '<t>' must be a relative path under 'usr' (UsrMerge)` — unnumbered, same per-element class as E-2 | **[NEW]** (D4) |
| E-5 | `created_at` not RFC 3339 UTC | `layout 'created_at' must be an RFC 3339 UTC timestamp, found '<value>'` — unnumbered, matching the validator's existing top-level field style (`layout name cannot be empty`) | **[NEW]** (D7) |
| E-6 | `dump` outside `{0,1}` | `mount '<path>' dump must be 0 or 1, found <n>` — unnumbered, matching `validate_mount_point`'s existing per-element style (`mount option cannot be empty`) | **[NEW]** (D7) |
| E-7 | No `required == true` mount | `FL6 violation: at least one mount must be marked required` | **[NEW]** (D8) |
| E-8 | Malformed JSON / missing mandatory field | existing deserialization error text (e.g. `failed to deserialize layout JSON: …`) | [REUSED] |
| E-9 | FL1–FL5 violations (root/pass rules, path hygiene, ordering, partition/ESP rules) | existing `FL<n> violation: …` messages, unchanged | [REUSED] |
| E-10 | Duplicate `id` at registration | `layout with id '<id>' is already registered` | [REUSED] |
| E-11 | `remove` of active or built-in layout | `cannot remove active layout '<id>'; switch active layout first` / `cannot remove built-in canonical layout '<id>'` | [REUSED] |
| E-12 | Store load failure | CLI envelope `LOAD_STORE_FAILED`; MCP error path | [REUSED] |
| E-13 | Persistence failure (≥ cap, replace failure, 10 MiB bound, unreadable dir) | CLI envelope `PERSIST_FAILED`; MCP `isError` + one `outcome=error` row; **no partially written state** (atomic fsync-then-rename, bounded retry, canonical-destination residue cap of 8, nothing deleted) | [REUSED — the T-01538/PR #4 contract] |
| E-14 | Policy refusal (PEP/scope) | MCP: refusal **before** any read/write, `isError` envelope, one honest audit row | [REUSED] |
| E-15 | fstab import: >128 mounts | `imported fstab exceeds maximum limit of 128 mounts (found <n>)` | [REUSED] |
| E-16 | Input document > 10 MiB, or unbounded/unreadable spec path | `read_bounded_text_file` error surfaces | [REUSED] |
| E-17 | `set_active` unknown id / `probe_target` infeasible size | existing errors (`layout with id '<id>' not found in store`; probe verdict message) | [REUSED] |

**Ordering rule [NEW], matching the validator's actual pass structure:** E-1 (schema rejection) precedes everything — an unknown field is a schema error, not a layout defect. Then, in the validator's existing order: top-level shape checks (id/name/description bounds, count limits, and **E-5**) → FL1 (exactly one root; **E-7** immediately after, as the second whole-mount-set invariant) → per-mount loop (`validate_mount_point`: FL2 hygiene, device/option shape, **E-6**, FL1 pass rules, FL4 incl. **E-3**) → FL3 duplicate/ordering → FL5 partitions → directory loop last (`validate_directory_spec`: owner/group/symlink shape, **E-2**, **E-4**) → duplicate-id check at registration. A document with multiple defects reports the first in this order (matches today's first-error-wins behavior).

---

## 5. Validation contract (the exact NEW rules)

Normative statement of every rule this specification *adds*, in implementable terms:

- **V-1 (D1).** All five spec structs — `FilesystemLayoutSpec`, `PartitionSpec`,
  `MountPointSpec`, `DirectorySpec`, and the `FsType`/`PartitionType` enum representations —
  reject unknown fields. Scope note: the store *file* (`FilesystemLayoutStore`) is a separate
  document already carrying `active_layout_id`; its field set is unchanged, and unknown-field
  rejection applies to the store document only if T-01543's scaffold puts spec parsing and
  store persistence on the same serde derive path — **recommended**, not required, by this spec.
- **V-2 (D2).** `DirectorySpec.mode`: reject `0` and `> 0o7777`. No other constraint.
- **V-3 (D3).** `MountPointSpec` with `path ∈ {"/tmp", "/dev/shm"}`: options must contain
  `nodev`, `nosuid`, **and** `noexec` (exact string match on the option token, as FL4 does
  today — not substring, not `=`-separated key=value).
- **V-4 (D4).** `DirectorySpec.symlink_target = Some(t)`: `t` must not begin with `/`; no
  segment of `t` may be `.` or `..`; first segment must be `usr`. (Length/control-char bounds
  are the existing ones.) Applies only to symlink entries; `None` is exempt.
- **V-5 (D7).** `created_at`: parses as RFC 3339 with `Z` designator (UTC). `dump ∈ {0,1}`.
- **V-6 (D8).** `mounts.iter().any(|m| m.required)` must hold — error class FL6, first new
  invariant number since the FL sequence froze.
- **V-7.** Error-message strings for NEW rules are exactly those in §4 (E-1..E-7), and they
  follow the validator's existing two-class convention: whole-layout invariants carry
  `FL<n> violation:` prefixes (FL1–FL5 today, FL6 added); per-element field checks inside
  `validate_mount_point`/`validate_directory_spec` are **unnumbered** (as `directory owner
  cannot be empty` and `mount option cannot be empty` are today). NEW per-element rules must
  not invent FL numbers.

Everything else about validation is [REUSED] and frozen: FL1–FL5 semantics and messages, path
hygiene bounds (1024 chars, control chars, `//`, `.`, `..`, trailing slash), device bounds
(≤256, no whitespace/control), option hygiene, partition index 1..=128, ESP ≥100 MiB + vfat,
root pass 1 / others ≠1, unique + parent-before-child mount ordering, exactly-one-root.

---

## 6. Persistence effects and audit effects

**Persistence [REUSED, frozen]:** mutations (`register`, `set_active`, `remove`,
`import_fstab`) persist only when a store location was supplied. The store file is written
atomically — staged temp beside the destination, fsync, bounded transient-only replace retry
(5 attempts, 20 ms base backoff, 5 s wall-clock budget) — inside the 10 MiB document bound,
with residue refused once 8 staged files sit beside the same **canonical** destination and
nothing ever deleted (the T-01538/PR #4 contract, adversarially verified in T-01540).
Read-only verbs never persist. In-memory mutation without `--store`/`store_path` is possible
for reads-after-write within one process but leaves no file.

**Audit effects [REUSED, frozen]:** each CLI layout subcommand emits exactly one audit row
(`classify_and_emit` → `AuditRing`): `tool="fs_layout"`, `command=<verb>`,
`outcome ∈ {success, failure}`, `outcome_detail` naming the stage that failed, target = layout
id where one exists, args snapshot, hash-chained to the ring. MCP writes one row per tool call
with the same honesty property; refusals are recorded as errors with the refusal reason — the
envelope and the row never disagree. NEW validation rules change **which** calls fail, never
**how** failures are recorded: one row per call, no partial state, no silent success. A
rejected-by-FL6 registration is recorded exactly like a rejected-by-FL1 one today.

---

## 7. Interfaces: reused vs new

**Reused unchanged (implementation tasks must not alter their signatures):**
- `FilesystemLayoutSpec::{from_json, to_json, validate}`, `validate_filesystem_layout`,
  `validate_path_hygiene`, `validate_partition_spec`, `validate_mount_point`,
  `validate_directory_spec` (`aiosh-core/src/fs_layout.rs`);
- `FilesystemLayoutService::{register_layout, remove_layout, set_active_layout, get_layout,
  list_layouts, get_active_layout, import_fstab_as_layout, probe_target, diff_layouts,
  save_to_path, load_from_path}` and the bounds constants (`MAX_LAYOUT_DOC_BYTES`,
  `MAX_STAGED_KEEP`, `MAX_REPLACE_ATTEMPTS`) (`aiosh-core/src/fs_layout_service.rs`);
- CLI verbs exactly as `aiosh layout` prints them today: `list, show, validate, check, probe,
  diff, fstab, register, set-active, remove, import-fstab` with their current flags;
- MCP tools `aios.fs_layout.{list,get,validate,fstab,probe,diff,register,set_active,remove,import_fstab}`
  with their current `inputSchema`s — `store_path` required on mutations,
  `additionalProperties: false`.
- The `.`/`..` `scope.paths` matcher and canonical-destination keying from the just-landed
  fixes (`pep.rs`, `fs_layout_service_key`) — configuration parsing sits behind them unchanged.

**New (declared for T-01543, all AIOS-specific, none invented upstream):**
- **N-1:** unknown-field rejection on spec deserialization (serde
  `deny_unknown_fields` or hand-rolled equivalent — implementation detail, contract is E-1's
  message and first-offender semantics);
- **N-2:** directory `mode` range validation inside `validate_directory_spec` (same function,
  extended — not a new public function);
- **N-3:** FL4 gains the `noexec` check (same function, extended);
- **N-4:** `symlink_target` UsrMerge validation inside `validate_directory_spec`;
- **N-5:** `created_at` RFC 3339 UTC + `dump ∈ {0,1}` checks inside the existing validators;
- **N-6:** FL6 in `validate_filesystem_layout` (the only new invariant number);
- **N-7:** the success-string update `FL1..FL5` → `FL1..FL6` in CLI/MCP validate output (§3 O1).
- **N-8 (docs):** guide §6.13's "mode is never validated" claim and any equivalent statements
  are updated when N-2 lands — flagged now so the doc correction ships with the code, not after.

No new CLI verbs, no new MCP tools, no new env vars, no new files, no store-format change.

---

## 8. Happy path and failure path, end to end

**Happy path (operator registers a hardened custom layout):**
`aiosh layout register --spec ./profile.json --store /etc/aios/layout-store.json`
→ file read bounded (≤10 MiB) → schema parse (unknown fields now rejected, E-1) →
`validate_filesystem_layout`: FL1–FL5 + new rules V-2..V-6 → duplicate-id check → in-memory
insert → atomic persist (E-13 contract) → one `outcome=success` audit row →
`SUCCESS: Registered filesystem layout '<id>'` (or the JSON envelope). Subsequent
`layout fstab <id>` emits fstab(5)-ordered content (O3); `aios.fs_layout.probe` gates it
against `target_disk_bytes`.

**Failure path (the same call with `mode: 0` on one directory):**
persist never begins — refusal happens at validation, before staging. CLI: exit 1, message
`directory '<path>' mode must be in 1..=0o7777 (octal), found 0` (E-2), one
`outcome=failure` audit row, store file untouched (byte-identical). MCP: `isError:true`
envelope, one honest `outcome=error` row, no staged file created. The pre-existing staged
files beside the store are unaffected (never deleted — per-store scoping preserved).

---

## 9. Compatibility check against the built-ins (probe-captured)

Captured from the real binary (`layout list` + `layout show --json`, isolated `AIOSH_HOME`,
2026-09-18):

- `aios-uefi-standard-v1`: mounts `/` ext4 pass 1 `rw,relatime,errors=remount-ro`;
  `/boot/efi` vfat pass 2 (+`fmask=0077,dmask=0077,nodev,nosuid`); `/tmp` tmpfs
  `rw,nosuid,nodev,noexec,size=4G`; `/dev/shm` tmpfs `rw,nosuid,nodev,noexec`; `/proc` procfs
  and `/sys` sysfs both `rw,nosuid,nodev,noexec`. All six `required: true`; all `dump: 0`.
  Directories: `/var/lib/aios`, `/run/aios`, `/var/log/aios` at `0o750`, `/etc/aios` at
  `0o755`, `/tmp` at `0o1777`, usrmerge symlinks `0o777` → `usr/bin`, `usr/sbin`, `usr/lib`.
  `created_at: 2026-09-16T00:00:00Z`.
- `aios-container-minimal-v1`: 0 partitions; mounts `/`, `/boot/efi`, `/tmp`, `/dev/shm`,
  `/proc`, `/sys` with the same option shapes; same `dump`/`required`/`created_at` properties.

Against the NEW rules: every directory mode ∈ {0o750, 0o755, 0o1777, 0o777} passes V-2; every
`/tmp` and `/dev/shm` mount already carries `noexec` (V-3); every symlink target is relative,
`..`-free, `usr`-rooted (V-4); `created_at` parses as RFC 3339 UTC and all dumps are 0 (V-5);
all mounts required (V-6). **Both built-ins remain valid under the full NEW rule set; every
rule is additive.** Under the *old* rules all of these also pass — the changes reject only
documents the built-ins never were.

---

## 10. Reviewability check (ledger acceptance)

A reviewer needs only this document plus T-01541's research to review the contract: the input
shapes are §2/§5–§7 of the research; the outputs §3 here; every failure case §4 with exact
message text; persistence and audit effects §6 with the frozen constants; the reused/new
interface split §7; end-to-end behavior §8; and the proof that nothing here breaks the
built-ins §9. No implementation reading required — the source citations that *do* exist
(research F1–F11, §7's module paths) are anchors for the implementer, not review prerequisites.

**Out of scope for this component chain:** a discovered default store path (D5 — rejected),
per-`FsType` option vocabularies (D6 — rejected), profile-dependent sizing (D9 — rejected),
boot-time verification of `required` mounts, store-format versioning, and any change to
FL1–FL5 semantics. The sealed-store finding (guide §6.12) remains a documented residual —
untouched by this specification.
