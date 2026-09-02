# T-00395 — Dependency & Toolchain Pinning / documentation: Unit Test

## 1. Unit Test Scope
This task tests and validates the documentation of Dependency & Toolchain Pinning against the automated structural documentation test suite (`tools/check_task_docs.py`).

## 2. Invariant Criteria Checked
- **C1 (Spec Health)**: Validates spec documents for malformed tables or corrupted headers.
- **C2 (Component Sections)**: Ensures all Phase 0 epics have clearly structured sections in `docs/README.md`.
- **C3 (Referenced Paths)**: Confirms all relative paths referenced in markdown exist in the filesystem tree.
- **C4 (Phase Map)**: Validates the consistency of the task ledger phase mappings.
- **C5 (Index Health)**: Validates link targets and table of contents anchors.
- **C6 (No Volatile Counts)**: Prevents hardcoded task counts that rot across task completions.

## 3. Test Execution Output
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```
