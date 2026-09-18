# T-01541: Filesystem Layout — configuration: Research

## Metadata
- **Task ID:** `T-01541`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout
- **Status:** Complete — research only. No code changed (verified: the pending tree after this
  task contains only this evidence file and ledger state).
- **Date:** 2026-09-18
- **Dependencies:** `T-01540` (closes the Filesystem Layout MCP/API-surface chain; this task opens
  the *configuration* sub-chain of the same component).
- **Feeds:** `T-01542` — "configuration: Specification" (`docs/tasks/evidence/T-01542-configuration-specification.md`),
  which must turn this research into an exact contract.
- **Artifact path note (recorded, not silently resolved):** the ledger `instructions` say
  "docs/tasks/evidence/T-01541-research.md" while the ledger `artifacts` (the binding contract the
  evidence checker validates against) say `docs/tasks/evidence/T-01541-configuration-research.md`.
  This file is written at the **artifacts** path; T-01542 should normalise the instruction text.

---

## 1. Scope & Method

**What "the configuration of Filesystem Layout" is.** The component's configuration surface, as
established by reading the code and the guide before assuming anything is missing, comprises:

1. the **layout spec JSON schema** — `FilesystemLayoutSpec` and its nested `PartitionSpec`,
   `MountPointSpec`, `DirectorySpec`, plus the `FsType`/`PartitionType` vocabularies
   (`code/aiosh-rust/aiosh-core/src/fs_layout.rs:8,115,195,210,231`);
2. the **validation invariants** FL1..FL5 that gate every registration/import
   (`fs_layout.rs:676` `validate_filesystem_layout`; FL2 hygiene `:517`; FL4 hardening `:596`;
   FL5 partition rules `:617,725`);
3. the **built-in defaults and active-layout selection** — `FilesystemLayoutStore::new()`
   seeds `aios-uefi-standard-v1` (active) and `aios-container-minimal-v1`
   (`fs_layout_service.rs:436-446`);
4. the **store location contract** — no default path is consulted: the CLI loads/persists a store
   *only* when `--store` is passed (`aiosh-cli/src/main.rs:3524-3535`), MCP mutations require an
   explicit `store_path` argument (guide §5.0; FL8 contract), and `AIOSH_HOME` scopes audit/PEP
   state, not the layout store (`aiosh-core/src/audit.rs:87`);
5. the **fstab ingestion path** — `import_fstab` parses 4–6-field `fstab(5)` lines with a 128-mount
   cap (`fs_layout.rs:163-217`; `fs_layout_service.rs:788-789`);
6. the **sizing inputs** — `probe_target(layout_id, target_disk_bytes)` checks the minimum-disk and
   partition-budget bounds with a <10 % slack warning (`fs_layout_service.rs:552-580`);
7. the **persistence contract** — ≤10 MiB doc bound (`:19`), fsync-then-rename atomic write,
   bounded transient-only retry (`:37`), and the canonical-destination staged-residue cap of 8
   (`:50,57,820-940`) delivered by T-01538/PR #4.

**Method.** Every code claim below was read out of the merged `main` source (line numbers cited);
the behavioural claims were **reproduced through the real binaries** — `aiosh.exe layout …` CLI
invocations with an isolated `AIOSH_HOME` under the OS temp dir and throwaway spec files
(4 probes, 6 scenarios). External claims are anchored to the authoritative sources in §4.
One method correction is recorded honestly: an initial regex slice of `standard_uefi()` from source
over-counted directory entries (it crossed a function boundary); the **real-binary `layout show`**
output (8 entries) is the figure of record.

---

## 2. Facts (code-cited or probe-reproduced)

**F1 — Spec schema and field contract.** `FilesystemLayoutSpec` = `id`, `name`, `description`,
`target_disk_min_bytes`, `partitions[]`, `mounts[]`, `directories[]`, `created_at`
(`fs_layout.rs:231`). Every field is mandatory: there is no `#[serde(default)]` on any spec struct
(grep over `fs_layout.rs` returns none).

**F2 — Unknown JSON fields are silently ignored.** No `deny_unknown_fields` exists anywhere in
`aiosh-core` (grep: none). Probe 2 reproduced it at the real CLI: a spec with an unknown top-level
field and unknown per-partition fields validates — exit 0,
`VALID: Filesystem layout 'aios-uefi-standard-v1' satisfies all FL1..FL5 invariants`.

**F3 — FL1..FL5 are implemented as documented.** Exactly-one-root + pass rules (FL1), path hygiene
rejecting `..`/`//`/trailing slash/control chars (FL2, `:517-560`), unique + parent-before-child
mount ordering (FL3), partition index/size/ESP rules (FL5). Probe 3 reproduced the FL4 refusal
verbatim: after stripping `nodev`/`nosuid` from `/tmp`, validation returns
`INVALID: FL4 violation: mount '/tmp' missing mandatory security option 'nodev'` with a non-zero
exit.

