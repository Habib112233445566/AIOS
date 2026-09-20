# T-02119: Core Service Documentation — PEP Decision Engine

## Overview
- **Task ID**: `T-02119`
- **Sub-Epic**: 2 (Core Service)
- **Status**: Completed

## Documentation Updates
1. **System Specification**:
   - `docs/pep_decision_engine.md` updated with Section 4 detailing `PepDecisionService` architecture, capacity limits (`5000` rules), atomic persistence (`save_to_path`), and non-destructive quarantine recovery (`load_or_recover`).
2. **Copy-Pasteable Code Examples**:
   - Rust code snippet showing service initialization, rule addition, and evaluation against combining algorithms.
   - MCP JSON-RPC call and response examples for `aios.pep.evaluate`.
   - Operator CLI commands preview for `aiosh pep`.
3. **Constraints & Known Limitations**:
   - Maximum 5000 rules per service instance.
   - In-memory multi-index rebuild on file reload.
   - Resource URIs must be normalized without `..` traversal components.
4. **Evidence Cross-Links**:
   - Links to all Sub-Epic 1 and Sub-Epic 2 task evidence files recorded in `docs/pep_decision_engine.md`.
