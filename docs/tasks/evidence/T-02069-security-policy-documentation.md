# Evidence: T-02069 - security policy: Documentation

## Task Overview
- **Task ID**: `T-02069`
- **Sub-Epic**: Sub-Epic 7: Security Policy (`T-02061`..`T-02070`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Document Capability Security Policy for operators and agents in `docs/capability_model.md`.

## Documentation Summary
- Authored Section 12 in `docs/capability_model.md`:
  - **12.1 Policy Invariants (`CAPSEC1..CAPSEC6`)**: Detailed table specifying default deny, depth bounds, prohibited paths/hosts, disallowed subject rights, temporal ceilings, and auditability.
  - **12.2 Hardened Evaluation Logic**: Path normalization, traversal detection, host sanitization, and cycle detection.
  - **12.3 MCP Usage Example**: Copy-pasteable JSON-RPC tool call and error envelope.
  - **12.4 Known Constraints & Limitations**: Documented prefix matching constraints and cycle guards.