**F4 — `mode` is not validated.** `validate_directory_spec` (`fs_layout.rs:640-668`) checks path
hygiene, owner/group shape and `symlink_target` bounds — never `mode`. Probe 3 reproduced the gap:
`directories[0].mode = 0` on a fresh path validates (`VALID …`). The built-in default itself
carries `mode=0o777` on the three usrmerge symlink entries (`/bin`, `/sbin`, `/lib`).

**F5 — `symlink_target` is bounded but not scoped.** Only non-empty / ≤1024 chars / no control
chars are enforced (`:657-666`); the UsrMerge intent (relative target under `/usr`) lives in the
doc comment only.

**F6 — Built-in defaults (from the real binary, `layout show --json`).**
`aios-uefi-standard-v1`: partitions `EFI 512 MiB`, `AIOS_ROOT 51200 MiB`, `AIOS_SWAP 4096 MiB`;
min disk 64 GiB; mounts `/` ext4 pass 1, `/boot/efi` vfat pass 2, `/tmp` + `/dev/shm` tmpfs
(nosuid,nodev,noexec), `/proc` procfs, `/sys` sysfs; 8 directory entries — `/var/lib/aios`,
`/run/aios`, `/var/log/aios` (0o750, root:aios), `/etc/aios` (0o755), `/tmp` (0o1777), and the
usrmerge symlinks `/bin→usr/bin`, `/sbin→usr/sbin`, `/lib→usr/lib`.
`aios-container-minimal-v1`: 0 partitions, mounts `/`, `/proc`, `/sys`, `/tmp`.

**F7 — The hard-coded GPT type-GUID table matches the canonical values exactly.** All six GUIDs in
`fs_layout.rs:100-112` were compared programmatically against the UAPI Discoverable Partitions
Specification / UEFI tables: ESP `c12a7328-…`, root-x86_64 `4f68bce3-…`, home `933ac7e1-…`, swap
`0657fd6d-…`, var `4d21b016-…`, generic `0fc63daf-…` — **6/6 match**, no extras, no alterations.

**F8 — Generated fstab matches fstab(5) field order.** Probe: `layout fstab aios-uefi-standard-v1`
emits `fs_spec` first (`LABEL=AIOS_ROOT  /  ext4  rw,relatime,errors=remount-ro  0  1`, then
`/boot/efi` vfat pass 2, `/tmp` and `/dev/shm` tmpfs pass 0) — the canonical 6-field order
(device, path, type, options, dump, pass), root pass 1, per fstab(5).

**F9 — Store semantics.** With no `--store`, the CLI builds an in-memory store of the two built-ins
and never reads or writes a layout-store file (Probe 4: fresh `AIOSH_HOME`, `layout list` → the two
built-in ids, `active: aios-uefi-standard-v1`). Built-ins cannot be removed and the active layout
cannot be removed until switched (`fs_layout_service.rs:481,487`). Mutations persist atomically
under the residue-cap bounds of §1(7).

**F10 — `required` and `created_at` are carried but unenforced.** `MountPointSpec.required` is a
free bool in direct registration and is auto-set only by fstab import (`path == "/"`,
`fs_layout.rs:210`); `validate_filesystem_layout` never reads it. `created_at` is a free-form
string — the validator (`:676-771`) never checks its shape.

**F11 — The doc's invariant list matches the code.** Guide §3.1 (FL1..FL5) and §3.2 (CS1..CS5) were
checked claim-by-claim against the validator and service; no claim in either table was found
contradicted by the code or the probes.

---

## 3. Assumptions (labeled as such — reasonable, not proven)

**A1 — AIOS directory placement.** `/etc/aios`, `/var/lib/aios`, `/run/aios`, `/var/log/aios`
follow FHS parent-directory semantics (§4 citations), but the `aios` sub-names are project
choices; no external standard constrains them.

**A2 — Default sizing adequacy.** 512 MiB ESP / 50 GiB root / 4 GiB swap and the 64 GiB floor are
plausible for a general-purpose UEFI host, but no authoritative source mandates them; adequacy
depends on the deployment profile the specification will target.

**A3 — usrmerge symlink entries' `mode=0o777`.** Presumed a deliberate "don't care" placeholder
(real permissions come from the symlink targets), but nothing in the code or docs says so.

**A4 — tmpfs sizing choice.** `/tmp` capped at `size=4G` is a workload assumption, not a standard
requirement.

## Unknowns and decisions needed before T-01542 (explicit)

| # | Decision | Context (facts involved) |
|---|---|---|
| D1 | Reject unknown JSON fields (`deny_unknown_fields`), tolerate-and-warn, or keep silent tolerance? | F2 — today typos in field names validate silently |
| D2 | Range-validate `mode` (reject 0 / >0o7777 / setuid+sticky oddities)? | F4 — `mode=0` is legal today |
| D3 | Extend FL4 to require `noexec` on `/tmp` and `/dev/shm` (CIS recommends nodev,nosuid,noexec)? Defaults already carry it; the invariant does not. | F3, A4 |
| D4 | Constrain `symlink_target` to UsrMerge shapes (relative `usr/…`), or keep free-form? | F5 |
| D5 | Keep "explicit store location only" as the permanent contract, or define a discovered default (e.g. under `AIOSH_HOME`)? | F9 |
| D6 | Constrain mount options to a per-`FsType` vocabulary, or keep free-form strings? | F1/F3 — options are unvalidated beyond hygiene |
| D7 | Validate `created_at` (RFC 3339) and `dump` bounds? | F10 — both are unvalidated today |
| D8 | Give `required` real semantics (enforced registration invariant / boot-verification hook) or document it as advisory? | F10 |
| D9 | Confirm default sizing (A2) as a specification constant, make it profile-dependent, or leave it operator-supplied? | A2 |

