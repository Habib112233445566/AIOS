# T-00071 — Task Ledger Control: security policy Research

**Date:** 2026-08-22
**Type:** research (no code changed)
**Depends on:** T-00070
**Artifact note:** instruction name `T-00071-research.md`; ledger row
declares `T-00071-security-policy-research.md` (mirrored).

Central question: what does a "security policy" component owe a
repository whose security knowledge exists but is SCATTERED?

## 1. Internal facts

| # | Fact | Anchor |
|---|---|---|
| F1 | There is NO `SECURITY.md`, no `.well-known/security.txt`, and no documented vulnerability-reporting channel anywhere in the tree | ls + grep, 2026-08-22 |
| F2 | Deep security knowledge EXISTS but is scattered across seven component security reviews (T-00009 retention, T-00017 ledger data model, T-00027 core service MCP, T-00037 CLI, T-00047 Python MCP, T-00057 config), the Constitution (P-principles), ADRs, and the rule-pack (`CLASSIFIER_REVISION = "sprint-2-rule-pack-v1"` with bump-on-change governance) | evidence dir, classifier.rs |
| F3 | Supported-surface fact: ONLY the Rust stack ships; legacy TS/Python are reference substrates (except Python MCP kept for cross-substrate contract) — a policy must say what IS covered | README v2.1, SPEC-TASK-LEDGER §7 |
| F4 | No public email/contact for the sole maintainer exists in-tree; inventing one would be fabrication | absence verified |

## 2. External authoritative facts (fetched live 2026-08-22)

Source: OpenSSF Scorecard checks doc — Security-Policy check,
<https://github.com/ossf/scorecard/blob/main/docs/checks.md#security-policy>

| # | Fact |
|---|---|
| E1 | Policy file = `SECURITY.md` (case-insensitive) in repo root (or well-known dirs) |
| E2 | Scoring: linking requirement (valid email OR http(s) reporting address) 6/10; free-form text beyond links 3/10; specific text on disclosure practices/timelines with ≥2 hits matching `vuln`/`disclos`/day-counts 1/10 |
| E3 | Content guidance: what constitutes a vulnerability + secure reporting route; follow coordinated-vulnerability-disclosure guidelines (oss-vulnerability-guide) |

## 3. Gap analysis

The project can *respond* to security concerns (reviews, gates, audit
ring) but cannot be *told* about one: no reporter knows where to send
findings, what's in scope, or the timeline. Candidate:

A (recommended): root `SECURITY.md` meeting E1–E3 exactly — scope
(what counts: PEP/classifier bypass, audit-chain break, sandbox escape,
no-skip violation, prompt-injection that flips a gate), reporting
channel, supported surfaces (F3), 90-day coordinated-disclosure
commitment, and an index linking the seven review artifacts (F2).
Rejected alternatives: security.txt only (E1 prefers root
SECURITY.md); doing nothing.

## 4. Assumptions (marked)

A1: reporting will ultimately flow through the project owner; until a
real channel exists, the file carries an explicit OWNER-ACTION
placeholder rather than a fabricated address (fabricating a contact =
hallucination, forbidden by project law).

## 5. Decisions needed before Specification (T-00072)

- **D1 (OWNER INPUT REQUIRED):** the actual reporting channel —
  private GitHub Security Advisory URL and/or owner email. Spec ships
  with a clearly marked `<OWNER-ACTION>` placeholder if unanswered.
- D2: supported scope statement = Rust ship stack + reference
  substrates as noted (default yes).
- D3: disclosure timeline = 90 days coordinated (default yes).
- D4: include index linking all seven component reviews (default yes).
- D5: document rule-pack revision-bump governance inside the policy
  (default yes).

## 6. Acceptance check
- [x] Facts vs assumptions separated; external criteria cited w/ date.
- [x] Decisions listed explicitly (D1 flagged OWNER-INPUT).
- [x] No code changed.
