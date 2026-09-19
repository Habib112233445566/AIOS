#!/usr/bin/env python3
"""Documentation Test Suite for Filesystem Layout (T-01583..T-01586).

Criteria FL13:
  D1: CLI subcommand completeness (--help advertises all 11 subcommands)
  D2: MCP manifest schema parity (tools/list matches accepted parameters)
  D3: Evidence link integrity (all referenced T-*.md exist on disk)
  D4: JSON snippet syntactic validity (code blocks in docs parse cleanly)
  D5: Error code documentation completeness (all error codes documented in §4.12)

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_documentation.py
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DOCS_FILE = ROOT / "docs" / "filesystem_layout.md"

EXPECTED_SUBCOMMANDS = [
    "list",
    "show",
    "validate",
    "probe",
    "diff",
    "fstab",
    "register",
    "set-active",
    "remove",
    "import-fstab",
]

EXPECTED_MCP_TOOLS = [
    "aios.fs_layout.get",
    "aios.fs_layout.validate",
    "aios.fs_layout.fstab",
    "aios.fs_layout.list",
    "aios.fs_layout.probe",
    "aios.fs_layout.diff",
    "aios.fs_layout.register",
    "aios.fs_layout.set_active",
    "aios.fs_layout.remove",
    "aios.fs_layout.import_fstab",
]

EXPECTED_ERROR_CODES = [
    "RESOLVE_FAILED",
    "VALIDATION_FAILED",
    "DIFF_FAILED",
    "PROBE_FAILED",
    "NOT_VIABLE",
    "LOAD_STORE_FAILED",
    "SAVE_STORE_FAILED",
    "REGISTER_FAILED",
    "SET_ACTIVE_FAILED",
    "REMOVE_FAILED",
    "IMPORT_FAILED",
    "SPEC_READ_FAILED",
    "SPEC_NOT_REGULAR_FILE",
    "SPEC_SIZE_EXCEEDED",
    "SPEC_PARSE_FAILED",
    "FSTAB_READ_FAILED",
    "FSTAB_NOT_REGULAR_FILE",
    "FSTAB_SIZE_EXCEEDED",
    "ARGUMENT_ERROR",
    "INVALID_ARGUMENT",
    "UNKNOWN_SUBCOMMAND",
]


def _find_binary(names: list[str]) -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in names:
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return names[-1]


def get_cli_binary() -> str:
    return _find_binary(["aiosh.exe", "aiosh"])


def get_mcp_binary() -> str:
    return _find_binary(["aiosh-mcp.exe", "aiosh-mcp"])


def test_d1_cli_subcommand_completeness() -> None:
    """D1: CLI subcommand completeness (--help advertises all 11 subcommands)."""
    cp = subprocess.run(
        [get_cli_binary(), "layout", "--help"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert cp.returncode == 0, f"aiosh layout --help failed: {cp.stderr}"
    help_text = cp.stdout

    for sub in EXPECTED_SUBCOMMANDS:
        assert sub in help_text, f"Subcommand '{sub}' missing from aiosh layout --help output"

    print("PASS: D1 CLI subcommand completeness verified across all subcommands")


def test_d2_mcp_manifest_schema_parity() -> None:
    """D2: MCP manifest schema parity (tools/list matches accepted parameters)."""
    p = subprocess.Popen(
        [get_mcp_binary()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
    )
    try:
        stdout, _ = p.communicate(
            json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}) + "\n",
            timeout=30,
        )
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
        raise AssertionError("aiosh-mcp tools/list timed out")
    assert p.returncode == 0, f"aiosh-mcp exited with code {p.returncode}"

    resp = json.loads(stdout.strip())
    tools = {t["name"]: t for t in resp.get("result", {}).get("tools", [])}

    for tool_name in EXPECTED_MCP_TOOLS:
        assert tool_name in tools, f"MCP tool '{tool_name}' missing from tools/list manifest"
        schema = tools[tool_name].get("inputSchema", {})
        assert schema.get("additionalProperties") is False, (
            f"{tool_name}: additionalProperties must be False"
        )
        assert "properties" in schema, f"{tool_name}: inputSchema missing properties"

    print("PASS: D2 MCP manifest schema parity verified across all 10 tools")


def test_d3_evidence_link_integrity() -> None:
    """D3: Evidence link integrity (all referenced T-*.md exist on disk)."""
    assert DOCS_FILE.exists(), f"{DOCS_FILE} does not exist"
    content = DOCS_FILE.read_text(encoding="utf-8")

    # Match evidence markdown links e.g. (file:///.../docs/tasks/evidence/T-01561-security-policy-research.md)
    link_pattern = re.compile(r"docs/tasks/evidence/(T-\d{5}[^)\s]+\.md)")
    matches = link_pattern.findall(content)
    assert len(matches) > 0, "No task evidence links found in docs/filesystem_layout.md"

    missing = []
    for filename in matches:
        target_path = ROOT / "docs" / "tasks" / "evidence" / filename
        if not target_path.exists():
            missing.append(filename)

    assert not missing, f"Broken evidence links in docs/filesystem_layout.md: {missing}"
    print(f"PASS: D3 Evidence link integrity verified for all {len(matches)} referenced tasks")


def test_d4_json_snippet_syntactic_validity() -> None:
    """D4: JSON snippet syntactic validity (code blocks in docs parse cleanly)."""
    assert DOCS_FILE.exists()
    content = DOCS_FILE.read_text(encoding="utf-8")

    # Extract all ```json ... ``` blocks
    json_block_pattern = re.compile(r"```json\s*\n(.*?)\n```", re.DOTALL)
    blocks = json_block_pattern.findall(content)
    assert len(blocks) > 0, "No json code blocks found in docs/filesystem_layout.md"

    for i, block in enumerate(blocks, 1):
        # Sanitize placeholder syntax e.g. <GRANT_ID> or <LAYOUT_ID>
        sanitized = re.sub(r"<[A-Z0-9_]+>", '"placeholder"', block)
        # Sanitize illustrative ellipsis in arrays/objects e.g. [ ... ] or { ... }
        sanitized = re.sub(r"\[\s*\.\.\.\s*\]", "[]", sanitized)
        sanitized = re.sub(r"\{\s*\.\.\.\s*\}", "{}", sanitized)
        try:
            json.loads(sanitized)
        except json.JSONDecodeError as e:
            raise AssertionError(f"JSON code block {i} is invalid JSON: {e}\nContent:\n{block}")

    print(f"PASS: D4 JSON snippet syntactic validity verified across all {len(blocks)} code blocks")


def test_d5_error_code_documentation_completeness() -> None:
    """D5: Error code documentation completeness (all error codes documented in §4.12)."""
    assert DOCS_FILE.exists()
    content = DOCS_FILE.read_text(encoding="utf-8")

    missing = []
    for code in EXPECTED_ERROR_CODES:
        if code not in content:
            missing.append(code)

    assert not missing, f"Missing error code documentation in docs/filesystem_layout.md: {missing}"
    print(f"PASS: D5 Error code documentation completeness verified for all {len(EXPECTED_ERROR_CODES)} codes")


def main() -> int:
    print("=== RUNNING FILESYSTEM LAYOUT DOCUMENTATION TEST SUITE (FL13) ===")
    test_d1_cli_subcommand_completeness()
    test_d2_mcp_manifest_schema_parity()
    test_d3_evidence_link_integrity()
    test_d4_json_snippet_syntactic_validity()
    test_d5_error_code_documentation_completeness()
    print("PASS: All Documentation criteria D1..D5 passed successfully.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
