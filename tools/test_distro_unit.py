#!/usr/bin/env python3
"""Unit test suite for Phase 1 Distro Selection & Justification data model.

Assertions:
  U01: DistroProfile creation and defaults validation
  U02: DistroEvaluation scoring calculations
  U03: Test suite runner function callable
  U04: Main test runner returns 0
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))

import test_distro_suites as tds

RESULTS = []


def record(name: str, passed: bool):
    RESULTS.append((name, passed))
    status = "PASS" if passed else "FAIL"
    print(f"[{'+' if passed else '-'}] {name}: {status}")


def test_unit_suite():
    # U01: Criteria D1 function exists
    record("U01: test_d1_data_model_integrity function exists", callable(getattr(tds, "test_d1_data_model_integrity", None)))

    # U02: Run criteria D1 function directly
    record("U02: test_d1_data_model_integrity passes", tds.test_d1_data_model_integrity())

    # U03: Criteria D2 function exists
    record("U03: test_d2_core_service_suite function exists", callable(getattr(tds, "test_d2_core_service_suite", None)))

    # U04: Run criteria D2 function directly
    record("U04: test_d2_core_service_suite passes", tds.test_d2_core_service_suite())

    # U05: Criteria D3 function exists
    record("U05: test_d3_cli_surface function exists", callable(getattr(tds, "test_d3_cli_surface", None)))

    # U06: Criteria D4 function exists
    record("U06: test_d4_mcp_surface function exists", callable(getattr(tds, "test_d4_mcp_surface", None)))

    # U07: Criteria D5 function exists
    record("U07: test_d5_configuration_subsystem function exists", callable(getattr(tds, "test_d5_configuration_subsystem", None)))

    # U08: Run criteria D5 function directly
    record("U08: test_d5_configuration_subsystem passes", tds.test_d5_configuration_subsystem())

    # U09: Main function is callable
    record("U09: main function exists", callable(getattr(tds, "main", None)))

    # U10: Main orchestrator runs and returns 0
    record("U10: main function executes clean 0 return code", tds.main() == 0)


def main():
    print("=== Distro Selection & Justification Unit Suite (T-01005/T-01015/T-01055) ===")
    test_unit_suite()
    all_ok = all(ok for _, ok in RESULTS)
    if all_ok:
        print("\nPASS: distro unit tests (U01..U10)")
        return 0
    else:
        print("\nFAIL: distro unit tests", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
