# T-00058 — Task Ledger Control configuration: Hardening

**Date:** 2026-08-22
**Type:** hardening (ceiling added, rust+py)
**Depends on:** T-00057 security review

## Gap found and fixed

**Lock-timeout ceiling.** T-00057 S4 recorded a platform caveat:
`AIOSH_LEDGER_LOCK_TIMEOUT_SECS` accepted u64::MAX (empirically safe on
rustc 1.98/Linux, but semantics = operator-typo → effectively-infinite
wait, and std arithmetic behavior is toolchain-dependent). Closed by
construction: explicit **ceiling 86400 s (24 h)** — values above fail
loudly naming the variable and the bound, in BOTH substrates.

## Verified-not-added
Text/evidence caps already floored; byte caps unbounded above is
intentional (a cap only matters if the file is actually that large);
loader/lock bounded (T-28); error paths audited.

## Tests + verification

```
$ cargo test -p aiosh-core ledger_config → includes new ceiling case
   ("86401" → must be <= 86400)
$ live probes: rust AND python both refuse 99999999 with the ceiling msg
$ cargo test (workspace) → 80 passed; 0 failed (0 warnings)
```

Python mirror carries the same ceiling (parity law).

## Acceptance check
- [x] Failure modes produce explicit, auditable errors.
- [x] No leaks introduced; prior suites re-run green via CI in T-60.
