# T-01256: Package Management - Automated Tests: Integration

## Metadata
- **Task ID:** `T-01256`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Package Management / Automated Tests
- **Status:** Complete
- **Date:** 2026-09-04

---

## 1. Integration Deliverables
- Integrated the automated test suite `test_package_automated` into the canonical subsystem test harness `tools/test_package_suites.py`.
- Added criterion `PM6`:
  ```python
  def test_pm6_automated_integration():
      return _run_cargo_test(
          ["--test", "test_package_automated"],
          "PM6",
          "package automated integration test matrix (PT1..PT6)",
      )
  ```
- Registered `test_pm6_automated_integration` in `main()` checks array.
- Verified end-to-end execution of criteria `PM1..PM6`.

---

## 2. Test Execution Output
```
[+] PM1 package data model integrity & invariants (PM1..PM5)
[+] PM2 package core service integrity & invariants (CS1..CS5)
[+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)
[+] PM4 package MCP tool surface (validate/list/get/plan/search/apply)
[+] PM5 package configuration resolution & invariants (PC1..PC6)
[+] PM6 package automated integration test matrix (PT1..PT6)

PASS: package_suites criteria (PM1..PM6)
```
Feature is discoverable and exercised end-to-end via the production test runner.