---

## 4. Authoritative sources & citations

- **FHS 3.0**, The Linux Foundation, 2015-06-03 — <https://refspecs.linuxfoundation.org/FHS_3.0/fhs/>.
  §3.2 "Requirements": the required root entries (`bin boot dev etc lib media mnt opt run sbin srv
  tmp usr var`), each *"or symbolic links to directories"* — the wording that directly legitimises
  the component's usrmerge symlink entries (F6). §3.7 `/etc` host-specific configuration;
  §3.15 `/run` run-time data; §3.18 `/tmp`; §5.8 `/var/lib`; §5.10 `/var/log` — the parent
  semantics behind A1.
- **fstab(5)**, Linux man-pages (util-linux) — <https://man7.org/linux/man-pages/man5/fstab.5.html>.
  Six fields fs_spec…fs_passno; *"The root filesystem should be specified with a fs_passno of 1.
  Other filesystems should have a fs_passno of 2"*; LABEL=/UUID= preferred over device names;
  `\040`/`\011` escaping; record order matters to fsck/mount — anchors F3 (pass rules) and F8.
- **UAPI Group, Discoverable Partitions Specification (UAPI.2)** —
  <https://uapi-group.org/specifications/specs/discoverable_partitions_specification/>. Defines
  `SD_GPT_ROOT_X86_64 = 4f68bce3-e8cd-4db1-96e7-fbcaf984b709`, `SD_GPT_HOME =
  933ac7e1-2eb4-4f13-b844-0e14e2aef915`, `SD_GPT_SWAP = 0657fd6d-a4ab-43c4-84e5-0933c84b4f4f`,
  `SD_GPT_VAR = 4d21b016-b534-45c2-a9fb-5c16e091fd2d`, `SD_GPT_LINUX_GENERIC =
  0fc63daf-8483-4772-8e79-3d69d8477de4` — anchors F7.
- **UEFI Specification 2.10, §12.3 (EFI System Partition)** — ESP GUID
  `c12a7328-f81f-11d2-ba4b-00a0c93ec93b`, FAT-based — anchors F7 (ESP row) and A2 (ESP sizing is
  deployment-dependent; the standard mandates no minimum).
- **CIS Distribution Independent Linux Benchmark v2.0.0, §1.1** (partition hardening: nodev, nosuid,
  noexec on `/tmp`, `/dev/shm`) — the rationale source for decision D3. *Assumption-flagged:* the
  exact section numbering follows the widely used benchmark layout and was not re-fetched during
  this pass; the FL4 option set itself is code-verified (F3).

## 5. Verification record

| Check | Result |
|---|---|
| `bash ci/run_all_smokes.sh` (mandated baseline gate, attempted for the record) | **FAIL as documented** — `rust_smoke` exits 1 with the WSL-stub error ("Windows Subsystem for Linux has no installed distributions"), pre-existing and host-level, identical to T-01538/T-01540's record |
| `python tools/test_fs_layout_suites.py` (FL1–FL8, incl. MCP contract C1–C8) | **PASS** (all criteria) |
| `cargo test -p aiosh-core` | **602 passed / 0 failed** |
| `cargo test -p aiosh-mcp` | **17 passed / 0 failed** |
| `cargo test -p aiosh-cli` | **24 passed / 0 failed** |
| `python tools/check_task_docs.py` | **PASS** (C1..C6) |
| `python tools/task_ledger.py validate` | **consistent**, `last_event_seq: 1540` before advance |
| Real-binary probes (4 probes / 6 scenarios, §2 F2–F9) | **all reproduced as stated**; temp stores under OS temp, isolated `AIOSH_HOME`, nothing written into the repository, probe scripts deleted after use |

**Acceptance check against the ledger.** *Evidence file exists and separates facts from
assumptions* — §2 facts are individually code-cited or probe-reproduced; §3 assumptions are
labeled; §4 separates external fact from the one unverified citation. *No code changed* — the
pending tree after this task contains exactly this evidence file and the ledger state;
`git status` was verified before writing. *Decisions needed are listed explicitly* — D1–D9 in §3.

**Defects found (for the record, none blocking this task).** No code defect was found in the
configuration surface itself; F2 (silent unknown-field tolerance), F4 (`mode` unvalidated) and
F10 (`required`/`created_at` unenforced) are **contract-hardening gaps for T-01542 to decide on**,
not malfunctions — each currently behaves as the code reads, and the guide does not claim
otherwise.
