# T-01527: Filesystem Layout - CLI Surface: Security Review

## Metadata
- **Task ID:** `T-01527`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface (`code/aiosh-rust/aiosh-cli::cmd_fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (7/10) — CLI Surface Security Review
- **Dependencies:** `T-01526` (CLI Surface Integration)
- **Next Task:** `T-01528` (Filesystem Layout / CLI surface: Hardening)

---

## 1. Scope

Reviewed the full operator surface: `cmd_fs_layout` (dispatch, argument parsing, store-path
validation, output/encoding) and the four mutating subcommands `cmd_fs_layout_register`,
`cmd_fs_layout_set_active`, `cmd_fs_layout_remove`, `cmd_fs_layout_import_fstab`.

Attack surface considered: argument parsing and flag smuggling, store-path validation, file reads
(`--spec`, `--fstab`, `--store`), output/encoding (human and `--json`), audit-row emission, and
resource bounds.

---

## 2. Findings and Fixes

### F-1 — Fail-open audit gap (**HIGH**, fixed)

**Vector.** `probe`, `diff` and the unknown-subcommand path returned a failure exit without writing
an audit row, violating ADR-0035 §F-2 ("fail-open behaviour must always write an honest audit row")
and the invariant that consequential actions write exactly one row.

Specifically unaudited before this task:

| Path | Condition |
|---|---|
| `aiosh layout probe` | layout/store resolution failure |
| `aiosh layout probe` | `probe_target` failure |
| `aiosh layout diff` | store load failure |
| `aiosh layout diff` | `diff_layouts` failure |
| `aiosh layout <unknown>` | unknown subcommand |

**Impact.** An operator or automation wrapper could not distinguish "rejected and recorded" from
"rejected and invisible"; repeated probing of non-existent layouts and store-load failures left no
audit trail at all, defeating tamper-evident review of the surface.

**Fix.** `classify_and_emit` is now called on each of those paths before returning, with a
descriptive `outcome_detail` (`"Failed to resolve layout for probe"`, `"Failed to compute layout
diff"`, `"Failed to load layout store for diff"`, `"Unknown filesystem layout subcommand"`, …).

**Secondary defect found while fixing F-1.** `probe` silently discarded a validation failure:

```rust
let _ = service.store_mut().register_layout(layout);   // error swallowed
```

A rejected layout degraded into a misleading generic `PROBE_FAILED`/`not found` error. It now fails
loudly with `INVALID_LAYOUT` and an audit row recording that FL1..FL5 rejected the spec.

### F-2 — Terminal escape-sequence injection (**MEDIUM**, fixed)

**Vector (CWE-150).** Free-text layout fields are not charset-restricted by the layout validators
(by design — `name`, `description`, `created_at` and `partition.label` are length-bounded only).
Layout specs may be authored outside the operator's trust boundary (shared profile, agent-written
store, imported fstab). Those fields were rendered verbatim to the terminal by `show`, `list` and
`fstab`, so a spec author could emit arbitrary ANSI/OSC sequences into the operator's terminal —
spoofed output, screen clearing, and OSC 52 clipboard writes.

**Impact.** Requires the operator to inspect an attacker-influenced layout; impact is confined to the
rendering terminal (no privilege escalation), but terminal manipulation is a recognised
social-engineering and data-exfiltration vector.

**Fix.** Added `sanitize_terminal()` and applied it to every human-readable site that can carry
spec- or argv-derived text: `show` (name, id, description, partition labels, mount device/path),
`list` (active id, id, name), `fstab` (whole generated content, covering `created_at`), `validate`
(id, violation message), `probe` (id, errors, warnings), `diff` (summary), and the error echoes in
all four mutating subcommands. Control characters become `U+FFFD` — the same substitution the argv
boundary already performs — so the attack text stays visible instead of silently disappearing.

Critically, `--json` output and the canonical store are **not** sanitized: serde escapes control
characters correctly, and the data model must stay faithful. This is explicitly asserted by the
proof suite.

### F-3 — Accepted risks (documented, no fix)

1. **Operator-supplied file paths (`--spec`, `--fstab`, `--store`).** These are local paths the
   operator chooses; no content-disclosure path was found (serde errors echo field *names* from the
   payload, which are now sanitized, never file contents). Reads are bounded to 10 MiB for files.
   An operator can already read/write any path their own account can; no privilege boundary is
   crossed. Accepted.
2. **Inline `--spec` / `--fstab` size.** Not separately capped; the OS bounds a single argument well
   below the documented 10 MiB ceiling (Linux `MAX_ARG_STRLEN` 128 KiB; Windows ~32 KiB command
   line), so the documented bound is the binding constraint for the file form. Accepted.
3. **Atomic-write temp path.** `save_to_path` uses a predictable `.tmp.<pid>` sibling. It unlinks any
   pre-existing temp path *before* writing, which defeats the symlink-clobber variant, and it
   requires write access to the store directory (already a privileged position). A residual narrow
   TOCTOU window is accepted; hardening to `O_EXCL` is noted for `T-01528`.
