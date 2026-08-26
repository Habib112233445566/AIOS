#!/usr/bin/env python3
"""CI Smoke Orchestration — core service: read-only summary consumer
(T-00123 scaffold).

Contract: docs/tasks/evidence/T-00122-spec.md.
Strictly read-only host tooling: loads the run-summary artifact written
by tools/ci_run.py, validates strictly, renders reports / gate verdicts.
Never writes anything anywhere.

Scaffold stage: function BODIES fail loudly until T-00124; the CLI
surface is wired now so the interface has a live call site.
"""

from __future__ import annotations


import json
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from ci_suites import RESULTS_PATH, SUITE_NAMES, RunSummary  # noqa: E402

TOOL = "ci-service"
SCHEMA_VERSION_EXPECTED = 1


def resolve_path(explicit: str | None = None) -> str:
    """--file PATH > AIOSH_CI_RESULTS > default (spec §3)."""
    if explicit:
        return explicit
    return os.environ.get("AIOSH_CI_RESULTS") or RESULTS_PATH


_REQUIRED_KEYS = ("tool", "schema_version", "started_at", "finished_at",
                  "total", "passed", "failed", "all_pass", "results")
_VALID_STATUSES = ("pass", "fail", "timeout", "error")


def load_summary(path: str) -> RunSummary:
    """Strict validator-loader (spec §4). Any violation ⇒ ValueError
    naming the field; missing file ⇒ FileNotFoundError."""
    try:
        if os.path.getsize(path) > 1024 * 1024:
            raise ValueError("summary file exceeds 1MB size cap")
    except OSError as e:
        raise FileNotFoundError(f"could not stat {path}: {e}")
    with open(path, encoding="utf-8") as f:
        raw = json.load(f)
    if not isinstance(raw, dict):
        raise ValueError("'top level' must be a JSON object")
    for key in _REQUIRED_KEYS:
        if key not in raw:
            raise ValueError(f"missing required key {key!r}")
    if raw["schema_version"] != SCHEMA_VERSION_EXPECTED:
        raise ValueError(
            f"'schema_version' is {raw['schema_version']!r}, expected "
            f"{SCHEMA_VERSION_EXPECTED} (refusing best-effort parse)")
    total, passed, failed = (raw["total"], raw["passed"], raw["failed"])
    for name, val in (("total", total), ("passed", passed), ("failed", failed)):
        if not isinstance(val, int) or isinstance(val, bool) or val < 0:
            raise ValueError(f"{name!r} must be a non-negative int, got {val!r}")
    if passed + failed != total:
        raise ValueError(f"arithmetic incoherence: passed({passed}) + "
                         f"failed({failed}) != total({total})")
    expected_all_pass = (failed == 0 and total == len(SUITE_NAMES))
    if bool(raw["all_pass"]) != expected_all_pass:
        raise ValueError(
            f"'all_pass' is {raw['all_pass']!r} but failed=={failed} and "
            f"total=={total} vs registry size {len(SUITE_NAMES)} implies "
            f"{expected_all_pass}")
    results = raw["results"]
    if not isinstance(results, list):
        raise ValueError("'results' must be a list")
    last_index = -1
    for r in results:
        if not isinstance(r, dict):
            raise ValueError("each result must be an object")
        suite, index, status = r.get("suite"), r.get("index"), r.get("status")
        if suite not in SUITE_NAMES:
            raise ValueError(f"result 'suite' {suite!r} not in registry")
        if index != SUITE_NAMES.index(suite):
            raise ValueError(f"result 'index' {index!r} is not the registry "
                             f"position of {suite!r}")
        if index <= last_index:
            raise ValueError(f"'results' index {index!r} out of order "
                             f"(previous {last_index})")
        last_index = index
        if status not in _VALID_STATUSES:
            raise ValueError(f"result 'status' {status!r} invalid")
        rc = r.get("exit_code")
        if status in ("timeout", "error"):
            if rc is not None:
                raise ValueError(f"result 'exit_code' must be null for "
                                 f"{status} rows ({suite})")
        elif not isinstance(rc, int):
            raise ValueError(f"result 'exit_code' must be an int for "
                             f"{status} rows ({suite})")
        for ts_key in ("started_at", "finished_at"):
            tv = r.get(ts_key)
            if not isinstance(tv, str) or not tv.endswith("Z"):
                raise ValueError(f"result {ts_key!r} must end with 'Z' ({suite})")
        if not isinstance(r.get("duration_ms"), int) or r["duration_ms"] < 0:
            raise ValueError(f"result 'duration_ms' invalid ({suite})")
    return raw


def failure_rows(summary: RunSummary) -> list[dict]:
    """Rows with status != pass, in run order."""
    return [r for r in summary["results"] if r["status"] != "pass"]


def human_report(summary: RunSummary) -> str:
    """Stable line-format report (spec §5)."""
    verdict = "PASS" if summary["all_pass"] else "FAIL"
    lines = [
        f"CI run {summary['started_at']} .. {summary['finished_at']}: {verdict}",
        f"suites: {summary['total']} run, {summary['passed']} passed, "
        f"{summary['failed']} failed",
    ]
    for r in summary["results"]:
        if r["status"] == "pass":
            lines.append(f"  [ok ] {r['index']} {r['suite']} "
                         f"({r['duration_ms']} ms)")
        else:
            rc = r["exit_code"] if r["exit_code"] is not None else "-"
            lines.append(f"  [FAIL] {r['index']} {r['suite']} "
                         f"({r['duration_ms']} ms) exit={rc} "
                         f"log={r['log_path']}")
    return "\n".join(lines) + "\n"


def _usage_error(msg: str) -> int:
    print(f"{TOOL}: {msg}", file=sys.stderr)
    print("usage: ci_service.py <show|failures|check> [--file PATH]",
          file=sys.stderr)
    return 2


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    action = None
    file_arg = None
    rest = list(argv)
    while rest:
        tok = rest.pop(0)
        if tok == "--file":
            if not rest:
                return _usage_error("--file requires a value")
            file_arg = rest.pop(0)
        elif tok in ("show", "failures", "check"):
            if action is not None:
                return _usage_error(f"multiple actions given ({action}, {tok})")
            action = tok
        else:
            return _usage_error(f"unknown argument {tok!r}")
    if action is None:
        return _usage_error("missing action")

    path = resolve_path(file_arg)
    try:
        summary = load_summary(path)
    except FileNotFoundError as e:
        print(f"{TOOL}: {e}", file=sys.stderr)
        return 2
    except ValueError as e:
        print(f"{TOOL}: invalid summary artifact {path}: {e}", file=sys.stderr)
        return 2

    if action == "show":
        print(human_report(summary), end="" if human_report(summary).endswith("\n") else "\n")
        return 0
    if action == "failures":
        rows = failure_rows(summary)
        if not rows:
            print("no failed suites")
            return 0
        for r in rows:
            rc = r["exit_code"] if r["exit_code"] is not None else "-"
            print(f"[FAIL] {r['index']} {r['suite']} ({r['duration_ms']} ms) "
                  f"exit={rc} log={r['log_path']}")
        return 0
    # action == "check" — gate semantics (spec §6).
    complete = summary["total"] == len(SUITE_NAMES)
    if summary["all_pass"] and complete:
        print(f"ci-check: PASS ({summary['passed']}/{len(SUITE_NAMES)} suites)")
        return 0
    print(f"ci-check: FAIL ({summary['passed']}/{len(SUITE_NAMES)} suites, "
          f"{summary['failed']} failed)")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
