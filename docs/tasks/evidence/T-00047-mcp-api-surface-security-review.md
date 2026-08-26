# T-00047 — Task Ledger Control MCP/API surface: Security Review

**Date:** 2026-08-22
**Type:** security review (no code changed)
**Depends on:** T-00046 integration
**Scope:** the Python reference substrate's `aios_task` surface
(`server.py::_validate_task_args/_run_task_action/aios_task`,
`_dispatch` gate, `task_ledger` module). Shared Rust-side findings
from T-00027 carry over; this review covers the port.

All findings verified empirically on a scratch sandbox (real grant
minted via the Rust CLI, real tool invocations).

## 1. Verified controls

| # | Control | Empirical result |
|---|---|---|
| S1 | **Scope isolation.** A grant scoped `tools=["aios.pentest.*"]` does NOT authorize `aios.task` on the Python server — refused at `gate:"pep"`. | PASS |
| S2 | **Hostile content containment.** Note with quotes/CRLF/tab/backslash/`<script>`/unicode + evidence `../../etc/shadow` stored verbatim-but-JSON-escaped; round-trip byte-exact; evidence dir received only the task-id stub. | PASS |
| S3 | **Flood caps.** 20 evidence items refused ("exceeds 16 items"); text caps from §2 enforced pre-gate. | PASS |
| S4 | **Grant-gating of consequential set.** `rebuild`/`skip` without grant → `gate:"pep"` refusal, audited (P6/P-suite); read-only status/check open by design (D1 parity). | PASS |
| S5 | **State integrity after abuse.** `task check` invariants clean; `TASK_STATE.json` completed-set exact; last audit row outcome consistent with the last action. | PASS |
| S6 | **Pre-gate validation ordering.** Structural/semantic violations return BEFORE classifier/PEP/audit interaction — no audit-row spam from malformed input (mirrors Rust `-32602` channel; envelope-form here per FastMCP conventions). | PASS (P3/P4/P5) |

## 2. Abuse scenarios → dispositions

| Scenario | Disposition |
|---|---|
| Grant confusion across tool namespaces | Glob scoping proven (S1) |
| Ledger-file poisoning via note/evidence | Inert storage (S2); readers gated |
| Unauthenticated state rewrite via `rebuild` | Was a REAL HOLE in the first port (read-only misclassification) — caught by P6 failing-test-first in T-00045, fixed before this review; suite now pins it permanently |
| Module-import path manipulation (`tools/task_ledger.py` location) | Path derived from package location (`parents[3]`), not user input; operator env remains the trust boundary |
| DoS via giant lists/strings | Caps S3; bounded lock (T-28 shared) |

## 3. Verdict

**No known policy bypass remains open** on the Python MCP surface.

Acceptance:
- [x] Security evidence file with abuse scenarios.
- [x] No known policy bypass remains open.
