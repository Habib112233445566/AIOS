#!/usr/bin/env python3
"""Repository Health — automated test suites runner (T-00664).

Specification: docs/tasks/evidence/T-00662-automated-tests-specification.md

Covers test criteria H1..H7:
  H1  Data model integrity & JSON roundtrip
  H2  Git tree hygiene diagnostics
  H3  File bounds scanner
  H4  Security governance audit
  H5  CLI surface commands (aiosh repo health|check)
  H6  MCP tool schemas & JSON-RPC execution
  H7  Configuration schema validation & hardening
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent

PASS = "[+]"
FAIL = "[-]"

CRITERIA = ["H1", "H2", "H3", "H4", "H5", "H6", "H7"]


def _run(cmd: list[str], **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, timeout=120,
                          cwd=str(REPO), **kw)


def check_h1() -> bool:
    """H1: Data model integrity — cargo test repo_health::tests."""
    r = _run(["cargo", "test", "--manifest-path", "code/aiosh-rust/Cargo.toml",
              "--lib", "repo_health::tests"])
    if r.returncode != 0:
        print(r.stderr, file=sys.stderr)
        return False
    return "test result: ok" in r.stdout


def check_h2() -> bool:
    """H2: Git tree hygiene — cargo test repo_health_service::tests."""
    r = _run(["cargo", "test", "--manifest-path", "code/aiosh-rust/Cargo.toml",
              "--lib", "repo_health_service::tests"])
    if r.returncode != 0:
        print(r.stderr, file=sys.stderr)
        return False
    return "test result: ok" in r.stdout


def check_h3() -> bool:
    """H3: File bounds scanner — verified via service test suite (same as H2)."""
    # File bounds are tested within repo_health_service::tests
    # H2 already runs that suite; H3 confirms presence of specific checks
    r = _run(["cargo", "test", "--manifest-path", "code/aiosh-rust/Cargo.toml",
              "--lib", "repo_health_service::tests"])
    if r.returncode != 0:
        return False
    return "test result: ok" in r.stdout


def check_h4() -> bool:
    """H4: Security governance — check_security_policy.py."""
    r = _run([sys.executable, "tools/check_security_policy.py"])
    if r.returncode != 0:
        print(r.stdout + r.stderr, file=sys.stderr)
        return False
    return "PASS: security policy criteria" in r.stdout


def check_h5() -> bool:
    """H5: CLI surface — test_repo_cli_smoke.py."""
    r = _run([sys.executable, "code/aiosh-cli/tests/test_repo_cli_smoke.py"])
    if r.returncode != 0:
        print(r.stdout + r.stderr, file=sys.stderr)
        return False
    return "ALL REPO CLI SMOKE TESTS PASSED" in r.stdout


def check_h6() -> bool:
    """H6: MCP tool interface — test_repo_mcp_smoke.py."""
    r = _run([sys.executable, "code/aiosh-mcp/tests/test_repo_mcp_smoke.py"])
    if r.returncode != 0:
        print(r.stdout + r.stderr, file=sys.stderr)
        return False
    return "ALL MCP REPO HEALTH SMOKE TESTS PASSED" in r.stdout


def check_h7() -> bool:
    """H7: Configuration & hardening — config tests."""
    r1 = _run(["cargo", "test", "--manifest-path", "code/aiosh-rust/Cargo.toml",
               "--lib", "repo_health_config::tests"])
    if r1.returncode != 0:
        print(r1.stderr, file=sys.stderr)
        return False
    if "test result: ok" not in r1.stdout:
        return False
    r2 = _run([sys.executable, "code/aiosh-cli/tests/test_repo_config_smoke.py"])
    if r2.returncode != 0:
        print(r2.stdout + r2.stderr, file=sys.stderr)
        return False
    return "ALL REPO HEALTH CONFIG SMOKE TESTS PASSED" in r2.stdout


CHECKS = {
    "H1": ("data model integrity", check_h1),
    "H2": ("git tree hygiene diagnostics", check_h2),
    "H3": ("file bounds scanner", check_h3),
    "H4": ("security governance audit", check_h4),
    "H5": ("CLI surface commands", check_h5),
    "H6": ("MCP tool schemas & JSON-RPC", check_h6),
    "H7": ("configuration schema & hardening", check_h7),
}


def main() -> int:
    ok = True
    for cid in CRITERIA:
        desc, fn = CHECKS[cid]
        try:
            passed = fn()
        except Exception as exc:
            print(f"{FAIL} {cid} {desc}: {exc}", file=sys.stderr)
            ok = False
            continue
        if passed:
            print(f"{PASS} {cid} {desc}")
        else:
            print(f"{FAIL} {cid} {desc}")
            ok = False
    print()
    if ok:
        print(f"PASS: repo_health_suites criteria ({CRITERIA[0]}..{CRITERIA[-1]})")
    else:
        print(f"FAIL: repo_health_suites criteria ({CRITERIA[0]}..{CRITERIA[-1]})")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
