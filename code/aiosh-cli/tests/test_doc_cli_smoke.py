#!/usr/bin/env python3
"""CLI Smoke & Unit Test for Documentation Index Control (T-00435)"""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

def get_binary_path():
    candidates = [
        Path("code/aiosh-rust/target/debug/aiosh.exe"),
        Path("code/aiosh-rust/target/debug/aiosh"),
        Path("target/debug/aiosh.exe"),
        Path("target/debug/aiosh"),
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    return "aiosh"

def run_aiosh(*args):
    bin_path = get_binary_path()
    cmd = [bin_path, *args]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    return res

def parse_json_output(res):
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)

def test_doc_show_prose():
    res = run_aiosh("doc", "show")
    if res.returncode != 0:
        print(f"FAIL: aiosh doc show returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    assert "AIOS Documentation Index" in res.stdout
    print("PASS: aiosh doc show prose")

def test_doc_show_json():
    res = run_aiosh("doc", "show", "--json")
    if res.returncode != 0:
        print(f"FAIL: aiosh doc show --json returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    data = parse_json_output(res)
    assert data.get("ok") is True
    assert data.get("subcommand") == "doc show"
    assert "entries" in data.get("data", {})
    assert len(data["data"]["entries"]) > 0
    print("PASS: aiosh doc show --json")

def test_doc_check_prose():
    res = run_aiosh("doc", "check")
    if res.returncode != 0:
        print(f"FAIL: aiosh doc check returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    assert "Documentation link verification passed" in res.stdout
    print("PASS: aiosh doc check prose")

def test_doc_check_json():
    res = run_aiosh("doc", "check", "--json")
    if res.returncode != 0:
        print(f"FAIL: aiosh doc check --json returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    data = parse_json_output(res)
    assert data.get("ok") is True
    assert data.get("subcommand") == "doc check"
    report = data.get("data", {})
    assert report.get("is_valid") is True
    assert report.get("total_links_checked", 0) > 0
    print("PASS: aiosh doc check --json")

def test_doc_search():
    res = run_aiosh("doc", "search", "task")
    if res.returncode != 0:
        print(f"FAIL: aiosh doc search returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    assert "Documentation search results for 'task':" in res.stdout
    print("PASS: aiosh doc search")

def test_doc_search_json():
    res = run_aiosh("doc", "search", "task", "--json")
    if res.returncode != 0:
        print(f"FAIL: aiosh doc search task --json returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    data = parse_json_output(res)
    assert data.get("ok") is True
    assert data.get("subcommand") == "doc search"
    matches = data.get("data", [])
    assert len(matches) > 0
    print("PASS: aiosh doc search --json")

def test_doc_invalid_subcommand():
    res = run_aiosh("doc", "unknown_cmd")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"
    assert "usage: aiosh doc" in res.stderr
    print("PASS: aiosh doc invalid subcommand")

def test_doc_search_missing_query():
    res = run_aiosh("doc", "search")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"
    assert "usage: aiosh doc search" in res.stderr
    print("PASS: aiosh doc search missing query")

def test_doc_check_broken_links_negative():
    with tempfile.TemporaryDirectory() as temp_dir:
        docs_dir = Path(temp_dir) / "docs"
        docs_dir.mkdir(parents=True, exist_ok=True)
        (docs_dir / "tasks").mkdir(parents=True, exist_ok=True)
        
        readme = docs_dir / "README.md"
        readme.write_text("# Readme\n[Broken](nonexistent_target.md)", encoding="utf-8")
        (docs_dir / "SPEC-TASK-LEDGER.md").write_text("# Spec\nNo links", encoding="utf-8")
        (docs_dir / "tasks" / "GOALS.md").write_text("# Goals\nNo links", encoding="utf-8")

        res = run_aiosh("doc", "check", "--repo", temp_dir, "--json")
        assert res.returncode == 1, f"Expected 1, got {res.returncode}"
        data = parse_json_output(res)
        assert data.get("ok") is False
        report = data.get("data", {})
        assert report.get("is_valid") is False
        assert len(report.get("broken_links", [])) == 1
    print("PASS: aiosh doc check broken link detection negative test")

def test_doc_custom_config_valid():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump({
            "version": "1.0.0",
            "root_dirs": ["docs"],
            "include_extensions": [".md"],
            "exclude_patterns": [],
            "enforce_strict_links": True
        }, f)
        temp_path = f.name
    try:
        res = run_aiosh("doc", "show", "--config", temp_path, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data.get("ok") is True
    finally:
        os.unlink(temp_path)
    print("PASS: aiosh doc custom config valid")

def test_doc_custom_config_missing_negative():
    res = run_aiosh("doc", "show", "--config", "nonexistent_config_12345.json", "--json")
    assert res.returncode == 1
    data = parse_json_output(res)
    assert data.get("ok") is False
    print("PASS: aiosh doc custom config missing negative test")

if __name__ == "__main__":
    test_doc_show_prose()
    test_doc_show_json()
    test_doc_check_prose()
    test_doc_check_json()
    test_doc_search()
    test_doc_search_json()
    test_doc_invalid_subcommand()
    test_doc_search_missing_query()
    test_doc_check_broken_links_negative()
    test_doc_custom_config_valid()
    test_doc_custom_config_missing_negative()
    print("PASS: test_doc_cli_smoke.py")
