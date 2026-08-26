# T-00108 — Task Ledger Control / recovery & validation: Hardening

Date: 2026-08-23 · Status: HARDENING COMPLETE

## Finding F-1 closed (from T-00107 security review)

**Evidence existence oracle.** Event-supplied evidence strings could point
at ANY absolute path on disk; if the target existed, validate emitted no
finding (silent attestation evasion). Fixed identically on both
substrates:

| File | Change |
|---|---|
| `tools/task_ledger.py` (validate_state) | a path is satisfiable ONLY if relative and contains no `..` component; absolute or escaping strings are classified **missing (suspicious)**; non-string entries also classified missing |
| `aiosh-core/src/ledger.rs` (validate_state) | same rule (`starts_with('/') ‖ split('/').any(=="..")`) |

Existence checks still read nothing (boolean stat only) — unchanged.

## Cross-substrate parity hardening

Probe exposed a cosmetic-but-real parity break in the drift `detail`
prose: Rust `Value` Display renders `[1,2,3]`, Python `repr` rendered
`[1, 2, 3]`. Python now renders live/replay values via compact JSON
(`separators=(',',':')`), making the ENTIRE findings payload byte-equal
between Rust MCP wire and Python MCP (verified post-fix on a hostile-path
sandbox).

## Resource-safety confirmation (per task checklist)

- Timeouts/caps/bounded retries: validate performs no subprocesses; all
  file reads inherit the `AIOSH_LEDGER_MAX_{STATE,EVENTS,LEDGER}_BYTES`
  caps and bounded-flock config already shipped (T-00018/T-00028/T-00054).
  No new unbounded input paths introduced.
- Standard envelope: every failure surfaces as `{ok:false,error}` or a
  fatal finding — never silent (pinned by V7).
- Resource cleanup: opens are context-managed / RAII; no temp files or DB
  handles created by validate (report-only by contract).
- Fail-open-with-audit: N/A to validate itself (no external failure mode
  can bypass the gate); refusals remain audited (S2/S3 pins).

## New regression pin

`tools/test_task_validate.py` V9: `/etc/passwd` (absolute, exists),
`../../../../etc/shadow` (escaping), and a legit-shaped but nonexistent
relative path ALL land in `missing`; drift check stays ok. Suite now
V1..V9.

## Verification

- `python3 tools/test_task_validate.py` → PASS V1..V9.
- `cargo test` → all targets ok (82 tests); zero-warning build.
- U-suite U1..U16 PASS; P/W/metrics suites PASS.
- Hostile-path cross-substrate probe → **byte-parity TRUE** after fixes.
