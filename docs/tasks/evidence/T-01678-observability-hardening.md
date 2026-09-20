# T-01678: Kernel Module Observability Hardening

## Sub-Epic
Kernel Module Management / Observability (T-01678)

## Objective
Implement defensive bounds, resource exhaustion mitigations, and envelope protections identified during the T-01677 security review.

## Implemented Hardening Measures

1. **Procfs File Size Ceiling (`MAX_PROC_MODULES_BYTES = 1 MiB`)**:
   - Enforced in `KernelModuleService::list_loaded_modules`.
   - Rejects mock or arbitrary input files exceeding 1 MiB (`1,048,576` bytes) before and during reading.
   - Verified via unit test `test_oversized_proc_modules_refusal`.

2. **Line Length Ceiling (`MAX_MODULE_LINE_BYTES = 512 bytes`)**:
   - Enforced on each line parsed from the procfs reader.
   - Prevents memory exhaustion or line buffer exploits from untrusted inputs.
   - Verified via unit test `test_overlong_proc_modules_line_refusal`.

3. **Incremental Streaming via `BufReader`**:
   - Migrated from whole-file string allocation to streaming `BufReader.lines()`.
   - Tracks cumulative byte count to abort reading early if a virtual file stream expands beyond 1 MiB.

4. **Saturating Arithmetic for Telemetry Totals**:
   - Telemetry memory metrics use `saturating_add` to eliminate any risk of integer overflow.

## Verification
- Unit test suite `test_kernel_module_service` (8 tests passing):
  - `test_oversized_proc_modules_refusal`: PASS
  - `test_overlong_proc_modules_line_refusal`: PASS
- Observability test suite `test_kernel_module_observability` (5 tests passing): PASS
- Integration smoke suite `test_kernel_module_observability_smoke.py`: PASS
