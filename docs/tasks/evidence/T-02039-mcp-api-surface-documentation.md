# Task Evidence: T-02039 (Capability Model / MCP/API surface: Documentation)

## Task Information
- **Task ID**: T-02039
- **Title**: Capability Model / MCP/API surface: Documentation
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 4: MCP / API Surface Integration
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Documentation Completed
1. **Core Documentation Added to `docs/capability_model.md`**:
   - Added comprehensive Section 9: `MCP / API Surface Reference (aios.capability.*)`.
   - Detailed specification of all 7 MCP capability tools:
     - `aios.capability.list`: List capabilities for a subject or all active capabilities.
     - `aios.capability.get`: Retrieve full capability details by capability ID.
     - `aios.capability.issue`: Issue a new root capability with subject, rights, resource URI, constraints, and quotas.
     - `aios.capability.attenuate`: Derive an attenuated child capability with restricted rights, narrowed constraints, or reduced quotas.
     - `aios.capability.revoke`: Revoke a capability and cascade revocation to all derived descendants.
     - `aios.capability.check`: Evaluate whether an agent holds a valid capability for a requested action on a target resource.
     - `aios.capability.prune`: Prune expired capabilities from the active store.
   - Documented exact JSON-RPC request and response payload schemas with field types and descriptions.
   - Documented input validation boundaries and bounds enforced by `validate_mcp_string` and quota range checks.
   - Documented standard error codes and failure modes (e.g., `-32602 Invalid params`, `-32000 Resource not found / Expired / Revoked`).
   - Documented security invariants: no capability forging, strict monotonic attenuation, revocation cascade, fail-closed enforcement.

## Verification
- Verified documentation accuracy against implementation in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- Validated markdown formatting and cross-references.
