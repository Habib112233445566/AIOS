# T-01569 — Filesystem Layout security policy: Documentation

## Metadata
- **Task ID:** `T-01569`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** Complete — documented security policy invariants, invocations, constraints, and limitations in `docs/filesystem_layout.md`.
- **Date:** 2026-09-19
- **Depends on:** `T-01568` (Hardening)
- **Feeds:** `T-01570` (Verification & Evidence)
- **Artifacts:** `docs/tasks/evidence/T-01569-security-policy-documentation.md`, `docs/tasks/evidence/T-01569-documentation.md`

---

## 1. Documentation Updates

Updated `docs/filesystem_layout.md` with:
1. **Sub-Epic 7 Overview & Invariants (FL11)**:
   - P1: Gated mutation without grant -> refused.
   - P2: Gated mutation with wrong tool scope -> refused.
   - P3: Gated mutation with out-of-scope path -> refused.
   - P4: Gated mutation with valid grant & in-scope paths -> allowed.
   - P5: Read-only tool execution without grant -> allowed.
2. **Copy-Pasteable Example Invocations**:
   - `aiosh grant create --to agent:mcp-contract --tools "aios.fs_layout.*" --allow /var/lib/aios`
   - JSON-RPC 2.0 tool invocation over `aiosh-mcp`.
   - Standalone test suite execution: `python code/aiosh-cli/tests/test_fs_layout_security_policy.py`.
   - Master test runner execution: `python tools/test_fs_layout_suites.py`.
3. **Honest Limitations & Constraints**:
   - Operator vs Agent boundaries: CLI operates in operator session context; MCP enforces PEP gating.
   - Path canonicalization and device namespace restrictions.
   - Pre-gate audit target resolution constraints (F-1/FIFO hazard prevention).
4. **Task Evidence Cross-Links**:
   - Linked all tasks `T-01561` through `T-01570`.

---

## 2. Acceptance Confirmation

- [x] Docs updated with working example.
- [x] Limitations are stated, not omitted.
