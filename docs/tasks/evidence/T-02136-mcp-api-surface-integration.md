# Task Evidence: T-02136 (MCP/API Surface: Integration)

## Overview
- **Task ID**: `T-02136`
- **Task Name**: MCP/API surface: Integration
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 4: MCP / API Surface
- **Timestamp**: 2026-09-21T00:53:10+05:00
- **Status**: COMPLETED

## Objective
Verify integration of the PEP Decision Engine MCP surface (`aios.pep.*`) with the surrounding AIOS runtime:
1. Wire into real JSON-RPC call path in `aiosh-mcp`.
2. Verify cross-substrate parity between CLI (`aiosh pep`) and MCP tools (`aios.pep.*`) sharing canonical JSON policy stores.
3. Confirm audit trail emission into the SQLite audit ring via `dispatch::recorded_call`.
4. Validate that `tools/list` exposes all 5 PEP tools with accurate schemas.

## Integration Details
- **Cross-Substrate Store Parity**:
  - Both `aiosh-cli` (`cmd_pep`) and `aiosh-mcp` interact with the same underlying `PepDecisionService` persistence model.
  - Rules written by CLI `aiosh pep rule-add --id <id> ...` are instantly readable and enforceable by MCP `aios.pep.evaluate` and `aios.pep.rule_list`.
  - Atomic persistence with `.tmp.<pid>` and rename guarantees zero corruption across concurrent readers.
- **Audit Ring Integration**:
  - Every call to `aios.pep.status`, `aios.pep.rule_add`, `aios.pep.rule_list`, `aios.pep.rule_remove`, and `aios.pep.evaluate` is wrapped in `dispatch::recorded_call`.
  - This automatically evaluates PEP policy for the calling actor, verifies grant IDs if required, and commits an immutable row into the SQLite audit ring.

## Verification
- `code/aiosh-mcp/tests/test_pep_decision_smoke.py` executed and PASSED (3/3 test suites).
- All 5 tools (`aios.pep.status`, `aios.pep.rule_add`, `aios.pep.rule_list`, `aios.pep.rule_remove`, `aios.pep.evaluate`) discoverable and verified.
