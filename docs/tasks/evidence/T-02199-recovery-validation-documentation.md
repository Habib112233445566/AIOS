# Task Evidence: T-02199 (recovery & validation: Documentation)

## 1. Scope & Execution
Documented the PEP Decision Engine Recovery & Validation subsystem for operators and automated agents:
- Updated `docs/pep_decision_engine.md` adding Section 14 (Recovery & Validation Subsystem Reference).
- Formally recorded subsystem invariants `PEPRECV1..PEPRECV6`.
- Provided copy-pasteable CLI commands (`aiosh pep validate`, `aiosh pep recover`) and MCP JSON tool call payloads (`aios.pep.validate`, `aios.pep.recover`).
- Documented limits, capacity constraints, and error behaviors honestly.
- Linked full chain of task evidence files (`T-02191` through `T-02200`).

## 2. Documentation Summary
- **Invariants Defined**:
  - `PEPRECV1`: Structural & semantic validation.
  - `PEPRECV2`: JSON array and object map parity.
  - `PEPRECV3`: Deterministic recovery strategies (`dry_run`, `salvage`, `strict_fail_closed`).
  - `PEPRECV4`: Atomic staging & hardened quarantine backup.
  - `PEPRECV5`: Dual-substrate CLI/MCP parity.
  - `PEPRECV6`: Zero silent failure and audit trail integration.
- **Operator Guides**:
  - Practical usage patterns for offline integrity auditing, salvage recovery, and strict fail-closed state resets.
- **Constraints & Boundaries**:
  - Store cap of 10 MiB, max 5,000 rules, Unix file mode `0600` on quarantine files.

## 3. Acceptance Confirmation
- [x] Documentation updated with working CLI and MCP examples.
- [x] Limitations, bounds, and failure modes explicitly documented.
- [x] Task evidence files linked from documentation.
