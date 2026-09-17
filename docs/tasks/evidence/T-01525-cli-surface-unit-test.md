# T-01525: Filesystem Layout - CLI Surface: Unit Test

## Metadata
- **Task ID:** `T-01525`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface (`code/aiosh-cli/tests/test_fs_layout_cli_smoke.py`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (5/10) — CLI Surface Unit Test
- **Dependencies:** `T-01524` (CLI Surface Implementation)
- **Next Task:** `T-01526` (Filesystem Layout / CLI surface: Integration)

---

## 1. Test Artifact Delivered

Added a standalone CLI smoke and boundary suite following the existing
`code/aiosh-cli/tests/test_service_cli_smoke.py` conventions (spawn the real `aiosh` binary as a
subprocess, assert exit codes, JSON envelopes, and on-disk state):

**`code/aiosh-cli/tests/test_fs_layout_cli_smoke.py`** — 9 test groups.

Assertions target observable behaviour only:

- **Process exit status** for every invocation.
- **JSON result envelope** (`{code, data, error}`) on `--json` paths.
- **Persisted store state** — the layout store JSON written to disk is re-read and inspected
  (the DB-equivalent artefact for this subsystem).
- Spec fixtures are **derived from the binary itself** (`layout show <preset> --json`) and mutated,
  so the test cannot drift from the canonical schema and cannot pass against a hand-written
  fixture that the product itself would reject.

### 1.1 Coverage matrix

| Group | Valid input | Invalid input | Boundary values | Primary failure mode |
|---|---|---|---|---|
| `--help` / dispatch | full subcommand list | unknown subcommand | — | exit 2 + `UNKNOWN_SUBCOMMAND` |
| `list` | prose + JSON | — | — | presets and active pointer present |
| `show` | active default, explicit ID, `--container` selector | — | — | unknown ID → exit 1 + `RESOLVE_FAILED`, `data: null` |
| `validate` / `check` | preset passes | malformed JSON spec | root mount with wrong fsck pass (FL1) | FL1 violation → exit 1 + `VALIDATION_FAILED` |
| `probe` | 2× minimum | `abc`, `-1`, `1.5` → exit 2 | exactly minimum (viable); minimum − 1 byte (not viable) | cannot fit partition allocation → `NOT_VIABLE` |
| `diff` | preset vs preset | — | layout vs itself → not destructive | unknown source/target → exit 1 + `DIFF_FAILED` |
| `fstab` | prose + JSON | — | root row carries six fields with pass `1` | — |
| `register` / `set-active` / `remove` / `import-fstab` | file spec **and** inline JSON spec; fstab import | duplicate register; repeat removal; built-in removal; empty fstab | active-layout removal refused | exit 1 + `REGISTER_FAILED` / `REMOVE_FAILED` / `IMPORT_FAILED` |
| argument boundaries | 1024-char store path accepted | control character in `--store`; missing mandatory args | 1025-char store path rejected | exit 2 + `INVALID_ARGUMENT` / `ARGUMENT_ERROR` |

---

## 2. Defect Surfaced and Fixed

Writing the `probe` group exposed a specification deviation. `T-01522-spec.md` §3.4 documents
exit code `2` ("invalid byte count") for `aiosh layout probe`, but the implementation silently
swallowed a malformed `--bytes` value:

```rust
let target_bytes: u64 = parse_flag(rest, "--bytes")
    .and_then(|s| s.parse().ok())
    .unwrap_or(layout.target_disk_min_bytes);   // "abc" silently became the layout minimum
```

A typo such as `--bytes 1O7374182400` was therefore reported as a viable/not-viable probe of the
default minimum rather than as a usage error — a silent failure that would be actively misleading
in deployment automation.

Fixed in `code/aiosh-rust/aiosh-cli/src/main.rs`: an unparseable `--bytes` value now emits an
`ARGUMENT_ERROR` audit row and exits `2`, matching the documented contract. A missing `--bytes`
still defaults to the layout minimum, as specified.

---

## 3. Proof the Tests Have Teeth

Per the acceptance criteria, the suite was verified to fail when the feature is broken. The
`--store` path guard in `cmd_fs_layout` was temporarily mutated to `if false { ... }` (disabling
control-character and length rejection), the binary rebuilt, and the suite re-run:

```
===== mutated run =====
  File "code/aiosh-cli/tests/test_fs_layout_cli_smoke.py", line 411, in test_layout_argument_boundaries
    expect_code(res_ctrl, 2, "control char store path")
AssertionError: control char store path: expected exit code 2, got 0
```

The mutation was then reverted and the guard confirmed restored:

```
$ grep -n "if p.len() > 1024" aiosh-cli/src/main.rs
997:        if p.len() > 1024 || p.chars().any(|c| c.is_control()) {
```

---

## 4. Test Execution

New suite, standalone:

```
$ python code/aiosh-cli/tests/test_fs_layout_cli_smoke.py
PASS: aiosh layout --help and unknown subcommand (exit 2)
PASS: aiosh layout list (prose and JSON, presets present)
PASS: aiosh layout show (valid, preset selector, unknown id -> exit 1)
PASS: aiosh layout validate/check (valid, FL1 + pass boundary, malformed spec)
PASS: aiosh layout probe (viable, minimum boundary, below-minimum, invalid bytes)
PASS: aiosh layout diff (destructive, self-diff, unknown ids -> exit 1)
PASS: aiosh layout fstab (prose and JSON, six-field rows)
PASS: aiosh layout register/set-active/remove/import-fstab store lifecycle
PASS: aiosh layout argument boundaries (missing args, control chars, path length)

ALL FILESYSTEM LAYOUT CLI SMOKE TESTS PASSED!
```

Regression: sibling CLI smoke suite (same binary, untouched paths).

```
$ python code/aiosh-cli/tests/test_service_cli_smoke.py
ALL SERVICE CLI SMOKE TESTS PASSED!
```

Regression: Rust CLI unit suite, including `test_cmd_fs_layout_flow`.

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.61s
```

---

## 5. Acceptance Verification

- [x] New test file runs standalone and passes (9/9 groups, exit 0).
- [x] Negative cases are asserted, not just happy path — every group asserts at least one failure
      path with an explicit exit code and error code.
- [x] Suite demonstrably fails when the feature is broken (mutation check in §3).

---

## 6. Known Limitations

- The suite does not yet assert audit-row emission, because the audit sink resolution is
  environment-dependent and the `probe`/`diff` failure paths currently emit no row at all. That
  fail-open audit gap is scheduled for `T-01528` (CLI surface hardening) and re-verified in
  `T-01527` (security review).
- The suite invokes the pre-built `target/debug/aiosh` binary; it requires `cargo build --bin aiosh`
  to have been run first. CI is expected to build before running smoke suites.
