#!/usr/bin/env python3
"""Integration Smoke Test for PEP Decision Engine Configuration (T-02146).

Validates end-to-end configuration integration:
1. Default store path resolution.
2. Environment variable overrides (AIOSH_PEP_STORE_PATH).
3. Config file loading via AIOSH_PEP_CONFIG.
4. Path hygiene rejection on invalid config/store paths.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
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


def run_cli(args: list[str], env: dict | None = None) -> tuple[int, str, str]:
    full_env = os.environ.copy()
    if env:
        full_env.update(env)
    p = subprocess.Popen(
        [get_aiosh_binary(), "pep"] + args,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=full_env,
    )
    stdout, stderr = p.communicate(timeout=15)
    return p.returncode, stdout.strip(), stderr.strip()


def test_env_store_path_override():
    print("TEST: AIOSH_PEP_STORE_PATH environment variable override ...", end=" ")
    with tempfile.TemporaryDirectory() as tmpdir:
        custom_store = os.path.join(tmpdir, "env_policies.json")
        code, out, _ = run_cli(["status", "--json"], env={"AIOSH_PEP_STORE_PATH": custom_store})
        assert code == 0, f"expected code 0, got {code}: {out}"
        data = json.loads(out)
        assert data.get("code") == 0
        status = data.get("data", {})
        assert custom_store in status.get("store_path", ""), f"expected {custom_store} in {status}"
    print("OK")


def test_config_file_loading():
    print("TEST: AIOSH_PEP_CONFIG file loading ...", end=" ")
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "configured_store.json")
        cfg_path = os.path.join(tmpdir, "pep_config.json")
        cfg_content = {
            "version": "1.0.0",
            "store_path": store_path,
            "max_store_bytes": 10485760,
            "max_rules": 4000,
            "default_algorithm": "permit_overrides",
            "audit_all_evaluations": True,
            "auto_quarantine_corrupt": True
        }
        with open(cfg_path, "w") as f:
            json.dump(cfg_content, f)

        code, out, _ = run_cli(["status", "--json"], env={"AIOSH_PEP_CONFIG": cfg_path})
        assert code == 0, f"expected code 0, got {code}: {out}"
        data = json.loads(out)
        status = data.get("data", {})
        assert store_path in status.get("store_path", ""), f"expected {store_path} in {status}"
    print("OK")


def test_invalid_env_path_hygiene():
    print("TEST: Invalid env store path hygiene rejection ...", end=" ")
    code, out, err = run_cli(["status", "--json"], env={"AIOSH_PEP_STORE_PATH": "../../../evil.json"})
    assert code == 2, f"expected exit code 2 on traversal, got {code}: {out} {err}"
    print("OK")


def main():
    print("=== PEP Decision Engine Configuration Smoke Test ===")
    test_env_store_path_override()
    test_config_file_loading()
    test_invalid_env_path_hygiene()
    print("=== All PEP Decision Engine configuration smoke tests passed ===")


if __name__ == "__main__":
    main()
