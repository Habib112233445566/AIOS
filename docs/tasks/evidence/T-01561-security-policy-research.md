# T-01561 — Filesystem Layout security policy: Research

## Metadata
- **Task ID:** `T-01561`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** Complete — established facts, constraints, prior art, and decisions for the security policy governing the Filesystem Layout subsystem.
- **Date:** 2026-09-19
- **Depends on:** `T-01560` (Automated Tests Verification & Evidence)
- **Feeds:** `T-01562` (Security Policy Specification)
- **Artifacts:** `docs/tasks/evidence/T-01561-security-policy-research.md`, `docs/tasks/evidence/T-01561-research.md`

---

## 1. Existing Security Architecture & Prior Art

The security architecture of the Filesystem Layout subsystem is grounded in:
1. **Policy Enforcement Point (ADR-0034 / `aiosh_core::pep`)**:
   - Capability-based authorization using cryptographic grant tokens.
   - Fine-grained scope boundaries:
     - `scope.tools`: Glob patterns restricting which tool names a caller can invoke.
     - `scope.paths`: Allow/deny lists restricting filesystem access. Canonical matching eliminates evasion via case sensitivity, 8.3 short names, or relative traversals.
2. **Command Classification (`aiosh_core::classifier`)**:
   - Classifies operations into risk tiers (`R-01`..`R-14`), tracking irreversibility and impact.
   - State-changing layout mutations (`register`, `set_active`, `remove`, `import_fstab`) are classified as high-impact and require explicit grant delegation.
3. **Audit Ring (ADR-0035 / `aiosh_core::audit`)**:
   - SHA-256 hash-chained SQLite WAL ring logging every call, whether successful, failed, or refused by policy.
4. **CIS Benchmark Mount Options**:
   - Invariant FL4 mandates `nodev`, `nosuid`, and `noexec` on `/tmp` and `/dev/shm`.

---

## 2. Fact vs. Assumption Analysis

| Topic | Established Fact | Assumption / Working Hypothesis |
|---|---|---|
| **PEP Gating Boundary** | All state-mutating MCP tools require valid PEP grants matching `scope.tools` and `scope.paths`. Read-only tools do not require grants. | A caller granted `aios.fs_layout.*` should be confined strictly to authorized directories for both read (`spec`, `fstab`) and write (`store_path`). |
| **Operator CLI Context** | CLI commands run with operator privileges and emit audit rows attributed to `actor="operator"`. | CLI mutations should enforce path hygiene and regular file validation even when run by root/operator. |
| **Forensic Traceability** | Gate refusals write audit records with `outcome="refused"` and `target="None"` or target ID. | Every refusal must record the exact rule or scope violation without leaking sensitive directory layouts. |

---

## 3. Decisions Needed for T-01562 (Specification)

- **D1: Tool Authorization Matrix**: Explicitly define which tools require PEP grants and which are ungated reads.
- **D2: Scope Path Confinement**: Define exact path subjects for every tool (e.g. `store_path`, `spec`, `fstab`).
- **D3: Audit Classification Contract**: Define exact classifier rule IDs and verdicts for layout operations.

---

## 4. Acceptance Confirmation

- [x] Evidence file exists and separates facts from assumptions.
- [x] No code changed; decisions needed are listed explicitly.
