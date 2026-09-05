#!/usr/bin/env python3
"""
test_package_doc.py - Automated Unit Test for Package Management Documentation
Validates docs/package_management.md structural integrity, completeness, and lack of rot.
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOC_PATH = ROOT / "docs" / "package_management.md"

REQUIRED_SECTIONS = [
    "## 1. Executive Overview & Architectural Role",
    "## 2. Core Data Model & Types",
    "## 3. Core Service, Store Registry & Transaction Lifecycle",
    "## 4. Configuration Subsystem",
    "## 5. Security Policy Subsystem",
    "## 6. Observability Telemetry Subsystem",
    "## 7. Operator CLI Surface Reference",
    "## 8. Autonomous Agent MCP Tool Surface Reference",
    "## 9. Failure Modes, Error Envelopes, and Audit Trail",
]

FORBIDDEN_MARKERS = ["TODO", "FIXME", "TBD", "XXX", "PLACEHOLDER"]


def test_file_existence_and_size():
    if not DOC_PATH.exists():
        print(f"[-] FAIL: {DOC_PATH} does not exist", file=sys.stderr)
        return False
    size = DOC_PATH.stat().st_size
    if size < 1000:
        print(f"[-] FAIL: {DOC_PATH} too small ({size} bytes)", file=sys.stderr)
        return False
    if size > 5 * 1024 * 1024:
        print(f"[-] FAIL: {DOC_PATH} too large ({size} bytes)", file=sys.stderr)
        return False
    print(f"[+] D1 doc existence and size bounds ({size} bytes)")
    return True


def test_required_sections():
    content = DOC_PATH.read_text(encoding="utf-8")
    missing = []
    for sec in REQUIRED_SECTIONS:
        if sec not in content:
            missing.append(sec)
    if missing:
        print(f"[-] FAIL: Missing sections: {missing}", file=sys.stderr)
        return False
    print(f"[+] D2 all 9 required sections present")
    return True


def test_no_forbidden_markers():
    content = DOC_PATH.read_text(encoding="utf-8")
    found = []
    for marker in FORBIDDEN_MARKERS:
        if marker in content:
            found.append(marker)
    if found:
        print(f"[-] FAIL: Found forbidden markers: {found}", file=sys.stderr)
        return False
    print(f"[+] D3 zero forbidden placeholders/markers")
    return True


def test_policy_and_invariant_coverage():
    content = DOC_PATH.read_text(encoding="utf-8")
    checks = [
        ("PM1..PM5", "Data model invariants PM1..PM5"),
        ("CS1..CS5", "Core service invariants CS1..CS5"),
        ("PC1..PC6", "Configuration invariants PC1..PC6"),
        ("PP1..PP6", "Security policy invariants PP1..PP6"),
        ("PO1..PO6", "Observability invariants PO1..PO6"),
        ("telnet", "Prohibited package telnet"),
        ("aiosh package validate", "CLI command validate"),
        ("aiosh package plan", "CLI command plan"),
        ("aiosh package apply", "CLI command apply"),
        ("aiosh package stats", "CLI command stats"),
        ("aios.package.plan", "MCP tool plan"),
        ("aios.package.apply", "MCP tool apply"),
        ("aios.package.stats", "MCP tool stats"),
    ]
    missing = [desc for token, desc in checks if token not in content]
    if missing:
        print(f"[-] FAIL: Missing policy/invariant coverage: {missing}", file=sys.stderr)
        return False
    print(f"[+] D4 policy invariants, CLI commands, and MCP tools coverage complete")
    return True


def test_negative_cases():
    dummy_missing = "# Title\n## 1. Executive Overview"
    if any(sec in dummy_missing for sec in REQUIRED_SECTIONS[1:]):
        print("[-] FAIL: Negative test assumption failed", file=sys.stderr)
        return False

    dummy_with_todo = "# Title\nTODO: write this\n"
    if not any(m in dummy_with_todo for m in FORBIDDEN_MARKERS):
        print("[-] FAIL: Negative test marker assumption failed", file=sys.stderr)
        return False

    print(f"[+] D5 negative rejection assertions verified")
    return True


def test_no_volatile_counts():
    content = DOC_PATH.read_text(encoding="utf-8")
    for i, line in enumerate(content.splitlines(), 1):
        if "CI " in line and "/" in line:
            import re
            if re.search(r"CI \d+/\d+", line):
                print(f"[-] FAIL: Volatile CI snapshot count found at line {i}: {line}", file=sys.stderr)
                return False
    print(f"[+] D6 zero volatile snapshot counts (C6 compliant)")
    return True


def main():
    checks = [
        test_file_existence_and_size,
        test_required_sections,
        test_no_forbidden_markers,
        test_policy_and_invariant_coverage,
        test_negative_cases,
        test_no_volatile_counts,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False
    if all_ok:
        print("\nPASS: package_doc unit tests (D1..D6)")
        return 0
    else:
        print("\nFAIL: package_doc unit tests", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
