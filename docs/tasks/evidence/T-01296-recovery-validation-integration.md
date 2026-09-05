# T-01296: Package Management Recovery & Validation Integration

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Recovery & Validation  
**Task ID:** T-01296  

---

## 1. Integration Overview
Task `T-01296` integrates the **Recovery & Validation** subsystem of Package Management into the operator CLI (`aiosh package check`), the agent MCP server (`aios.package.check`), and the master test runner (`tools/test_package_suites.py`).

---

## 2. Integrated Surfaces

### 1. Operator CLI: `aiosh package check`
- Implemented in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Options:
  - `--store <PATH>`: Custom package store path with length ($\le 1024$) and control character validation.
  - `--fix`: Enables automated healing with non-destructive backup (`RV4`).
  - `--json`: Outputs structured ADR-0035 envelope.
- Emits non-repudiation audit row via `classify_and_emit`.

### 2. MCP Server: `aios.package.check`
- Implemented in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- Registered in `tool_manifest`.
- Accepts `store_path` and `auto_recover: bool`.
- Gated and audited through `dispatch::recorded_call`.

### 3. Master Test Runner: `tools/test_package_suites.py`
- Added criterion `PM10`: `package recovery & validation integrity (RV1..RV4)`.
- Verified entire test matrix: PM1..PM10 PASS.
