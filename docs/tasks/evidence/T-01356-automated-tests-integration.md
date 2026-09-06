# T-01356: Init & Service Supervision - Automated Tests: Integration

## Metadata
- **Task ID:** `T-01356`
- **Subsystem:** `tools/`, `code/aiosh-rust`
- **Component:** Init & Service Supervision Automated Tests Integration
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Integration Overview

This task integrates the automated test suite `code/aiosh-rust/aiosh-core/tests/test_service_automated.rs` into the canonical subsystem test runner `tools/test_service_suites.py` under the formal criterion `SS6`.

### Integration Matrix:
- **Criterion `SS1`**: Data model integrity & invariants (`test_service_data_model.rs`)
- **Criterion `SS2`**: Operator CLI surface commands & options (`aiosh` binary unit tests)
- **Criterion `SS3`**: Autonomous Agent MCP tool surface (`aiosh-mcp` binary unit tests)
- **Criterion `SS4`**: Core service lifecycle, FSM & dependency ordering (`test_service_service.rs`)
- **Criterion `SS5`**: Configuration subsystem invariants, precedence & sizing (`test_service_config.rs`)
- **Criterion `SS6`**: Holistic automated integration test suite (`test_service_automated.rs`)

---

## 2. Test Execution & Output

Ran `python tools/test_service_suites.py`:
```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate, list, get, action, order, config)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)
[+] SS5 service configuration subsystem invariants, precedence & sizing (SC1..SC7)
[+] SS6 service automated integration tests (ST1..ST5)

PASS: service_suites criteria (SS1..SS6)
```

---

## 3. Acceptance Verification
- [x] Feature reachable through its production surface.
- [x] Integration smoke runner updated and passing end-to-end (`SS1..SS6` all green).
