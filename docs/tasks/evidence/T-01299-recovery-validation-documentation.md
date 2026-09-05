# T-01299: Package Management Recovery & Validation Documentation

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Recovery & Validation  
**Task ID:** T-01299  

---

## 1. Documentation Overview
Task `T-01299` documents the **Recovery & Validation** subsystem of Package Management for human operators and autonomous AI agents.

---

## 2. Documentation Updates Applied
1. **`docs/package_management.md`**:
   - Added `aiosh package check [--store <path>] [--fix] [--json]` CLI reference with syntax, flags, and expected outputs.
   - Added `aios.package.check` MCP agent tool specification with JSON-RPC payload schema.
   - Documented recovery and integrity invariants `RV1..RV4`.
   - Documented error envelopes: `RECOVERY_FAILED` and `UNHEALTHY_STORE`.
   - Validated against criteria D1..D6 via `tools/test_package_doc.py`.

2. **`docs/README.md`**:
   - Updated Section 8.12 to document `aiosh package check` and `aios.package.check`.
   - Added subsection for `Package Recovery & Validation Subsystem (RV1..RV4)`.
   - Added copy-pasteable CLI and JSON-RPC invocation examples.
   - Updated test runner execution matrix to PM1..PM10.

---

## 3. Invariants Documented
- **`RV1` (Count Conservation)**: $\text{valid\_packages} + \text{invalid\_packages} == \text{total\_packages}$.
- **`RV2` (Health Equivalence)**: $\text{healthy} \iff (\text{errors.is\_empty}() \land \text{invalid\_packages} == 0)$.
- **`RV3` (Error Completeness)**: $\text{errors.len}() \ge \text{invalid\_packages}$.
- **`RV4` (Forensic Preservation)**: Corrupted files are backed up to `<path>.bak.<timestamp>` before any reseed.

---

## 4. Constraints and Known Limitations (Honest Disclosure)
- Recovery restores default reference packages for Debian 12 and Alpine 3.19; custom installed third-party packages in a corrupted store must be re-added from their `.bak` archives or source repositories.
- The recovery mechanism operates on JSON store files; underlying filesystem binaries are not removed during store quarantine.
