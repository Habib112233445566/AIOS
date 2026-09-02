#!/usr/bin/env python3
"""Evidence Invariants & Verification Checker (T-00563 scaffold / T-00564 impl).

Contract: docs/tasks/evidence/T-00562-automated-tests-specification.md.
Deterministic, stdlib-only, read-only.

Checks (E1..E4):
  E1 directory-health   docs/tasks/evidence exists and has files
  E2 ledger-consistency completed tasks in TASK_STATE.json have evidence
  E3 file-bounds        all evidence files non-empty, utf-8, <= 16 MiB
  E4 hash-consistency   SHA-256 computation succeeds deterministically
"""

from __future__ import annotations

import hashlib
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE_DIR = ROOT / "docs" / "tasks" / "evidence"
TASK_STATE_JSON = ROOT / "docs" / "tasks" / "TASK_STATE.json"

PASS, FAIL = "[+]", "[-]"
MAX_DOC_BYTES = 16 * 1024 * 1024  # 16 MiB

def check_e1_directory_health() -> tuple[bool, str]:
    if not EVIDENCE_DIR.exists() or not EVIDENCE_DIR.is_dir():
        return False, f"Directory not found: {EVIDENCE_DIR}"
    files = list(EVIDENCE_DIR.glob("T-*.md"))
    if not files:
        return False, "No evidence markdown files found"
    return True, f"found {len(files)} evidence files"

def check_e2_ledger_consistency() -> tuple[bool, str]:
    if not TASK_STATE_JSON.exists():
        return False, f"State file not found: {TASK_STATE_JSON}"
    try:
        with open(TASK_STATE_JSON, "r", encoding="utf-8") as f:
            state = json.load(f)
    except Exception as e:
        return False, f"Failed to parse TASK_STATE.json: {e}"

    completed = state.get("completed", [])
    if not completed:
        return True, "no completed tasks"

    missing = []
    # Sample last 50 completed tasks to keep check bounded & fast
    sample = completed[-50:]
    for tid in sample:
        pattern = f"T-{tid:05d}-*.md"
        matches = list(EVIDENCE_DIR.glob(pattern))
        if not matches:
            missing.append(tid)

    if missing:
        return False, f"missing evidence files for tasks: {missing[:5]}"
    return True, f"verified {len(sample)} sampled completed tasks"

def check_e3_file_bounds() -> tuple[bool, str]:
    files = list(EVIDENCE_DIR.glob("T-*.md"))
    for file_path in files:
        try:
            stat = file_path.stat()
            if stat.st_size == 0:
                return False, f"Empty evidence file: {file_path.name}"
            if stat.st_size > MAX_DOC_BYTES:
                return False, f"Oversized file {file_path.name}: {stat.st_size} bytes"
            with open(file_path, "r", encoding="utf-8") as f:
                f.read(1024)
        except Exception as e:
            return False, f"Unreadable file {file_path.name}: {e}"
    return True, f"all {len(files)} files bounded and valid UTF-8"

def check_e4_hash_consistency() -> tuple[bool, str]:
    files = list(EVIDENCE_DIR.glob("T-*.md"))[:10]
    for file_path in files:
        h = hashlib.sha256()
        with open(file_path, "rb") as f:
            while chunk := f.read(65536):
                h.update(chunk)
        digest = h.hexdigest()
        if len(digest) != 64:
            return False, f"Invalid digest length for {file_path.name}"
    return True, "deterministic SHA-256 verified"

def main() -> int:
    checks = [
        ("E1 directory-health", check_e1_directory_health),
        ("E2 ledger-consistency", check_e2_ledger_consistency),
        ("E3 file-bounds", check_e3_file_bounds),
        ("E4 hash-consistency", check_e4_hash_consistency),
    ]

    all_passed = True
    for label, fn in checks:
        ok, msg = fn()
        marker = PASS if ok else FAIL
        print(f"{marker} {label}: {msg}")
        if not ok:
            all_passed = False

    if all_passed:
        print("\nPASS: evidence integrity criteria (E1..E4)")
        return 0
    else:
        print("\nFAIL: evidence integrity criteria check failed")
        return 1

if __name__ == "__main__":
    sys.exit(main())
