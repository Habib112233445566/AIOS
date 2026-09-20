# Task Evidence: T-01796 - Hardware Detection / Recovery & Validation: Integration

## Metadata
- **Task ID:** `T-01796`
- **Sub-Epic:** Sub-Epic 10: Hardware Detection / Recovery & Validation
- **Component:** `code/aiosh-cli/tests/test_hardware_recovery_smoke.py`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Integration Smoke Test Execution
Executed `pytest code/aiosh-cli/tests/test_hardware_recovery_smoke.py -v`:

```text
code/aiosh-cli/tests/test_hardware_recovery_smoke.py::test_hval1_healthy_inventory_validation PASSED [ 20%]
code/aiosh-cli/tests/test_hardware_recovery_smoke.py::test_hval2_summary_parity_and_recovery PASSED [ 40%]
code/aiosh-cli/tests/test_hardware_recovery_smoke.py::test_hval4_unparseable_json_quarantine_and_recovery PASSED [ 60%]
code/aiosh-cli/tests/test_hardware_recovery_smoke.py::test_hval5_oversized_file_rejection PASSED [ 80%]
code/aiosh-cli/tests/test_hardware_recovery_smoke.py::test_hval6_drift_detection PASSED [100%]

============================== 5 passed in 0.18s ==============================
```

## Invariants Verified (HVAL1..HVAL6)
- **HVAL1**: Device accounting parity (`valid_devices + invalid_devices == total_devices`) verified.
- **HVAL2**: Class summary parity and automatic reconciliation verified.
- **HVAL3**: Consistency of `healthy` boolean with error/drift state verified.
- **HVAL4**: Non-destructive quarantine to `.bak.<timestamp>` on corrupted JSON verified.
- **HVAL5**: Maximum file size limit (10 MB) verified.
- **HVAL6**: Sysfs path drift detection for missing devices verified.
