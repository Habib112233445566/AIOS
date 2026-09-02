#!/usr/bin/env python3
"""Security-policy criteria checker (T-00075).

Validates root SECURITY.md against the OpenSSF Scorecard
Security-Policy text criteria (E1–E3) AND against the repo itself:

  S1 file exists at root, no TODO markers remain
  S2 reporting URL present exactly as pinned by the owner (D1)
  S3 free-form prose beyond links (length floor)
  S4 specific-text hits: >=2 'vuln*', >=1 'disclos*', >=1 day-count
  S5 every relative link target in the file exists in the tree
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / "SECURITY.md"
ADVISORY_URL = "https://github.com/Habib112233445566/AIOS/security/advisories/new"

PASS, FAIL = "[+]", "[-]"


def main() -> int:
    failures = 0

    def check(label, ok, detail=""):
        nonlocal failures
        print(f"{PASS if ok else FAIL} {label}")
        if not ok:
            print("   ", detail)
            failures += 1

    # S1 exists + no TODOs
    ok = POLICY.exists()
    check("S1 SECURITY.md exists at root", ok)
    if not ok:
        return 1
    text = POLICY.read_text(encoding="utf-8")
    check("S1b no TODO markers remain", "TODO" not in text)

    # S2 pinned channel verbatim
    check("S2 advisory URL present verbatim", ADVISORY_URL in text)

    # S3 free-form floor
    check("S3 free-form prose (>1200 chars)", len(text) > 1200)

    # S4 specific-text hits (scorecard regex families)
    vuln = len(re.findall(r"vuln", text, re.I))
    disc = len(re.findall(r"disclos", text, re.I))
    days = bool(re.search(r"\d+\s*days", text))
    check(f"S4 specific text (vuln={vuln}, disclos={disc}, day-count={days})",
          vuln >= 2 and disc >= 1 and days)

    # S5 every markdown-relative link target exists in the tree
    missing = []
    for m in re.finditer(r"`([^`]+)`", text):
        rel = m.group(1)
        if rel.startswith(("http", "AIOSH_", "sprint-")) or "/" not in rel:
            continue
        if "*" in rel:   # directory-glob references: resolve prefix only
            import glob as _g
            if not _g.glob(str(ROOT / rel)):
                missing.append(rel)
            continue
        if not (ROOT / rel).exists():
            missing.append(rel)
    check("S5 all referenced in-tree paths exist", not missing, missing)

    if failures:
        print(f"\n{failures} check(s) failed")
        return 1
    print("\nPASS: security policy criteria (S1..S5)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
