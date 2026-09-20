# Integration Evidence: Hardware Detection Documentation Subsystem (T-01786)

## 1. Executive Summary
- **Subsystem**: Hardware Detection Documentation Subsystem (`code/aiosh-cli/tests/test_hardware_doc_smoke.py`, `hardware_service.rs`)
- **Task**: `T-01786`
- **Scope**: Cross-surface integration smoke tests and service API integration.
- **Status**: **PASS (Integration Complete)**

---

## 2. Integration Deliverables

### 1. Service API Wiring
- Integrated `HardwareService::get_doc_topic` and `HardwareService::search_doc_topics` into `code/aiosh-rust/aiosh-core/src/hardware_service.rs`.
- Enables callers to query and search offline documentation topics directly via `HardwareService`.

### 2. Integration Smoke Suite
- Executed `code/aiosh-cli/tests/test_hardware_doc_smoke.py`:
  - `test_hdoc1_canonical_topics`: PASS
  - `test_hdoc2_case_insensitive_lookup`: PASS
  - `test_hdoc3_search_scoring`: PASS
  - `test_hdoc4_category_filter`: PASS
  - `test_hdoc5_markdown_formatting`: PASS
- Verified cross-substrate parity and deterministic search ranking.
