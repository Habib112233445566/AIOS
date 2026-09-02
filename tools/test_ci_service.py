"""CI core service unit tests (T-00125).

Standalone test for tools/ci_service.py following the repo smoke-test
style. Builds synthetic summary artifacts in temp dirs; never runs CI.

Coverage (spec T-00122):
  X1  valid artifact: show stable lines; check gate exit 0
  X2  seeded failures: check exit 1 with counts; failures lists rows
  X3  strict load rejections (each exit 2, message names the field):
      missing file, corrupt JSON, wrong schema_version, missing key,
      incoherent math, wrong all_pass, bad status, non-Z timestamp,
      out-of-order index, null exit on fail row
  X4  usage errors exit 2 (no action, unknown token, double action,
      --file without value)
  X5  boundary: incomplete run (total<19, failed=0) is loadable but the
      gate FAILS it (all_pass must be false by construction)
  X6  failures with zero failures prints the no-failures line
  X7  mutation sensitivity: disabled version check is caught
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
SERVICE = HERE / "ci_service.py"
PASS, FAIL = "[PASS]", "[FAIL]"

from ci_suites import SUITE_NAMES
NAMES = SUITE_NAMES
N_SUITES = len(SUITE_NAMES)


def run(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run([sys.executable, str(SERVICE), *args],
                          capture_output=True, text=True, timeout=30)


def make_artifact(path: Path, *, n: int = N_SUITES, fail_at: int | None = None,
                  **overrides) -> None:
    results = []
    for i in range(n):
        status = "fail" if i == fail_at else "pass"
        results.append({
            "suite": NAMES[i], "index": i, "status": status,
            "exit_code": 2 if status == "fail" else 0,
            "duration_ms": 100 + i, "started_at": "2026-08-23T00:00:00Z",
            "finished_at": "2026-08-23T00:01:00Z",
            "log_path": f"/tmp/aiosh-ci-{NAMES[i]}.log",
        })
    failed = sum(1 for r in results if r["status"] != "pass")
    doc = {
        "tool": "aios-ci-orchestrator", "schema_version": 1,
        "started_at": "2026-08-23T00:00:00Z",
        "finished_at": "2026-08-23T00:30:00Z",
        "total": n, "passed": n - failed, "failed": failed,
        "all_pass": failed == 0 and n == N_SUITES, "results": results,
    }
    doc.update(overrides)
    path.write_text(json.dumps(doc), encoding="utf-8")


def main() -> int:
    tmp = Path(tempfile.mkdtemp(prefix="ci-service-x-"))
    good = tmp / "good.json"
    make_artifact(good)

    # ---- X1 — happy path
    p = run("show", "--file", str(good))
    ok = (p.returncode == 0
          and p.stdout.startswith("CI run 2026-08-23T00:00:00Z .. "
                                  "2026-08-23T00:30:00Z: PASS")
          and f"suites: {N_SUITES} run, {N_SUITES} passed, 0 failed" in p.stdout
          and "  [ok ] 0 rust_smoke (100 ms)" in p.stdout)
    if ok:
        p = run("check", "--file", str(good))
        ok = (p.returncode == 0
              and p.stdout.strip() == f"ci-check: PASS ({N_SUITES}/{N_SUITES} suites)")
    print(f"{PASS if ok else FAIL} X1 show stable lines + check gate exit 0")
    if not ok:
        print(p.stdout, p.stderr)
        return 1

    # ---- X2 — seeded failure
    bad = tmp / "bad.json"
    make_artifact(bad, fail_at=5)
    p = run("check", "--file", str(bad))
    ok = p.returncode == 1 and f"FAIL ({N_SUITES-1}/{N_SUITES} suites, 1 failed)" in p.stdout
    if ok:
        p = run("failures", "--file", str(bad))
        ok = (p.returncode == 0
              and "[FAIL] 5 pentest_smoke" in p.stdout
              and "exit=2" in p.stdout and "log=/tmp/aiosh-ci-pentest_smoke.log"
              in p.stdout)
    print(f"{PASS if ok else FAIL} X2 gate exit 1 + failure projection")
    if not ok:
        print(p.stdout, p.stderr)
        return 1

    # ---- X3 — strict rejections (each must exit 2 and name the field)
    def expect_reject(name: str, doc_mut, frag: str) -> bool:
        f = tmp / f"rej-{name}.json"
        make_artifact(f)
        doc = json.loads(f.read_text())
        doc_mut(doc)
        f.write_text(json.dumps(doc) if name != "corrupt"
                     else "{not json")
        p = run("check", "--file", str(f))
        if p.returncode != 2 or frag not in (p.stderr + p.stdout):
            print(f"{FAIL} X3/{name}: exit={p.returncode} "
                  f"want frag {frag!r} got: {p.stderr.strip()[:120]}")
            return False
        return True

    ok = expect_reject("missing", lambda d: os_unlink_key(d), "missing required key")
    ok = ok and expect_reject(
        "version", lambda d: d.update(schema_version=2),
        "schema_version")
    ok = ok and expect_reject(
        "math", lambda d: d.update(passed=3), "arithmetic incoherence")
    ok = ok and expect_reject(
        "allpass", lambda d: d.update(failed=1, passed=N_SUITES - 1, all_pass=True),
        "'all_pass'")
    ok = ok and expect_reject(
        "status", lambda d: d["results"][0].update(status="wat"), "'status'")
    ok = ok and expect_reject(
        "ts", lambda d: d["results"][1].update(started_at="2026-08-23 00:00"),
        "started_at")
    ok = ok and expect_reject(
        "order", lambda d: d["results"].reverse(), "out of order")
    ok = ok and expect_reject(
        "nullexit", lambda d: d["results"][0].update(exit_code=None),
        "must be an int")
    ok = ok and expect_reject(
        "suite", lambda d: d["results"][0].update(suite="nope"), "not in registry")
    # corrupt JSON + missing file
    corrupt = tmp / "corrupt.json"
    corrupt.write_text("{not json")
    p = run("check", "--file", str(corrupt))
    ok = ok and p.returncode == 2
    p = run("check", "--file", str(tmp / "absent.json"))
    ok = ok and p.returncode == 2
    print(f"{PASS if ok else FAIL} X3 strict load rejections (11 cases, "
          f"exit 2, field-naming)")
    if not ok:
        return 1

    # ---- X4 — usage errors
    ok = run().returncode == 2
    ok = ok and run("bogus").returncode == 2
    ok = ok and run("show", "check").returncode == 2
    ok = ok and run("show", "--file").returncode == 2
    print(f"{PASS if ok else FAIL} X4 usage errors exit 2")
    if not ok:
        return 1

    # ---- X5 — incomplete run boundary (fail-fast artifact, failed=0)
    part = tmp / "partial.json"
    make_artifact(part, n=5)
    doc = json.loads(part.read_text())
    assert doc["all_pass"] is False  # by construction (total<N_SUITES)
    p = run("check", "--file", str(part))
    ok = p.returncode == 1 and f"FAIL (5/{N_SUITES} suites" in p.stdout
    print(f"{PASS if ok else FAIL} X5 incomplete-but-clean run fails the gate")
    if not ok:
        print(p.stdout, p.stderr)
        return 1

    # ---- X6 — zero-failure projection line
    p = run("failures", "--file", str(good))
    ok = p.returncode == 0 and p.stdout.strip() == "no failed suites"
    print(f"{PASS if ok else FAIL} X6 zero-failure projection line")
    if not ok:
        return 1

    # ---- X7 — mutation sensitivity
    src = SERVICE.read_text(encoding="utf-8")
    mut = src.replace("if raw[\"schema_version\"] != SCHEMA_VERSION_EXPECTED:",
                      "if False and raw[\"schema_version\"] != SCHEMA_VERSION_EXPECTED:")
    assert mut != src
    backup = tmp / "service.bak"
    backup.write_text(src, encoding="utf-8")
    SERVICE.write_text(mut, encoding="utf-8")
    try:
        p = run("check", "--file", str(tmp / "rej-version.json"))
        ok = p.returncode == 0  # mutant wrongly accepts v2 artifact
    finally:
        SERVICE.write_text(backup.read_text(encoding="utf-8"), encoding="utf-8")
    if not ok:
        print(f"{FAIL} X7 mutant was still rejected — suite not sensitive")
        return 1
    p = run("check", "--file", str(good))
    ok = p.returncode == 0
    print(f"{PASS if ok else FAIL} X7 mutation sensitivity proven "
          f"(version-check neuter accepted v2; restored green)")
    if not ok:
        return 1

    print("PASS: ci_service unit tests (X1..X7)")
    return 0


def os_unlink_key(d: dict) -> None:
    del d["passed"]


if __name__ == "__main__":
    raise SystemExit(main())
