# T-00072 — Task Ledger Control security policy: Specification

**Date:** 2026-08-22
**Type:** specification (no code changed)
**Depends on:** T-00071 research
**Status:** SPECIFIED — D1 RESOLVED by project owner (2026-08-22):
reporting channel = **`https://github.com/Habib112233445566/AIOS/security/advisories/new`**
(GitHub private Security Advisory). D2–D5 locked to research defaults.

## 1. Deliverable

Root-level `SECURITY.md` (repo root = `AIOS_MERGED/`), meeting OpenSSF
Scorecard Security-Policy criteria (E1–E3, fetched 2026-08-22):

1. **Linking (6/10):** the advisory URL above, verbatim, as the
   reporting route.
2. **Free-form text (3/10):** full prose — no bare-links-only file.
3. **Specific text (1/10):** ≥2 case-insensitive hits across
   `vuln*`/`disclos*` + explicit day-count timelines.

## 2. Content contract (sections)

| Section | Content (source of truth) |
|---|---|
| Scope — what is a vulnerability | PEP/classifier bypass; audit-chain break or truncation; sandbox escape; no-skip-law violation; prompt-injection that flips a gate decision; secret exposure (derived from component reviews T-09/17/27/37/47/57) |
| Out of scope | The frozen RISC-V microkernel substrate; legacy TS/Python EXCEPT the Python MCP reference server; the R2/Drive backup transport |
| Supported surfaces | Rust ship stack (aiosh-rust) current ledger pointer state; Python MCP reference server |
| How to report | GitHub Security Advisory URL (private by design); what to include (affected surface, reproduction, audit row id if any) |
| Response & disclosure | Owner responds ≤ 7 days; coordinated disclosure ≤ 90 days; credit by default unless reporter opts out |
| Policy governance | Rule-pack changes require `CLASSIFIER_REVISION` bump (existing governance); policy file changes are ledger tasks like any code |
| Security knowledge index | Links to the seven review artifacts + SPEC-TASK-LEDGER + AI_CONSTITUTION |

## 3. Reused vs new

Reused: all facts/links from in-tree docs (no new claims). New: one
file + README pointer. No dependencies.

## 4. Acceptance criteria (for T-00075 verification)

- File exists at root; contains advisory URL; ≥2 `vuln`/`disclos` hits;
  ≥1 day-count; no fabricated emails/URLs; every linked path exists in
  tree.
