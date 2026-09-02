#!/usr/bin/env python3
"""Scaffold/interface test for tools/test_doc_index_suites.py (T-00463).

Proves: the test suite runner imports cleanly, every spec'd interface exists with
the right signature shape, and unimplemented bodies fail LOUDLY
(NotImplementedError) until T-00464.
"""

from __future__ import annotations

import importlib.util
import inspect
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MOD_PATH = HERE / "test_doc_index_suites.py"

spec = importlib.util.spec_from_file_location("test_doc_index_suites", MOD_PATH)
assert spec is not None and spec.loader is not None, f"cannot load {MOD_PATH}"
td = importlib.util.module_from_spec(spec)
spec.loader.exec_module(td)

REQUIRED = {
    "check_d1_manifest_model": 0,
    "check_d2_config_hierarchy": 0,
    "check_d3_title_and_link_extraction": 0,
    "check_d4_link_integrity_and_traversal": 0,
    "check_d5_cli_subcommands": 0,
    "check_d6_mcp_surface": 0,
    "check_d7_hardening_limits": 0,
    "run_all_criteria": 0,
    "main": 0,
}

fails = []
for name, min_pos in REQUIRED.items():
    fn = getattr(td, name, None)
    if fn is None:
        fails.append(f"missing: {name}")
        continue
    sig = inspect.signature(fn)
    pos = sum(1 for p in sig.parameters.values()
              if p.kind in (p.POSITIONAL_ONLY, p.POSITIONAL_OR_KEYWORD))
    if pos < min_pos:
        fails.append(f"{name}: positional params {pos} < required {min_pos}")

# Constants from specification
assert td.CRITERIA == ["D1", "D2", "D3", "D4", "D5", "D6", "D7"], f"unexpected CRITERIA: {td.CRITERIA}"

implemented = getattr(td, "IS_IMPLEMENTED", False)
check_functions = [
    "check_d1_manifest_model",
    "check_d2_config_hierarchy",
    "check_d3_title_and_link_extraction",
    "check_d4_link_integrity_and_traversal",
    "check_d5_cli_subcommands",
    "check_d6_mcp_surface",
    "check_d7_hardening_limits",
]

for name in check_functions:
    fn = getattr(td, name)
    try:
        result = fn()
    except NotImplementedError:
        if implemented:
            fails.append(f"{name}: still raises NotImplementedError but IS_IMPLEMENTED is True")
        continue
    except Exception as e:
        fails.append(f"{name}: unexpected exception type {type(e).__name__}: {e}")
        continue
    if not implemented:
        fails.append(f"{name}: returned {result!r} while IS_IMPLEMENTED is False")
    elif not (isinstance(result, tuple) and len(result) == 2 and isinstance(result[0], bool) and isinstance(result[1], str)):
        fails.append(f"{name}: bad return shape {result!r} (expected (bool, str))")

if fails:
    print("FAIL:")
    for f in fails:
        print("  -", f)
    sys.exit(1)

print("PASS: doc_index test scaffold — all interfaces present and fail loudly")
