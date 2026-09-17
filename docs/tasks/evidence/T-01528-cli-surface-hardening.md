# T-01528: Filesystem Layout - CLI Surface: Hardening

## Metadata
- **Task ID:** `T-01528`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface (`code/aiosh-rust/aiosh-cli::cmd_fs_layout`) and its persistence layer (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (8/10) — CLI Surface Hardening
- **Dependencies:** `T-01527` (CLI Surface Security Review)
- **Next Task:** `T-01529` (Filesystem Layout / CLI surface: Documentation)

---

## 1. Scope

Hardened the Filesystem Layout CLI surface against failure and misuse, per the task contract:

- timeouts, size caps, and bounded retries where external processes or files are involved;
- errors reported in the standard result envelope, never silently;
- resource cleanup on the error path (DB connections, temp files, child processes);
- fail-open behaviour always writing an honest audit row (ADR-0035 §F-2).

Surface audited: `cmd_fs_layout` dispatch plus `cmd_fs_layout_register`,
`cmd_fs_layout_set_active`, `cmd_fs_layout_remove`, `cmd_fs_layout_import_fstab`, the `resolve_layout`
resolver, the `load_fs_layout_service` store resolver, and the persistent store writer/reader in
`fs_layout_service`.

**No external processes are involved.** Verified that the surface spawns nothing:

```
$ grep -n "Command::new\|process::Command" code/aiosh-rust/aiosh-cli/src/main.rs
(no matches)
```

Process timeouts are therefore not applicable to this surface. The file analogue — an unbounded or
blocking read — was real and is addressed as H-4/H-5.

---

## 2. Findings and Fixes

### H-1 — Store replacement was not atomic and could destroy the registry (**HIGH**, fixed)

**Vector.** `save_to_path` deleted the destination *before* renaming the staged file into place:

```rust
if path.exists() {
    let _ = fs::remove_file(path);      // destination gone from here...
}
fs::rename(&tmp_path, path)             // ...until here
```

Between the unlink and the rename the store file **does not exist**. A crash, `ENOSPC`, or a signal
in that window leaves the operator with no store at all: the previous good state was destroyed and
the new state was never installed. The unlink was also unnecessary — `fs::rename` already replaces
an existing destination atomically on both POSIX and Windows (Rust's std maps it to
`MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`).

**Fix.** The destination is never unlinked. The staged file is renamed directly over the target, so a
crash leaves either the complete old store or the complete new one, never nothing.

### H-2 — The staged file was deleted when the rename failed (**HIGH**, fixed)

**Vector.** The rename error path began with `let _ = fs::remove_file(&tmp_path);`. At that moment the
staged file is the **only complete copy** of the state the caller asked to persist — the destination
still holds the *old* store. Deleting it discarded the caller's data silently, behind an error message
that mentioned only the failed rename.

**Fix.** A failed rename now preserves the staged file and names it in the error, so the operator can
recover the state that could not be installed:

```
failed to atomically replace store file '<dest>' from '<staged>': <cause>
(the fully-written new store was preserved at '<staged>' for manual recovery)
```

A failed *write* (not rename) still removes the partial staged file, so a later run can never pick up
a truncated store.

### H-3 — Predictable temporary path enabled a symlink/TOCTOU attack (**HIGH**, fixed)

**Vector.** The staged path was derived deterministically from a value the caller controls plus the
process id:

```rust
let pid = std::process::id();
let tmp_path = path.with_extension(format!("tmp.{}", pid));   // e.g. layouts.tmp.4321
```

An attacker with write access to the store directory could pre-plant that exact name as a symlink to
any file the CLI can write (an operator's `~/.ssh/authorized_keys`, a systemd unit, …) and have the
CLI write attacker-chosen content with its own privileges. The pre-existing
`if tmp_path.exists() { let _ = fs::remove_file(&tmp_path); }` mitigated only the *dangling* case,
introduced its own check-then-use race (the attacker simply re-creates the link), and silently
destroyed any legitimate file that happened to sit at that path.

**Fix.** Staged files are created with `OpenOptions::new().write(true).create_new(true)`
(`O_CREAT | O_EXCL`), which the kernel creates or fails — it is never opened through a pre-existing
path, and `O_EXCL` does not follow a symlink for the final component. The name now carries a
nanosecond timestamp and an attempt counter so it is not guessable, and a collision (including a
deliberately pre-planted name) costs at most one **bounded retry** (`MAX_TEMP_ATTEMPTS = 8`) before
failing with an explicit error.

### H-4 — Readers followed FIFOs and devices: unauthenticated hang / unbounded memory (**HIGH**, fixed)

**Vector.** Both `--spec`/`--fstab` and `--store` resolved a path with `path.exists()` and then read it
with `std::fs::read_to_string`. Nothing required the path to be a *regular file*:

- a **FIFO** reports a metadata length of `0`, so a length-based cap passes, and the read then blocks
  **forever** waiting for a writer — an unauthenticated denial of service against the CLI and any
  automation wrapping it;
- a **character device** such as `/dev/zero` passes the same check and then never reaches EOF,
  streaming until the process exhausts memory;
- a **directory** produced an opaque platform I/O error (`Access is denied`, `EISDIR`) instead of a
  statement of what was wrong.

**Fix.** A shared `read_bounded_text_file(path, max_bytes, label)` in `fs_layout_service` stats first
and rejects anything that is not a regular file by *type*, naming the defect. It is used by
`FilesystemLayoutService::load_from_path` (shared by the CLI and the MCP server, so both surfaces are
covered by one guard), by the CLI `register` spec read, by the `resolve_layout` resolver used by
`show`/`validate`/`probe`/`fstab`, and by `import-fstab`. Symlinks to regular files remain readable:
only reads happen here, and the write path never follows a pre-existing link (H-3).

### H-5 — Size cap applied only to the pre-read metadata snapshot (**MEDIUM**, fixed)

**Vector.** `meta.len() > 10 MiB` was checked before reading, then the whole file was read
unconditionally. A document that grows between the check and the read (or a pseudo-file that reports
`0` and yields real content, as under `/proc`) bypassed the cap entirely.

**Fix.** The read is capped at the stream level: the reader takes `max_bytes + 1` bytes and rejects any
document that reaches the extra byte, so the limit holds regardless of what metadata claimed. The
metadata check is kept as a fast path that also produces the dedicated size error.

### H-6 — "Atomic" persistence was not durable (**MEDIUM**, fixed)

**Vector.** The staged bytes were written with `fs::write` and immediately renamed. Without a flush to
stable storage, a power loss after the call returned could leave a renamed-but-empty or partially
written store, contradicting the "atomically persists" contract on the method.

**Fix.** The staged file is written with `write_all` followed by `sync_all()` before the rename.

### H-7 — Distinct failure causes collapsed into one opaque error (**LOW**, fixed)

**Vector.** Every disk problem on the spec path reported the same `SPEC_READ_FAILED`
("failed to inspect spec file" / "failed to read spec file"). An operator could not tell a typo'd path
from a directory, a device, a permission problem, or non-UTF-8 content.

**Fix.** The reader returns a typed `LayoutDocReadError` (`NotFound`, `NotRegularFile`, `TooLarge`,
`NotUtf8`, `Io`) and the CLI maps each onto an explicit code in the standard envelope **and** its own
audit row, so failures are explicit and auditable rather than generic:

| Condition | Code | Exit |
|---|---|---|
| `--spec` names a directory / FIFO / device | `SPEC_NOT_REGULAR_FILE` | 1 |
| `--fstab` names a directory / FIFO / device | `FSTAB_NOT_REGULAR_FILE` | 1 |
| `--spec` exceeds 10 MiB | `SPEC_SIZE_EXCEEDED` | 1 |
| `--fstab` exceeds 10 MiB | `FSTAB_SIZE_EXCEEDED` | 1 |
| `--spec` is not valid UTF-8 / other I/O failure | `SPEC_READ_FAILED` | 1 |
| `--store` is not a regular file / unreadable / oversized | `LOAD_STORE_FAILED` | 1 |

### Accepted risks and limits (documented, not silently omitted)

1. **No external processes** are spawned by this surface, so no process timeouts or child-process
   reaping are required here.
2. **A crash can leave one stale staged file.** Names are deliberately unguessable (H-3), so a
   leftover `.store.json.tmp.<pid>.<nanos>.<n>` cannot be swept by pattern without reintroducing an
   attacker-influenced unlink primitive. The residue is inert garbage in the store directory, and the
   destination is always either the old or the new complete store (H-1). Recovery guidance belongs to
   T-01529's documentation.
3. **Windows DOS device names bypass `exists()`.** `Path::new("NUL").exists()` is false on Windows, so
   `--store NUL` takes the documented "store file missing → seeded canonical presets" path rather than
   being read. Nothing is read from or written to the device, so this is benign; it is recorded here
   because it is the one path where a non-regular file is neither rejected nor opened.
4. **Inline `--spec` JSON is bounded by the OS**, not by this surface: `argv` cannot exceed
   `MAX_ARG_STRLEN` (~128 KiB on Linux, ~32 KiB on Windows), which binds well below the 10 MiB
   document ceiling (carried forward from T-01527).

---

## 3. Proof

### 3.1 Core hardening suite (Rust, `aiosh-core`)

Three tests added to `code/aiosh-rust/aiosh-core/tests/test_fs_layout_service.rs` (14 pass in total,
up from 11):

```
test test_fs_layout_save_replaces_atomically_and_leaves_no_temp_files ... ok
test test_fs_layout_save_preserves_staged_file_when_rename_fails ... ok
test test_fs_layout_reads_reject_non_regular_files_and_oversize_documents ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

- **Atomic replacement / no leftovers** — saves over an existing store, then asserts the destination
  exists the instant the call returns, holds the *new* complete state (previous layouts preserved), and
  that no staged `.name.tmp.*` file survives a success.
- **Staged-file preservation** — a non-empty directory at the destination makes the rename fail on
  every platform (POSIX `EISDIR`, Windows `ACCESS_DENIED`); asserts the error names the preserved file
  and that the staged file is still on disk with the full new store content.
- **Type/UTF-8/cap rejection** — a directory is refused by type with "is a directory" rather than an
  opaque `Access is denied`; a 4 KiB document against a 64-byte cap is refused as `TooLarge`
  (proving the cap is enforced during the read, not only from metadata); non-UTF-8 bytes are reported
  as `NotUtf8`.

`test_fs_layout_reader_bounds_growing_proc_files` (`#[cfg(target_os = "linux")]`) pins the read-time
cap on the case metadata alone cannot catch: `/proc/self/status` reports a metadata length of `0`
while yielding real content. It compiles out on Windows and runs in Linux CI.

### 3.2 CLI hardening proof suite (real binary)

New suite **`code/aiosh-cli/tests/test_fs_layout_hardening.py`** drives the real `aiosh` binary and
asserts only observable behaviour: exit status, `{code, data, error}` envelopes, the store JSON on
disk, the staged files present in the working directory, and the real SQLite `audit_ring`.

```
$ python code/aiosh-cli/tests/test_fs_layout_hardening.py
=== RUNNING FILESYSTEM LAYOUT CLI HARDENING PROOF ===
PASS: H-1 non-regular paths rejected by type, explicitly and audited
SKIP: H-2 FIFO/character-device proof requires POSIX (Windows has no mkfifo)
PASS: H-3 oversized spec and store documents rejected explicitly
SKIP: H-4b read-only-directory preservation proof requires POSIX non-root
PASS: H-4a failed save is explicit, audited, and leaves no staged file
SKIP: H-5 symlink proof requires POSIX
PASS: H-6 successful re-saves install the complete state with no staged files

SKIPPED (not applicable on this platform):
  - H-2 FIFO/character-device blocking proof (needs POSIX mkfifo)
  - H-4b failed-save preservation on a read-only directory (needs POSIX non-root)
  - H-5 symlink-at-destination proof (needs POSIX symlink)

ALL FILESYSTEM LAYOUT CLI HARDENING PROOFS PASSED!
```

**Honest platform statement.** This host is Windows: it has no `mkfifo` (`os.name == "nt"`), and
symlink creation fails without privilege (`WinError 1314: A required privilege is not held by the
client`), so the H-2 FIFO/character-device group, the H-4b read-only-directory group, and the H-5
symlink group are **skipped here and execute in Linux CI**. That is a coverage gap on this host, not a
pass. Two mitigations keep the underlying properties covered locally:

- the equivalent core properties — rename-failure preservation, atomic replacement with no leftovers,
  and non-regular-file rejection with a read-time cap — are asserted by the Rust tests in §3.1, which
  **did** run on this host (including the rename-failure path, which fails identically on Windows);
- H-1's cross-platform case (a directory named by `--spec`, `--store`, and `--fstab`) exercises the
  same `is_file()` guard the FIFO and device paths rely on.

The suite reports skips explicitly and lists them, so a green run can never be mistaken for full
platform coverage.

What the executed groups assert:

- **H-1** — directory via `--spec` → `SPEC_NOT_REGULAR_FILE`, exit 1, exactly one audit row with
  `outcome_detail = "Spec path is not a regular file"`; via `--store` → `LOAD_STORE_FAILED`, exit 1,
  one row; via `--fstab` → `FSTAB_NOT_REGULAR_FILE`, exit 1, one row; a non-UTF-8 spec →
  `SPEC_READ_FAILED` naming UTF-8; no staged file left by any rejected read.
- **H-3** — a 10 MiB + 1 spec → `SPEC_SIZE_EXCEEDED`; a 10 MiB + 1 store → `LOAD_STORE_FAILED`; both
  within the bounded budget (the harness fails if the CLI does not return in 30 s).
- **H-4a** — a store path whose parent is a regular file → `SAVE_STORE_FAILED`, exit 1, exactly one
  audit row (`"Failed to persist layout store"`), and **no staged file** anywhere under the tree.
- **H-6** — two successive registers into one store: the destination is a complete, parseable store
  after *every* step, both custom layouts and both canonical presets are present, the active pointer is
  intact, and no staged file ever leaks.

### 3.3 Proof the assertions detect the defects

Both fixes were reverted, rebuilt, and re-run. Each reverted guard fails its test:

```
===== MUTATED (pre-hardening behaviour) =====
cargo test -p aiosh-core --test test_fs_layout_service
test test_fs_layout_reads_reject_non_regular_files_and_oversize_documents ... FAILED
test test_fs_layout_save_preserves_staged_file_when_rename_fails ... FAILED
test result: FAILED. 12 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out

panicked at tests/test_fs_layout_service.rs:382:
unexpected message: failed to read layout store file '...\store_is_a_dir.json': Access is denied. (os error 5)

$ python code/aiosh-cli/tests/test_fs_layout_hardening.py
AssertionError: {'code': 1, 'data': None, 'error': {'code': 'SPEC_READ_FAILED',
  'message': "failed to read spec file '...\\a_directory': Access is denied. (os error 5)"}}
```

With the `is_file()` guard removed, the directory is no longer refused by type: the CLI reports the
opaque `SPEC_READ_FAILED` / `Access is denied` instead of `SPEC_NOT_REGULAR_FILE`. With the
preservation behaviour removed, the staged file is deleted and its test fails. Both guards were then
restored (`is_file()` at `fs_layout_service.rs:103`, the recovery message at line 731) and everything
re-verified green.

---

## 4. Regression & Aggregate Verification

The FsLayout aggregate runner gained an **FL7** criterion for the new hardening suite:

```
$ python tools/test_fs_layout_suites.py
[+] FL1 filesystem layout data model integrity & invariants (FL1..FL5)
[+] FL2 filesystem layout core service (store, probe, diff, fstab, persistence)
[+] FL3 filesystem layout CLI surface smoke & boundaries (cmd_fs_layout)
[+] FL4 filesystem layout CLI audit emission & escape-injection security proof
[+] FL5 filesystem layout CLI in-tree unit test
[+] FL5 filesystem layout MCP in-tree unit test
[+] FL6 filesystem layout cross-surface CLI <-> MCP integration parity
[+] FL7 filesystem layout CLI hardening (non-regular paths, bounded reads, atomic persistence)

PASS: fs_layout_suites criteria (FL1..FL7)
```

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.47s

$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.96s

$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core
test result: ok. 372 passed; 0 failed; ... (lib) plus all integration targets green, incl.
test_fs_layout_service: 14 passed; 0 failed

$ python code/aiosh-mcp/tests/test_session_mcp_smoke.py
ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!

$ python code/aiosh-cli/tests/test_service_cli_smoke.py
ALL SERVICE CLI SMOKE TESTS PASSED!

$ python tools/check_task_docs.py
PASS: task docs criteria (C1..C6)

$ python tools/check_evidence.py
PASS: evidence integrity criteria (E1..E4)
```

---

## 5. Resource-Cleanup Audit

| Resource | Finding | Status |
|---|---|---|
| **Child processes** | The surface spawns none (verified by grep). | N/A — nothing to reap |
| **Audit/PEP DB connections** | `Ctx` owns the `AuditRing` (`rusqlite::Connection`, WAL) and the `PepStore`. Both are dropped when `cmd_fs_layout` returns, on every exit path including the early argument-error returns, so no connection outlives an invocation; no explicit `close()` is needed. | No leak |
| **Staged temp files** | Removed by the rename on success; removed when the write fails (partial content); **preserved with its path in the error** when only the rename fails. Asserted by H-4a and H-6 (no `.*.tmp.*` residue after rejection, failure, or success). | No leak on the error path |
| **Open file handles** | The staged handle is explicitly dropped before the rename (required on Windows, where an open handle would block the move), and closed on every branch of the write error path. | No leak |
| **Partial store visible to a later run** | Impossible: content only ever appears at the destination via a rename of a fully written, fsync'd file. | No |

---

## 6. Acceptance Verification

- [x] **Failure modes produce explicit, auditable errors.** Every new failure path returns the standard
      envelope with a specific code (`SPEC_NOT_REGULAR_FILE`, `FSTAB_NOT_REGULAR_FILE`,
      `SPEC_SIZE_EXCEEDED`, `FSTAB_SIZE_EXCEEDED`, `LOAD_STORE_FAILED`, `SAVE_STORE_FAILED`) and writes
      exactly one audit row; asserted against the real `audit_ring` for H-1 and H-4a.
- [x] **No temp/connection leaks on the error path.** Staged files are asserted absent after rejection,
      save failure, and success; DB handles are owned and dropped per invocation (§5).
- [x] **Timeouts / size caps / bounded retries.** No external processes exist to time out; reads are
      bounded at the stream level (cap enforced during the read, H-5); staged-file creation retries a
      bounded 8 times (H-3).
- [x] **Fail-open behaviour writes an honest audit row.** Extended in T-01527 and preserved here — every
      added rejection routes through `classify_and_emit` before returning.
- [x] Fixes proven through the real `aiosh` binary, with mutations demonstrating the assertions fail
      against the pre-hardening code (§3.3).

---

## 7. Handover to T-01529 (Documentation)

1. Document the new failure codes (`SPEC_NOT_REGULAR_FILE`, `FSTAB_NOT_REGULAR_FILE`) and the
   `--store`/`--spec`/`--fstab` regular-file requirement, including at least one copy-pasteable example.
2. Document the crash-recovery residue (§2, accepted risk 2): a failed rename prints the preserved
   staged path, and a crash can leave one inert hidden staged file beside the store.
3. State the platform limits honestly: the Linux-only read-time-cap proof and the POSIX-only FIFO,
   read-only-directory, and symlink groups skipped on Windows.
4. Link this file and `T-01528-hardening.md` from the operator documentation.
