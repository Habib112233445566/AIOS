#!/usr/bin/env python3
"""Comprehensive Unit and Smoke Tests for Sandbox CLI Subcommands (T-02425).

Covers all subcommands and parameters of `aiosh sandbox` (and `aiosh sb`):
1. `help`: Missing args, `--help`, `-h`.
2. `profiles`: Listing default profiles in human format and `--json` envelope.
3. `probe`: Probing host kernel sandbox containment capabilities in text and `--json`.
4. `exec`:
   - Valid execution of command under standard/permissive profile.
   - Output piping and `--json` envelope validation.
   - Validation failure: missing `--` delimiter.
   - Validation failure: missing binary executable after delimiter.
   - Validation failure: directory traversal in `--cwd`.
   - Validation failure: nonexistent profile request.
5. `unknown`: Rejection of unknown subcommands with exit code 2.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _find_binary(names: list[str]) -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in names:
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return names[-1]


def get_aiosh_binary() -> str:
    return _find_binary(["aiosh.exe", "aiosh"])


def run_aiosh(*args: str, timeout_s: int = 15) -> subprocess.CompletedProcess:
    bin_path = get_aiosh_binary()
    return subprocess.run(
        [bin_path, *args],
        capture_output=True,
        text=True,
        timeout=timeout_s,
    )


def parse_json_output(res: subprocess.CompletedProcess) -> dict:
    for line in reversed(res.stdout.splitlines()):
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                return json.loads(line)
            except Exception:
                pass
    raise ValueError(f"Could not find JSON in stdout:\nSTDOUT:\n{res.stdout}\nSTDERR:\n{res.stderr}")


def test_sandbox_help():
    # Bare command
    r = run_aiosh("sandbox")
    assert r.returncode == 0
    assert "aiosh sandbox" in r.stdout
    assert "profiles" in r.stdout
    assert "probe" in r.stdout
    assert "exec" in r.stdout

    # With --help
    r = run_aiosh("sandbox", "--help")
    assert r.returncode == 0
    assert "aiosh sandbox" in r.stdout

    # Alias `sb`
    r = run_aiosh("sb", "-h")
    assert r.returncode == 0
    assert "aiosh sandbox" in r.stdout


def test_sandbox_profiles():
    # Human readable
    r = run_aiosh("sandbox", "profiles")
    assert r.returncode == 0
    assert "Sandbox Profiles" in r.stdout
    assert "standard" in r.stdout
    assert "strict" in r.stdout
    assert "permissive" in r.stdout

    # JSON mode
    r = run_aiosh("sandbox", "profiles", "--json")
    assert r.returncode == 0
    data = parse_json_output(r)
    assert data["code"] == 0
    assert isinstance(data["data"], list)
    names = {p["name"] for p in data["data"]}
    assert "standard" in names
    assert "strict" in names
    assert "permissive" in names


def test_sandbox_probe():
    # Human readable
    r = run_aiosh("sandbox", "probe")
    assert r.returncode == 0
    assert "Host Sandbox Capabilities" in r.stdout
    assert "Landlock LSM" in r.stdout

    # JSON mode
    r = run_aiosh("sandbox", "probe", "--json")
    assert r.returncode == 0
    data = parse_json_output(r)
    assert data["code"] == 0
    caps = data["data"]
    assert "landlock_supported" in caps
    assert "seccomp_bpf_supported" in caps
    assert "no_new_privs_supported" in caps
    assert "platform" in caps


def test_sandbox_exec_validation():
    # 1. Missing delimiter
    r = run_aiosh("sandbox", "exec", "python", "-c", "print(1)")
    assert r.returncode == 2

    # 2. Missing command binary after delimiter
    r = run_aiosh("sandbox", "exec", "--")
    assert r.returncode == 2

    # 3. Directory traversal in --cwd
    r = run_aiosh("sandbox", "exec", "--cwd", "../secret", "--", "python", "-c", "print(1)")
    assert r.returncode == 2

    # 4. Unknown profile
    r = run_aiosh("sandbox", "exec", "--profile", "bogus_profile_xyz", "--", "python", "-c", "print(1)")
    assert r.returncode == 2


def test_sandbox_exec_success():
    # Human mode execution
    r = run_aiosh("sandbox", "exec", "--profile", "permissive", "--", sys.executable, "-c", "print('hello_sandbox_cli')")
    assert r.returncode == 0
    assert "hello_sandbox_cli" in r.stdout

    # JSON envelope mode execution
    r = run_aiosh("sandbox", "exec", "--json", "--profile", "permissive", "--", sys.executable, "-c", "print('json_mode_sandbox')")
    assert r.returncode == 0
    data = parse_json_output(r)
    assert data["code"] == 0
    res = data["data"]
    assert res["exit_code"] == 0
    assert "json_mode_sandbox" in res["stdout"]


def test_sandbox_unknown_subcommand():
    r = run_aiosh("sandbox", "unknown_sub")
    assert r.returncode == 2

    r_json = run_aiosh("sandbox", "unknown_sub", "--json")
    assert r_json.returncode == 2
    data = parse_json_output(r_json)
    assert data["code"] == 2
    assert data["error"]["code"] == "UNKNOWN_SUBCOMMAND"


if __name__ == "__main__":
    test_sandbox_help()
    test_sandbox_profiles()
    test_sandbox_probe()
    test_sandbox_exec_validation()
    test_sandbox_exec_success()
    test_sandbox_unknown_subcommand()
    print("ALL 6 SANDBOX CLI TESTS PASSED!")
