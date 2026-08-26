# T-00109 — Task Ledger Control / recovery & validation: Documentation

Date: 2026-08-23 · Status: DOCUMENTATION COMPLETE

## What shipped (operator view)

`validate` — the read-only ledger integrity action — is now documented in
the governing spec and the docs index:

1. **`docs/SPEC-TASK-LEDGER.md`**
   - §8 grant table: `validate` listed among grant-free read-only actions.
   - NEW **§9 "Recovery & validation — `task validate`"**: what it does,
     report-only design statement, check/severity table, hardening note
     (evidence-path confinement from T-00108), copy-pasteable invocations
     for all three surfaces (Rust CLI, Python reference CLI, MCP wire call),
     trimmed example output JSON, known limitations, and links to ALL eight
     evidence artifacts T-00101..T-00108.
   - L4 amended: evidence-existence is now machine-checked; content-vs-
     acceptance remains out of scope (honest residual).
2. **`docs/README.md`** task-ledger paragraph: read-only set updated to
   include `validate`, one-line pointer to SPEC §9.

## Copy-pasteable examples (verified live during this epic)

```bash
aiosh task validate                      # Rust ship path
python3 tools/task_ledger.py validate    # Python reference
# MCP: {"name":"aios.task","arguments":{"action":"validate"}}
```

## Constraints & limitations recorded (not omitted)

- Report-only; `rebuild` stays the sole repair path (lock-free by design).
- Evidence existence checked; content-vs-acceptance NOT validated (L4).
- Corrupt event line ⇒ loud error via shared reader, never partial findings.
- Findings are advisory; severity model fatal-vs-warning per SPEC §9 table.

## Invariant compliance

`tools/check_task_docs.py` → C1..C6 all green after edits (no TODO markers,
no volatile CI counts, every backticked path resolves); docs test suite PASS.
