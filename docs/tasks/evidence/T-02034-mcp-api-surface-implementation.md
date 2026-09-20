# T-02034: Capability Model / MCP/API Surface — Implementation

**Task ID**: `T-02034`  
**Phase**: Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic**: Sub-Epic 4: Capability Model / MCP/API Surface  
**Status**: COMPLETED  
**Date**: 2026-09-20  

---

## 1. Implementation Summary

The 7 Model Context Protocol (MCP) tools for the AIOS Capability Model have been implemented in `code/aiosh-rust/aiosh-mcp/src/main.rs`:

1. **`aios.capability.list`**:
   - Lists registered capabilities with optional subject or active-only filtering.
   - Loads via `CapabilityService::load_or_create(path)`.
   - Returns count and capability list.
2. **`aios.capability.get`**:
   - Retrieves capability metadata by ID (`CAP-<uuid>`).
   - Returns 404 error envelope if missing.
3. **`aios.capability.issue`**:
   - Parses `scope_type`, `scope_target`, and `rights`.
   - Evaluates caller identity (`kernel` or `admin:*`) and constructs `CapabilityConstraints` (expiration, quotas).
   - Issues root capability and atomically persists to backing store.
4. **`aios.capability.attenuate`**:
   - Validates parent delegation rights and enforces monotonic restriction on child scope/rights.
   - Issues child capability with lineage tracking and persists to disk.
5. **`aios.capability.revoke`**:
   - Performs transitive cascade revocation of the capability and all descendant subtrees.
   - Commits updated state to disk and returns list of all revoked IDs.
6. **`aios.capability.check`**:
   - Fast access check for subject, scope, and right.
   - Supports optional invocation quota consumption when `consume: true`.
7. **`aios.capability.prune`**:
   - Prunes expired leaf capabilities without active children.
   - Commits pruned state to disk if any records were removed.

---

## 2. Invariants & Audit Parity

- All 7 tools execute through `dispatch::recorded_call`, preserving the ADR-0035 §A F-2 invariant: every tool call emits exactly one hash-chained audit row to the SQLite WAL audit ring.
- Consequential mutations (`issue`, `attenuate`, `revoke`, `prune`) perform atomic file updates via `CapabilityService::save_to_path`.
- Path traversal and size limit enforcement protect the backing store from corruption or resource exhaustion.
