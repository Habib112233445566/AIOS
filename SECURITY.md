# Security Policy — AI-Native Operating System (AIOS)

**Status:** ACTIVE (2026-08-22) · Governed by ledger task law ·
Rule-pack revision: `sprint-2-rule-pack-v1`

## Supported Surfaces

| Surface | Status |
|---|---|
| `code/aiosh-rust/` (MCP server + CLI + core) — current ledger pointer state | ✅ Supported |
| `code/aiosh-mcp/` Python MCP reference server | ✅ Supported (reference contract) |
| `code/aiosh-cli/` TypeScript CLI | ⚠️ Reference only |
| Frozen RISC-V microkernel substrate (`docs/research*`) | ❌ Not covered |

## Reporting a Vulnerability

**Report privately via GitHub Security Advisories:**

> https://github.com/Habib112233445566/AIOS/security/advisories/new

Please include: affected surface (which tool/command), reproduction
steps, expected vs actual behavior, and — if you have it — the audit
row id(s) from `aiosh audit tail` that show the misbehavior. Do NOT
open a public issue for an unpatched vulnerability.

## What Counts as a Vulnerability

- Bypassing or weakening the classifier → PEP → audit gate ordering
  (ADR-0035 §D-4), including any path that mutates state without
  exactly one honest audit row.
- Breaking the audit-ring hash chain, truncation detection, retention
  archival guarantees, or the no-skip task law.
- Sandbox escape or privilege escalation beyond a granted PEP scope.
- Prompt injection that flips a gate decision or forges audit/event
  content.
- Secret exposure through any supported surface.
- Escaping CI orchestrator process group timeouts or overflowing bounds to cause Denial of Service.

## Out of Scope

- The frozen RISC-V microkernel research substrate.
- Legacy TypeScript CLI internals beyond the reference contract.
- The Drive/R2 backup transport itself.
- Reports about the *difficulty* of ethical-hacking features — those
  are the product.

## Response & Disclosure Timeline

- **Acknowledgement:** within 7 days of your report.
- **Coordinated disclosure:** fixes and public disclosure within
  **90 days**, extended only by mutual agreement.
- You are credited by default; tell us if you prefer otherwise.

## Policy Governance

- The classifier rule pack is version-stamped
  (`CLASSIFIER_REVISION`, currently `sprint-2-rule-pack-v1`); ANY rule
  change requires a revision bump and is audited.
- This policy file changes only through the same sequential task law
  as code (ledger pointer).

## Security Knowledge Index

- Constitution: `mostimportanAIfolder/AI_CONSTITUTION.md`
- Component security reviews: `docs/tasks/evidence/T-00009.md`
  (retention) · `T-00017-security.md` (ledger data model) ·
  `T-00027-security.md` (core service MCP) · `T-00037-security.md`
  (CLI) · `T-00047-security.md` (Python MCP) · `T-00057-security.md`
  (configuration) • `T-00127-security.md` (ci data model) • `T-00147-security.md` (ci mcp) • `T-00157-security.md` (ci config) • `T-00167-security.md` (ci automated tests) • `T-00177-security.md` (ci security policy)
- Ledger spec & limitations: `docs/SPEC-TASK-LEDGER.md`
- Classifier formal spec: `docs/SPEC-CONSTITUTION-CLASSIFIER.md`
