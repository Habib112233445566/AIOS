# Integration Evidence - T-01866: Network Bootstrap Security Policy Integration

- Test File: `code/aiosh-cli/tests/test_network_policy_smoke.py`
- Coverage:
  - `NPOL1`: Interface gatekeeping, promiscuous detection, MAC checks.
  - `NPOL2`: Route orphan detection.
  - `NPOL3`: DNS server governance.
  - `NPOL4`: Capacity constraints across enforcing, audit, and permissive modes.
  - `NPOL5` & `NPOL6`: State sanitization, path bounds, persistence, and size enforcement.
- Execution: All 5 integration test suites passed cleanly with exit code 0.
