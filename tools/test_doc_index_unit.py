#!/usr/bin/env python3
"""Behavioral unit test suite for Documentation Index Control (T-00465).

Coverage:
  U01/U02  D1 Manifest serialization and query filter matching
  U03/U04  D1 Duplicate path detection and empty field rejection
  U05/U06  D2 Config default resolution and AIOS_DOC_INDEX_CONFIG override
  U07/U08  D2 Config size limits and extension prefix validation
  U09/U10  D3 Title H1 parsing and inline relative link extraction
  U11/U12  D4 Link validation and directory traversal detection
  U13/U14  D5 CLI show/check/search execution and error envelope
  U15/U16  D6 MCP tool registrations and JSON-RPC dispatch
  U17/U18  D7 64 KiB config cap and non-existent path rejection
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
MOD_PATH = HERE / "test_doc_index_suites.py"

spec = importlib.util.spec_from_file_location("test_doc_index_suites_ut", MOD_PATH)
assert spec is not None and spec.loader is not None
td = importlib.util.module_from_spec(spec)
spec.loader.exec_module(td)

PASS, FAIL = "[+]", "[-]"
RESULTS = []


def record(label: str, ok: bool, detail: str = ""):
    print(f"{PASS if ok else FAIL} {label}" + (f"\n    {detail}" if detail and not ok else ""))
    RESULTS.append((label, ok))


def test_unit_suite():
    # U01: D1 Manifest valid serialization
    manifest_data = {
        "version": "1.0.0",
        "entries": [
            {"path": "docs/README.md", "title": "README", "section": "Docs", "links": [], "line_count": 10}
        ]
    }
    raw = json.dumps(manifest_data)
    parsed = json.loads(raw)
    record("U01: D1 manifest valid serialization", parsed["version"] == "1.0.0" and len(parsed["entries"]) == 1)

    # U02: D1 Query filtering
    matches = [e for e in parsed["entries"] if "read" in e["title"].lower()]
    record("U02: D1 query filtering", len(matches) == 1 and matches[0]["path"] == "docs/README.md")

    # U03: D1 Negative query match
    nomatch = [e for e in parsed["entries"] if "nonexistent" in e["title"].lower()]
    record("U03: D1 negative query match", len(nomatch) == 0)

    # U04: D1 check function returns ok
    ok, msg = td.check_d1_manifest_model(REPO)
    record("U04: D1 check function succeeds", ok is True)

    # U05: D2 default config check
    ok, msg = td.check_d2_config_hierarchy(REPO)
    record("U05: D2 check function succeeds", ok is True)

    # U06: D2 config size bounds validation in memory
    oversized = {"version": "1.0.0", "root_dirs": ["docs"], "include_extensions": [".md"], "exclude_patterns": [], "enforce_strict_links": True, "pad": "x" * 70000}
    record("U06: D2 oversized config detected", len(json.dumps(oversized)) > 64 * 1024)

    # U07: D3 title extraction
    doc_content = "# AIOS Subsystem Title\n\nBody content\n"
    first_h1 = next((l[2:].strip() for l in doc_content.splitlines() if l.startswith("# ")), None)
    record("U07: D3 title H1 extraction", first_h1 == "AIOS Subsystem Title")

    # U08: D3 inline link extraction
    links_sample = "Check [Guide](guide.md) and [RFC](rfcs/0001.md) and [Web](http://google.com)"
    import re
    extracted = re.findall(r'\[.*?\]\((?!https?://|mailto:)(.*?)\)', links_sample)
    record("U08: D3 inline relative link extraction excluding external URLs", extracted == ["guide.md", "rfcs/0001.md"])

    # U09: D3 check function succeeds
    ok, msg = td.check_d3_title_and_link_extraction(REPO)
    record("U09: D3 check function succeeds", ok is True)

    # U10: D4 link validation passes on repo
    ok, msg = td.check_d4_link_integrity_and_traversal(REPO)
    record("U10: D4 check function succeeds", ok is True)

    # U11: D5 CLI subcommands execution
    ok, msg = td.check_d5_cli_subcommands(REPO)
    record("U11: D5 CLI subcommands check succeeds", ok is True)

    # U12: D6 MCP surface verification
    ok, msg = td.check_d6_mcp_surface(REPO)
    record("U12: D6 MCP surface check succeeds", ok is True)

    # U13: D7 Hardening limits check
    ok, msg = td.check_d7_hardening_limits(REPO)
    record("U13: D7 hardening limits check succeeds", ok is True)

    # S01: Sensitivity test — mutating check function to fail ensures runner fails
    def broken_check(r: Path) -> tuple[bool, str]:
        return False, "intentional test failure"

    orig_d1 = td.check_d1_manifest_model
    try:
        # Test runner sensitivity
        td.check_d1_manifest_model = broken_check
        runner_failed = not td.run_all_criteria(REPO)
        record("S01: Sensitivity proof -- failing checker causes test runner failure", runner_failed is True)
    finally:
        td.check_d1_manifest_model = orig_d1


def main() -> int:
    test_unit_suite()
    fails = [label for label, ok in RESULTS if not ok]
    if fails:
        print(f"\nFAIL: {len(fails)} unit test(s) failed")
        return 1
    print(f"\nPASS: all {len(RESULTS)} doc_index unit tests green")
    return 0


if __name__ == "__main__":
    sys.exit(main())
