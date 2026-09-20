# T-01739: Hardware Detection — MCP/API Surface Documentation

## Metadata
- **Task ID**: `T-01739`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Documentation Updates
Section 10 ("Hardware Detection MCP & Agent Surface (Sub-Epic 4)") was added to `docs/hardware_detection.md` detailing:
1. **Architecture & JSON-RPC stdio Protocol**: Integration of `aiosh-mcp` with AI agents and orchestrators.
2. **Tool Catalog**: Detailed parameter specifications, required fields, and response structures for `aios.hardware.scan`, `list`, `get`, `summary`, and `verify`.
3. **Protocol Invariants (HM1..HM5)**: Documented schema conformity (`additionalProperties: false`), uniform response envelopes, hash-chained SQLite WAL audit logging, parameter hygiene, and hermetic path overrides.
4. **Sub-Epic 4 Evidence Traceability**: Comprehensive reference table to artifacts `T-01731` through `T-01740`.
