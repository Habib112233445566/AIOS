# T-00097 — documentation component: Security Review (evidence)

**Date:** 2026-08-22 · **Scope:** `tools/check_task_docs.py` (+ its CI
invocation) · **Verdict:** **NO policy bypass.** Two robustness findings
queued to T-00098 hardening (neither crosses a trust boundary).

## S0 — Agent reachability & privilege

- Live wire probe: MCP `tools/list` = same 13 tools; **no docs tool is
  exposed** — the checker is operator/CI-only, unreachable from agents.
- Source scan: no `eval/exec/subprocess/os.system/__import__`.
- State changes: none possible (pure `Path.read_text` + `exists()`).
  Therefore **no PEP-gating or audit-row obligations attach** (ADR-0035
  §F-2 applies to consequential actions; there are none).

## S1 — Untrusted-content inertness

Doc text is only ever regex-scanned / exists()-probed. Fixture with
shell metacharacters, backticks and `${…}` flows through as inert bytes;
nothing interpolates into commands or paths used for anything but a
boolean existence check by the local operator.

## S3 — ReDoS (empirical timing)

| Hostile input | Pattern | Time |
|---|---|---|
| 80 KB nested `(())` link blob | `_MD_LINK` | 0.006 s |
| 60 KB deep path token | `_PATH_TOKEN` | 0.001 s |
| 20 KB phase row | `_PHASE_ROW` | 0.065 s |

No nested/ambiguous quantifiers → linear; no catastrophic backtracking.

## Findings (dispositioned)

- **F1 — silent pass on external link targets.** `[x](/etc/passwd)`,
  `[x](../../etc/hostname)` and a symlink escaping the tree are all
  treated as "resolved" (probe reproduced: ok=True, empty detail).
  Impact: cosmetic false-green in a local report; no disclosure (the
  operator already owns the filesystem). **Disposition:** flag such
  targets instead of silently passing → implemented in T-00098.
- **F2 — unbounded file reads.** A 50 MiB SPEC was read whole without
  complaint (probe reproduced). A pathological multi-GB doc could spike
  memory. **Disposition:** 16 MiB cap with loud named failure →
  implemented in T-00098.

## Blocking notes

None required at this task: F1/F2 are hardening of an operator-only,
read-only tool, not policy bypasses. Both are closed in T-00098 with
regression tests (U18/U19 added there via the U-suite).
