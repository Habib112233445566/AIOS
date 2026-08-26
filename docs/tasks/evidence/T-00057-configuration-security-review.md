# T-00057 — Task Ledger Control configuration: Security Review

**Date:** 2026-08-22
**Type:** security review (no code changed)
**Depends on:** T-00056 integration
**Scope:** the `AIOSH_LEDGER_*` env-config layer (`ledger_config.rs`,
consumers in ledger/task_service/CLI, Python mirrors).

All findings verified empirically on scratch sandboxes.

## 1. Verified controls

| # | Control | Empirical result |
|---|---|---|
| S1 | **Trust boundary.** Config comes from OPERATOR env vars; agents reach the system only through CLI args / MCP tool args and cannot set environment variables. No MCP tool exposes config mutation (D5) — knobs are not agent-writable. | PASS (by construction + surface audit) |
| S2 | **Loud, named misconfiguration.** Non-numeric (`soon`), negative (`-5`), hex (`0x10`) all refused with `invalid AIOSH_LEDGER_<NAME>='<raw>': …`; whitespace is trimmed before parse (`" 5000 "` accepted); every refusal exits 1 through the standard envelope with an honest audit row. | PASS |
| S3 | **Floors prevent self-bricking.** Zero/near-zero byte caps and text caps below 64 are refused at the floor — an operator cannot accidentally disable the size-cap defense entirely. | PASS |
| S4 | **u64 extremes.** `AIOSH_LEDGER_LOCK_TIMEOUT_SECS=18446744073709551615` exercised through a MUTATING command (`task rebuild`, which acquires the lock): clean execution, no panic/overflow on this platform (rustc 1.98, Linux). Semantic = operator-chosen effectively-unbounded wait, identical to pre-T-28 behavior; documented rather than clamped. | PASS (with note) |
| S5 | **Forward compatibility.** Unknown `AIOSH_LEDGER_*` variables are ignored (documented), so future knobs cannot be weaponized against older binaries. | PASS |
| S6 | **No new gate bypass.** Config affects sizes/timeouts only — never grants, classifier verdicts, or the no-skip law. The PEP/classifier/audit ordering is untouched. | PASS |

## 2. Abuse scenarios → dispositions

| Scenario | Disposition |
|---|---|
| Agent escalates by raising caps/timeout | Impossible: env not settable by agents (S1); MCP has no config tool (D5) |
| Operator disables protections via tiny caps | Floors block zeroing (S3); anything else is explicit operator intent, visible via `aiosh task config` sources |
| Extreme values crash the binary | u64::MAX probed through lock acquisition path — no panic (S4); unparseable values fail loudly BEFORE touching state |
| Silent misconfiguration | D3 forbids fallbacks: every bad value names itself at first use |

## 3. Notes

- Platform caveat on S4 recorded honestly: saturation behavior is
  std/platform-dependent; if a future toolchain panics there, the fix
  is a one-line clamp in `acquire_lock` — tracked here rather than
  silently ignored.
- Residual: none open.

## 4. Verdict

**No known policy bypass remains open.**

Acceptance:
- [x] Security evidence file with abuse scenarios.
- [x] No known policy bypass remains open.
