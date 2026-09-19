# T-01567 — Filesystem Layout security policy: Security Review

## Metadata
- **Task ID:** `T-01567`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** In Progress — security review initiated for the Filesystem Layout security policy across MCP tools, PEP gating, path containment, and audit logging.
- **Date:** 2026-09-19
- **Depends on:** `T-01566` (Security Policy Integration)
- **Feeds:** `T-01568` (Security Policy Hardening)
- **Artifacts:** `docs/tasks/evidence/T-01567-security-policy-security-review.md`, `docs/tasks/evidence/T-01567-security.md`

---

## 1. Security Review Scope & Objectives
Review the security policy implementation governing the Filesystem Layout subsystem:
- Input validation and schema conformance across all 10 MCP arms.
- Enforcement of Policy Enforcement Point (PEP) gating for all state-mutating operations (`register`, `set_active`, `remove`, `import_fstab`).
- Path confinement and canonical evasion resistance (8.3 aliases, case variations, trailing separators, device/extended namespaces `\\?\`, `\\.\`).
- Audit logging fidelity and tamper resistance via SQLite WAL hash chaining.
- Prompt-injection resistance in nested structures (e.g. `layout` object in `register`).

---

## 2. Abuse Scenarios Analyzed

### Scenario A-1: Unauthenticated Mutation Attempt
- **Vector**: Caller invokes `aios.fs_layout.remove` or `aios.fs_layout.register` without passing `grant_id`.
- **Expected Defense**: PEP gate intercepts request prior to any disk I/O, returns `ok: false, gate: "pep"`, emits an audit row with `outcome="refused"`.
- **Verdict**: Mitigated — tested by FL11 P1.

### Scenario A-2: Cross-Tool Scope Privilege Escalation
- **Vector**: Caller possesses a valid PEP grant scoped to another domain (e.g. `pentest.*`) and attempts to invoke `aios.fs_layout.register`.
- **Expected Defense**: PEP gate verifies tool glob against requested tool; fails closed if pattern does not match `aios.fs_layout.*` or the specific tool name.
- **Verdict**: Mitigated — tested by FL11 P2.

### Scenario A-3: Path Confinement Bypass via Direct Paths
- **Vector**: Caller provides a valid `aios.fs_layout.*` grant with restricted `scope.paths` (e.g. `/var/lib/aios`), but sets `store_path` or `spec` to `/etc/shadow` or an unallowed directory.
- **Expected Defense**: PEP gate canonicalizes all path parameters and validates them against allowed prefixes before granting execution. Fails closed with `path subject` violation if out-of-scope.
- **Verdict**: Mitigated — tested by FL11 P3.

### Scenario A-4: Canonical Path Evasion (Case, 8.3 Short Names, Device Prefixes)
- **Vector**: Caller attempts to bypass `scope.paths.deny` using case manipulation (`SECRETDIR`), 8.3 short names (`SECRET~1`), trailing dots/spaces, or Windows device prefixes (`\\?\`, `\\.\`).
- **Expected Defense**: Canonical path resolver expands and canonicalizes all aliases before matching against allow/deny lists.
- **Verdict**: Mitigated — tested by FL8 C7 & C8.

### Scenario A-5: Nested Prompt Injection in Layout Definitions
- **Vector**: Attacker embeds prompt-injection payload strings inside layout partition labels or metadata within the nested `layout` object.
- **Expected Defense**: Classifier scans all top-level and nested string arguments, refusing execution before processing or persisting.
- **Verdict**: Mitigated — tested by FL8 C6.

---

## 3. Initial Assessment
- All identified abuse scenarios (A-1 through A-5) are covered by existing test cases in FL8 and FL11.
- No open policy bypass vulnerabilities identified.
