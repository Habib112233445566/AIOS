# T-02532: Privilege Escalation Prevention MCP/API Surface Specification

- **Task**: `T-02532`
- **Sub-Epic**: Privilege Escalation Prevention / MCP/API surface
- **Date**: 2026-09-29
- **Status**: PASSED

## 1. Specification Overview
This specification establishes the Model Context Protocol (MCP) JSON-RPC 2.0 tool interface and schema contracts for exposing Privilege Escalation Prevention capabilities to autonomous AI agents in `code/aiosh-rust/aiosh-mcp`.

## 2. Tools Defined
1. `aios.privilege.status`: Inspect active privilege tier, baseline tier, grant ID, and capabilities for an actor.
2. `aios.privilege.elevate`: Request dynamic privilege elevation to a higher tier with grant validation.
3. `aios.privilege.drop`: De-escalate privilege tier safely to a lower tier.
4. `aios.privilege.revoke`: Revoke dynamic elevations and restore baseline privileges.
5. `aios.privilege.check`: Check whether an actor currently holds a specified capability.

## 3. Normative Document
Full specifications, JSON Schemas, invariants `PRIVESC_MCP1`..`PRIVESC_MCP5`, and error matrix are published in `docs/SPEC-PRIVILEGE-MCP.md`.
