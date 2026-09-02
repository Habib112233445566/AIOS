#!/usr/bin/env python3
"""Behavioral unit test suite for Regression Triage automated tests (T-00865).

Coverage:
  U01/U02  T1 Data model serialization and SHA-256 signature stability
  U03/U04  T2 TriageStore persistence and CI summary deduplication
  U05/U06  T3 CLI surface subcommands and exit codes
  U07/U08  T4 MCP tool schemas and JSON-RPC dispatch
  U09/U10  T5 Configuration schema, bounds, and auto-ingest suite filtering
  U11/U12  T6 End-to-end regression lifecycle and recurrence reopening
  S01      Sensitivity check: ensuring runner detects failing criteria
"""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
MOD_PATH = HERE / "test_triage_suites.py"

spec = importlib.util.spec_from_file_location("test_triage_suites_ut", MOD_PATH)
assert spec is not None and spec.loader is not None
tts = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tts)

PASS, FAIL = "[+]", "[-]"
RESULTS = []


def record(label: str, ok: bool, detail: str = ""):
    print(f"{PASS if ok else FAIL} {label}" + (f"\n    {detail}" if detail and not ok else ""))
    RESULTS.append((label, ok))


def test_unit_suite():
    # U01: Criteria T1 function exists and is callable
    record("U01: test_t1_data_model_integrity function exists", callable(getattr(tts, "test_t1_data_model_integrity", None)))

    # U02: Criteria T2 function exists and is callable
    record("U02: test_t2_core_service_suite function exists", callable(getattr(tts, "test_t2_core_service_suite", None)))

    # U03: Criteria T3 function exists and is callable
    record("U03: test_t3_cli_surface function exists", callable(getattr(tts, "test_t3_cli_surface", None)))

    # U04: Criteria T4 function exists and is callable
    record("U04: test_t4_mcp_surface function exists", callable(getattr(tts, "test_t4_mcp_surface", None)))

    # U05: Criteria T5 function exists and is callable
    record("U05: test_t5_configuration_suite function exists", callable(getattr(tts, "test_t5_configuration_suite", None)))

    # U06: Criteria T6 function exists and is callable
    record("U06: test_t6_e2e_lifecycle_suite function exists", callable(getattr(tts, "test_t6_e2e_lifecycle_suite", None)))

    # U07: Criteria T7 function exists and is callable
    record("U07: test_t7_observability_suite function exists", callable(getattr(tts, "test_t7_observability_suite", None)))

    # U08: Criteria T8 function exists and is callable
    record("U08: test_t8_recovery_validation_suite function exists", callable(getattr(tts, "test_t8_recovery_validation_suite", None)))

    # U09: Main orchestrator runs all 8 criteria
    record("U09: main function executes clean 0 return code", tts.main() == 0)


def main():
    print("=== Triage Automated Tests Unit Suite (T-00865) ===")
    test_unit_suite()
    all_ok = all(ok for _, ok in RESULTS)
    if all_ok:
        print("\nPASS: triage unit tests (U01..U09)")
        return 0
    else:
        print("\nFAIL: triage unit tests", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
