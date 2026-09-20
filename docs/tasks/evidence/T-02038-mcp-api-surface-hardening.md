# T-02038: Capability Model / MCP/API Surface — Hardening

**Task ID**: `T-02038`  
**Phase**: Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic**: Sub-Epic 4: Capability Model / MCP/API Surface  
**Status**: COMPLETED  
**Date**: 2026-09-20  

---

## 1. Hardening Measures Implemented

Defensive validations were introduced across all 7 MCP capability tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`:

1. **Input String Bounds & Control Character Rejection**:
   - Implemented `validate_mcp_string(val, name, max_len)` enforcing string length caps:
     - Capability IDs $\le 128$ chars
     - Subjects & Issuers $\le 256$ chars
     - Scope types & rights $\le 64$ chars
     - Scope targets & store paths $\le 1024$ chars
   - Strictly rejects any ASCII control characters (`< 32` or `\0`) to prevent log injection in the Audit Ring or terminal escape attacks.
2. **Quota Bounds & Numeric Overflow Defenses**:
   - `max_invocations` bounded in range $[1, 100\,000\,000]$.
   - `quota_bytes` bounded in range $[1, 10\,000\,000\,000]$.
   - `expires_in_secs` bounded in range $[1, 315\,360\,000]$ (10 years).
   - Zero or out-of-range values immediately rejected with descriptive error envelopes.
3. **Path Traversal & Storage Hygiene**:
   - `store_path` validated via `validate_mcp_string` and checked for path traversal (`..`), symlinks, and `.json` extension by `CapabilityService::load_or_create`.
4. **Honest Audit Logging**:
   - All failure modes produce explicit error messages routed through `dispatch::recorded_call`, preserving the ADR-0035 §A F-2 invariant: every tool invocation emits exactly one hash-chained audit row.

---

## 2. Verification

All unit tests and integration tests pass with zero regressions.
