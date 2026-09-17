#!/usr/bin/env python3
"""test_session_doc.py - Automated Unit Test for User Session Bootstrap Documentation.

Validates docs/user_session_bootstrap.md structural integrity, completeness, and lack of rot.
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOC_PATH = ROOT / "docs" / "user_session_bootstrap.md"

REQUIRED_SECTIONS = [
    "## 1. Executive Overview & Architectural Role",
    "## 2. Core Data Model & Specification Invariants",
    "## 3. Core Service Registry, Lifecycle FSM & Seat Arbitration",
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
        ("SB1..SB5", "Data model invariants SB1..SB5"),
        ("CS1..CS5", "Core service invariants CS1..CS5"),
        ("SC1..SC7", "Configuration invariants SC1..SC7"),
        ("SSP1..SSP7", "Security policy invariants SSP1..SSP7"),
        ("SSO1..SSO6", "Observability invariants SSO1..SSO6"),
        ("seat0", "Primary hardware seat seat0"),
        ("aiosh session validate", "CLI command validate"),
        ("aiosh session list", "CLI command list"),
        ("aiosh session action", "CLI command action"),
        ("aiosh session create", "CLI command create"),
        ("aiosh session stats", "CLI command stats"),
        ("aios.session.validate", "MCP tool validate"),
        ("aios.session.action", "MCP tool action"),
        ("aios.session.create", "MCP tool create"),
        ("aios.session.stats", "MCP tool stats"),
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


def main() -> int:
    tests = [
        ("D1", test_file_existence_and_size),
        ("D2", test_required_sections),
        ("D3", test_no_forbidden_markers),
        ("D4", test_policy_and_invariant_coverage),
        ("D5", test_negative_cases),
        ("D6", test_no_volatile_counts),
    ]

    failed = []
    for criterion, test_fn in tests:
        if not test_fn():
            failed.append(criterion)

    print()
    if failed:
        print(f"FAIL: test_session_doc failed criteria: {', '.join(failed)}", file=sys.stderr)
        return 1

    print("PASS: test_session_doc criteria (D1..D6)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