4. **No PEP grant requirement on the CLI.** The CLI is an operator surface acting as the `operator`
   actor, matching every sibling subsystem's CLI. Machine-initiated calls go through the PEP-gated
   MCP surface instead. The CLI's control is the hash-chained audit ring plus classifier flags.
5. **`--help` is intentionally unaudited** — it has no side effects and no state change, consistent
   with sibling surfaces. Every non-help execution path is audited.

**No known policy bypass remains open.**

---

## 3. Proof Through the Real Binary

New standalone proof suite: **`code/aiosh-cli/tests/test_fs_layout_audit_security.py`**. It isolates
the audit ring with `AIOSH_HOME=<tempdir>` and inspects the real SQLite `audit_ring` table, so
assertions are against genuine persisted audit state rather than in-process stubs.

```
$ python code/aiosh-cli/tests/test_fs_layout_audit_security.py
=== RUNNING FILESYSTEM LAYOUT AUDIT & INJECTION SECURITY PROOF ===
PASS: F-1 successful path emits exactly one audit row
PASS: F-1 probe resolve failure is audited
PASS: F-1 probe invalid --bytes is audited
PASS: F-1 diff failure paths (diff + store load) are audited
PASS: F-1 unknown subcommand is audited
PASS: F-1 all 8 sampled failure paths emit exactly one audit row
PASS: F-2 spec-derived fields neutralized on render, faithful in JSON and store
PASS: F-2 argv-derived echoes neutralized on the human path

ALL FILESYSTEM LAYOUT AUDIT & INJECTION SECURITY PROOFS PASSED!
```

### 3.1 Proof the F-1 assertions detect the original defect

The `probe` resolve path was temporarily reverted to its pre-fix shape (no `classify_and_emit`),
rebuilt, and the suite re-run:

```
===== MUTATED (pre-fix probe) run =====
PASS: F-1 successful path emits exactly one audit row
AssertionError: probe resolve failure must emit exactly 1 row, got []
```

The pre-fix behaviour emits **zero** audit rows and the suite detects it. The source was then
restored and re-verified green.

### 3.2 Escape-injection assertions

The `F-2` group registers a layout whose `name`, `description`, `created_at` and first partition
`label` carry `ESC[31m`, `ESC[2J` and an OSC 52 clipboard payload, then asserts:

- `layout show` / `layout list` / `layout fstab` (human) contain no raw `ESC` or `BEL`, while the
  attack text itself (`PWNED`) remains visible as neutralised content;
- `layout show --json` still contains the raw escape sequences (no lossy sanitization of the
  machine contract);
- the on-disk store JSON still contains the raw escape sequences (sanitization is a rendering
  concern, not a storage transformation).

---

## 4. Regression & Aggregate Verification

Added **`tools/test_fs_layout_suites.py`**, the aggregate runner every sibling subsystem already had
(the Filesystem Layout epic previously had none — verification was spread across ad-hoc commands):

```
$ python tools/test_fs_layout_suites.py
[+] FL1 filesystem layout data model integrity & invariants (FL1..FL5)
[+] FL2 filesystem layout core service (store, probe, diff, fstab, persistence)
[+] FL3 filesystem layout CLI surface smoke & boundaries (cmd_fs_layout)
[+] FL4 filesystem layout CLI audit emission & escape-injection security proof
[+] FL5 filesystem layout CLI in-tree unit test
[+] FL5 filesystem layout MCP in-tree unit test
[+] FL6 filesystem layout cross-surface CLI <-> MCP integration parity

PASS: fs_layout_suites criteria (FL1..FL6)
```

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.76s

$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.37s

$ python code/aiosh-mcp/tests/test_session_mcp_smoke.py
ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!

$ python tools/check_task_docs.py
PASS: task docs criteria (C1..C6)

$ python tools/check_evidence.py
PASS: evidence integrity criteria (E1..E4)
```

---

## 5. Acceptance Verification

- [x] Security evidence file exists with abuse scenarios (F-1, F-2) plus documented accepted risks.
- [x] No known policy bypass remains open — every non-help execution path emits exactly one audit
      row, and no unsanitized spec/argv-derived text reaches the terminal.
- [x] Fixes proven through the real `aiosh` binary against the real audit ring, with a mutation check
      demonstrating the assertions fail against the pre-fix code.

---

## 6. Handover to T-01528 (Hardening)

1. Replace the predictable `.tmp.<pid>` temp path with `OpenOptions::new().create_new(true)`
   (`O_EXCL`) to remove the residual TOCTOU window in the store save path.
2. Consider whether the layout validators themselves should reject control characters in `name`,
   `description`, `created_at` and `partition.label`, which would protect every present and future
   rendering surface (MCP responses included) rather than only the CLI terminal path. This is a
   data-model change and belongs to that subsystem's review, not this one.
3. The same `sanitize_terminal` treatment is applicable to sibling CLI surfaces
   (`session`, `service`, `package`, …), which share the identical rendering pattern. Out of scope
   here; noted as a cross-cutting follow-up.
