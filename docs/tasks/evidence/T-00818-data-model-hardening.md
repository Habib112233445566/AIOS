# T-00818 — Regression Triage / data model: Hardening

## 1. Hardening Deliverables
- **Size Capping**: Implemented hard bounds on `TriageRecord` fields:
  - `MAX_ERROR_MSG_BYTES` = 65536 bytes (64 KiB).
  - `MAX_REPRO_CMD_BYTES` = 4096 bytes (4 KiB).
  - `MAX_TEST_TARGET_BYTES` = 512 bytes.
- **Arithmetic Safety**: `occurrences` counter uses `saturating_add(1)` to guarantee overflow protection.
- **Fail-Closed Validation**: `validate_triage_report` fails closed on arithmetic or slice length discrepancies.
