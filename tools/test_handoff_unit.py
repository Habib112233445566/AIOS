#!/usr/bin/env python3
"""Behavioral unit test suite for Agent Handoff Protocol data model (T-00915).

Asserts criteria:
  U01: test_h1_data_model_integrity function exists in test_handoff_suites
  U02: Handoff data model cargo test runs cleanly and passes
  U03: Main runner returns 0
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))

try:
    import test_handoff_suites as ths
except ImportError as e:
    print(f"[-] Failed to import test_handoff_suites: {e}", file=sys.stderr)
    sys.exit(1)

RESULTS: list[tuple[str, bool]] = []


def record(name: str, ok: bool):
    RESULTS.append((name, ok))
    status = "[+]" if ok else "[-]"
    print(f"{status} {name}")


def test_unit_suite():
    # U01: Criteria H1 function exists and is callable
    record("U01: test_h1_data_model_integrity function exists", callable(getattr(ths, "test_h1_data_model_integrity", None)))

    # U02: Run criteria H1 function directly
    record("U02: test_h1_data_model_integrity passes", ths.test_h1_data_model_integrity())

    # U03: Criteria H2 function exists and is callable
    record("U03: test_h2_core_service_suite function exists", callable(getattr(ths, "test_h2_core_service_suite", None)))

    # U04: Run criteria H2 function directly
    record("U04: test_h2_core_service_suite passes", ths.test_h2_core_service_suite())

    # U05: Criteria H3 function exists and is callable
    record("U05: test_h3_cli_surface function exists", callable(getattr(ths, "test_h3_cli_surface", None)))

    # U06: Run criteria H3 function directly
    record("U06: test_h3_cli_surface passes", ths.test_h3_cli_surface())

    # U07: Criteria H4 function exists and is callable
    record("U07: test_h4_mcp_surface function exists", callable(getattr(ths, "test_h4_mcp_surface", None)))

    # U08: Run criteria H4 function directly
    record("U08: test_h4_mcp_surface passes", ths.test_h4_mcp_surface())

    # U09: Criteria H5 function exists and is callable
    record("U09: test_h5_configuration function exists", callable(getattr(ths, "test_h5_configuration", None)))

    # U10: Run criteria H5 function directly
    record("U10: test_h5_configuration passes", ths.test_h5_configuration())

    # U11: Criteria H6 function exists and is callable
    record("U11: test_h6_automated_suite function exists", callable(getattr(ths, "test_h6_automated_suite", None)))

    # U12: Run criteria H6 function directly
    record("U12: test_h6_automated_suite passes", ths.test_h6_automated_suite())

    # U13: Criteria H7 function exists and is callable
    record("U13: test_h7_security_policy function exists", callable(getattr(ths, "test_h7_security_policy", None)))

    # U14: Run criteria H7 function directly
    record("U14: test_h7_security_policy passes", ths.test_h7_security_policy())

    # U15: Criteria H8 function exists and is callable
    record("U15: test_h8_observability function exists", callable(getattr(ths, "test_h8_observability", None)))

    # U16: Run criteria H8 function directly
    record("U16: test_h8_observability passes", ths.test_h8_observability())

    # U17: Main orchestrator runs and returns 0
    record("U17: main function executes clean 0 return code", ths.main() == 0)


def main():
    print("=== Agent Handoff Protocol Unit Suite (T-00915/T-00925/T-00935/T-00945/T-00955/T-00965/T-00975/T-00985) ===")
    test_unit_suite()
    all_ok = all(ok for _, ok in RESULTS)
    if all_ok:
        print("\nPASS: handoff unit tests (U01..U17)")
        return 0
    else:
        print("\nFAIL: handoff unit tests", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
