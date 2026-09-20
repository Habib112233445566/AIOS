# Task Evidence: T-02135 (MCP/API Surface: Unit Test)

## Overview
- **Task ID**: `T-02135`
- **Task Name**: MCP/API surface: Unit Test
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 4: MCP / API Surface
- **Timestamp**: 2026-09-21T00:49:00+05:00
- **Status**: COMPLETED

## Objective
Verify MCP/API surface tools for the PEP Decision Engine via end-to-end integration and smoke tests in `code/aiosh-mcp/tests/test_pep_decision_smoke.py`:
1. Tool Registration in `tools/list`: `aios.pep.status`, `aios.pep.rule_add`, `aios.pep.rule_list`, `aios.pep.rule_remove`, and `aios.pep.evaluate`.
2. Rule Evaluation: Default deny, permit rules, and `deny_overrides` combining algorithm.
3. Persistent Lifecycle:
   - Initial status check on clean/empty store (`rules_count: 0`).
   - Rule addition (`rule_add`) for permit and deny rules.
   - Rule listing (`rule_list`) with and without subject/action filters.
   - Evaluation (`evaluate`) directly against persistent store.
   - Rule removal (`rule_remove`) with atomic update.
   - Error handling for non-existent rule removal.
   - Path traversal rejection (`../../../etc/shadow.json`).

## Test Execution Results
```text
=== PEP Decision Engine MCP Smoke Test ===
TEST: tool registration via tools/list ... OK
TEST: PEP decision evaluation ... OK
TEST: PEP MCP persistent lifecycle (status, rule_add, list, eval, remove) ... OK
=== All PEP Decision Engine smoke tests passed ===
```

## Security & Conformance Verification
- **Path Traversal Protection**: MCP tool endpoints enforce `validate_pep_service_path`, rejecting any path containing `..` or non-`.json` extensions.
- **Audit Logging**: Every MCP tool execution is routed through `dispatch::recorded_call` and recorded in the SQLite audit ring.
- **Fail-Safe Defaults**: Any evaluation against a store with no matching rules resolves to `deny`.
