#!/usr/bin/env python3
"""Task completion CLI — thin wrapper over tools/task_ledger.py (spec §3).

Usage:
  python3 tools/complete_task.py <task_id> [--note "..."]

Mechanically enforces the NO-SKIP law: only the current next_task can be
completed; the pointer advances by exactly 1; every completion is appended
to docs/tasks/COMPLETIONS.jsonl (append-only event log).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from task_ledger import main  # noqa: E402

if __name__ == "__main__":
    raise SystemExit(main())
