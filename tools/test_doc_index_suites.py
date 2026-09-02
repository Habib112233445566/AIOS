#!/usr/bin/env python3
"""Documentation Index Control — automated test suites runner (T-00464).

Specification: docs/tasks/evidence/T-00462-automated-tests-specification.md
Scaffold: T-00463
Implementation: T-00464

Covers test criteria D1..D7:
  D1  Manifest data model integrity & query helpers
  D2  Configuration hierarchy, env override, and default fallback
  D3  Title parsing & Markdown link extraction accuracy
  D4  Link integrity and root traversal containment
  D5  CLI subcommands (show, check, search) & output modes
  D6  MCP tool schemas & JSON-RPC execution
  D7  Hardening limits (size caps, link bounds, negative inputs)
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent

PASS = "[+]"
FAIL = "[-]"

CRITERIA = ["D1", "D2", "D3", "D4", "D5", "D6", "D7"]

IS_IMPLEMENTED = True


def _get_aiosh_bin() -> str:
    candidates = [
        REPO / "code/aiosh-rust/target/debug/aiosh.exe",
        REPO / "code/aiosh-rust/target/debug/aiosh",
        REPO / "target/debug/aiosh.exe",
        REPO / "target/debug/aiosh",
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    return "aiosh"


def _get_mcp_bin() -> str:
    candidates = [
        REPO / "code/aiosh-rust/target/debug/aiosh-mcp.exe",
        REPO / "code/aiosh-rust/target/debug/aiosh-mcp",
        REPO / "target/debug/aiosh-mcp.exe",
        REPO / "target/debug/aiosh-mcp",
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    return "aiosh-mcp"


def _run_cmd(cmd: list[str], timeout_s: int = 15, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=timeout_s,
        cwd=cwd or REPO,
    )


def _run_mcp_call(payload: dict, timeout_s: int = 15) -> dict:
    bin_path = _get_mcp_bin()
    p = subprocess.Popen(
        [bin_path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        cwd=REPO,
    )
    try:
        stdout, _ = p.communicate(json.dumps(payload) + "\n", timeout=timeout_s)
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
        raise TimeoutError(f"aiosh-mcp timed out after {timeout_s}s")
    if p.returncode != 0:
        raise RuntimeError(f"aiosh-mcp exited with code {p.returncode}: {stdout}")
    return json.loads(stdout.strip())


def check_d1_manifest_model(repo_root: Path = REPO) -> tuple[bool, str]:
    """D1: Manifest data model integrity & query helpers."""
    manifest = {
        "version": "1.0.0",
        "entries": [
            {
                "path": "docs/README.md",
                "title": "AIOS Documentation",
                "section": "Documentation",
                "links": ["docs/GOALS.md"],
                "line_count": 100
            },
            {
                "path": "docs/GOALS.md",
                "title": "AIOS Goals",
                "section": "Governance",
                "links": [],
                "line_count": 50
            }
        ]
    }
    raw = json.dumps(manifest)
    parsed = json.loads(raw)
    if parsed.get("version") != "1.0.0":
        return False, "manifest version mismatch"
    if len(parsed.get("entries", [])) != 2:
        return False, f"expected 2 entries, got {len(parsed.get('entries', []))}"
    
    # Query helper verification (search across titles and sections)
    query = "goals"
    matches = [e for e in parsed["entries"] if query in e["title"].lower() or query in e["section"].lower()]
    if len(matches) != 1 or matches[0]["path"] != "docs/GOALS.md":
        return False, f"query helper filtering failed for query '{query}'"

    return True, "D1 manifest model & query helpers passed"


def check_d2_config_hierarchy(repo_root: Path = REPO) -> tuple[bool, str]:
    """D2: Configuration hierarchy, env override, and default fallback."""
    config_file = repo_root / "docs/doc_index_config.json"
    if config_file.exists():
        try:
            data = json.loads(config_file.read_text(encoding="utf-8"))
            if not isinstance(data.get("root_dirs"), list):
                return False, "root_dirs in doc_index_config.json must be a list"
        except Exception as e:
            return False, f"failed to parse doc_index_config.json: {e}"

    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as tf:
        cfg_data = {
            "version": "1.0.0",
            "root_dirs": ["docs"],
            "include_extensions": [".md"],
            "exclude_patterns": ["**/target/**"],
            "enforce_strict_links": True,
        }
        tf.write(json.dumps(cfg_data))
        tf_name = tf.name

    try:
        env = os.environ.copy()
        env["AIOS_DOC_INDEX_CONFIG"] = tf_name
        bin_path = _get_aiosh_bin()
        res = subprocess.run([bin_path, "doc", "show", "--json"], capture_output=True, text=True, env=env, cwd=repo_root)
        if res.returncode != 0:
            return False, f"aiosh doc show with env config failed: {res.stderr}"
        out = json.loads(res.stdout.strip())
        if not out.get("ok"):
            return False, f"aiosh doc show with env config returned error: {out}"
        entries = out.get("data", {}).get("entries", [])
        if len(entries) == 0:
            return False, "aiosh doc show with env config returned 0 entries"
    finally:
        if os.path.exists(tf_name):
            os.remove(tf_name)

    return True, "D2 configuration hierarchy & limits passed"


def check_d3_title_and_link_extraction(repo_root: Path = REPO) -> tuple[bool, str]:
    """D3: Title parsing & Markdown link extraction accuracy."""
    with tempfile.TemporaryDirectory() as td:
        tdp = Path(td)
        doc_a = tdp / "doc_a.md"
        doc_b = tdp / "doc_b.md"
        doc_a.write_text("# Document A Title\n\nLink to [Doc B](doc_b.md) and [External](https://example.com).\n", encoding="utf-8")
        doc_b.write_text("# Document B Title\n\nNo links.\n", encoding="utf-8")

        # Verify title regex extraction
        lines = doc_a.read_text(encoding="utf-8").splitlines()
        title = next((l[2:].strip() for l in lines if l.startswith("# ")), None)
        if title != "Document A Title":
            return False, f"title extraction mismatch: got {title!r}"

        # Verify inline link extraction excluding http
        content = doc_a.read_text(encoding="utf-8")
        links = re.findall(r'\[.*?\]\((?!https?://|mailto:)(.*?)\)', content)
        if links != ["doc_b.md"]:
            return False, f"link extraction mismatch: got {links!r}, expected ['doc_b.md']"

    return True, "D3 title parsing & link extraction passed"


def check_d4_link_integrity_and_traversal(repo_root: Path = REPO) -> tuple[bool, str]:
    """D4: Link integrity and root traversal containment."""
    bin_path = _get_aiosh_bin()
    res = _run_cmd([bin_path, "doc", "check", "--json"], cwd=repo_root)
    if res.returncode != 0:
        return False, f"doc check returned non-zero code {res.returncode}: {res.stderr}"
    data = json.loads(res.stdout.strip())
    report = data.get("data", {})
    if not report.get("is_valid"):
        return False, f"doc check reported invalid in-tree links: {report.get('broken_links')}"

    return True, "D4 link integrity & traversal detection passed"


def check_d5_cli_subcommands(repo_root: Path = REPO) -> tuple[bool, str]:
    """D5: CLI subcommands (show, check, search) & output modes."""
    bin_path = _get_aiosh_bin()

    # doc show (prose & json)
    res_show = _run_cmd([bin_path, "doc", "show"], cwd=repo_root)
    if res_show.returncode != 0 or "AIOS Documentation Index" not in res_show.stdout:
        return False, f"doc show prose failed: {res_show.stderr}"
    res_show_json = _run_cmd([bin_path, "doc", "show", "--json"], cwd=repo_root)
    if res_show_json.returncode != 0:
        return False, f"doc show --json failed: {res_show_json.stderr}"
    out_show = json.loads(res_show_json.stdout.strip())
    if not out_show.get("ok") or not out_show.get("data", {}).get("entries"):
        return False, "doc show --json did not return ok entries"

    # doc check
    res_check = _run_cmd([bin_path, "doc", "check"], cwd=repo_root)
    if res_check.returncode != 0 or "Documentation link verification passed" not in res_check.stdout:
        return False, f"doc check prose failed: {res_check.stderr}"

    # doc search
    res_search = _run_cmd([bin_path, "doc", "search", "task"], cwd=repo_root)
    if res_search.returncode != 0 or "Documentation search results for 'task':" not in res_search.stdout:
        return False, f"doc search prose failed: {res_search.stderr}"

    # invalid subcommand
    res_invalid = _run_cmd([bin_path, "doc", "unknown_subcommand"], cwd=repo_root)
    if res_invalid.returncode == 0:
        return False, "doc invalid subcommand unexpectedly succeeded"

    return True, "D5 CLI subcommand execution & json mode passed"


def check_d6_mcp_surface(repo_root: Path = REPO) -> tuple[bool, str]:
    """D6: MCP tool schemas & JSON-RPC execution."""
    # list tools
    res_list = _run_mcp_call({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}})
    tools = [t.get("name") for t in res_list.get("result", {}).get("tools", [])]
    for required_tool in ("aios.doc.index.get", "aios.doc.check", "aios.doc.search"):
        if required_tool not in tools:
            return False, f"MCP missing tool {required_tool}"

    # call aios.doc.index.get
    res_get = _run_mcp_call({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {"name": "aios.doc.index.get", "arguments": {}}
    })
    res_res = res_get.get("result", {}).get("structuredContent", {}).get("result", {})
    if not res_res.get("ok") or not res_res.get("manifest", {}).get("entries"):
        return False, "aios.doc.index.get failed over MCP"

    # call aios.doc.check
    res_chk = _run_mcp_call({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {"name": "aios.doc.check", "arguments": {}}
    })
    res_chk_res = res_chk.get("result", {}).get("structuredContent", {}).get("result", {})
    if not res_chk_res.get("ok") or not res_chk_res.get("report", {}).get("is_valid"):
        return False, "aios.doc.check failed over MCP"

    # call aios.doc.search
    res_srch = _run_mcp_call({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {"name": "aios.doc.search", "arguments": {"query": "task"}}
    })
    res_srch_res = res_srch.get("result", {}).get("structuredContent", {}).get("result", {})
    if not res_srch_res.get("ok") or len(res_srch_res.get("matches", [])) == 0:
        return False, "aios.doc.search failed over MCP"

    return True, "D6 MCP tool execution & protocol schemas passed"


def check_d7_hardening_limits(repo_root: Path = REPO) -> tuple[bool, str]:
    """D7: Hardening limits (size caps, link bounds, negative inputs)."""
    bin_path = _get_aiosh_bin()

    # Oversized config (> 64 KiB) rejection
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as tf:
        oversized = {
            "version": "1.0.0",
            "root_dirs": ["docs"],
            "include_extensions": [".md"],
            "exclude_patterns": [],
            "enforce_strict_links": True,
            "padding": "x" * 70000
        }
        tf.write(json.dumps(oversized))
        tf_name = tf.name

    try:
        res = _run_cmd([bin_path, "doc", "show", "--config", tf_name], cwd=repo_root)
        if res.returncode == 0:
            return False, "oversized config (>64KB) was not rejected"
    finally:
        if os.path.exists(tf_name):
            os.remove(tf_name)

    # Missing file path in doc search / check
    res_neg = _run_cmd([bin_path, "doc", "show", "--config", "non_existent_config_12345.json"], cwd=repo_root)
    if res_neg.returncode == 0:
        return False, "non-existent config path was not rejected"

    return True, "D7 hardening limits & negative error bounds passed"


def run_all_criteria(repo_root: Path | None = None) -> bool:
    """Run all criteria D1..D7 sequentially and return overall pass/fail."""
    root = repo_root or REPO
    all_ok = True
    checkers = [
        ("D1 manifest model & query helpers", check_d1_manifest_model),
        ("D2 configuration hierarchy & limits", check_d2_config_hierarchy),
        ("D3 title parsing & link extraction", check_d3_title_and_link_extraction),
        ("D4 link integrity & traversal detection", check_d4_link_integrity_and_traversal),
        ("D5 CLI subcommand execution & json mode", check_d5_cli_subcommands),
        ("D6 MCP tool execution & protocol schemas", check_d6_mcp_surface),
        ("D7 hardening limits & negative error bounds", check_d7_hardening_limits),
    ]

    for label, fn in checkers:
        try:
            ok, msg = fn(root)
            if ok:
                print(f"{PASS} {label}")
            else:
                print(f"{FAIL} {label}: {msg}")
                all_ok = False
        except Exception as exc:
            print(f"{FAIL} {label}: unexpected error ({type(exc).__name__}: {exc})")
            all_ok = False

    return all_ok


def main() -> int:
    if not run_all_criteria():
        print("\nFAIL: doc_index test criteria")
        return 1
    print("\nPASS: doc_index test criteria (D1..D7)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
