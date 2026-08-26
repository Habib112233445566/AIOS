#!/usr/bin/env python3
"""Task-docs invariant checker — SCAFFOLD (T-00093).

Bodies fail loudly until T-00094. Contract:
docs/tasks/evidence/T-00092-spec.md. Deterministic, stdlib-only,
read-only; output/exit parity with tools/check_security_policy.py.

Checks (C1..C6):
  C1 spec-health        SPEC exists, no TODO markers
  C2 component sections ### 8.1..8.6 with frozen epic ranges present
  C3 referenced paths   backticked repo-relative paths in SPEC resolve
                        (excluding fenced blocks + evidence/x.md)
  C4 phase map          MASTER_TASK_LEDGER.md table == jsonl phases
  C5 index health       GOALS.md / docs/README.md: no TODO; links resolve
  C6 no volatile counts CI <n>/<n> forbidden in living docs
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SPEC = ROOT / "docs" / "SPEC-TASK-LEDGER.md"
INDEX_MD = ROOT / "docs" / "tasks" / "MASTER_TASK_LEDGER.md"
LEDGER_JSONL = ROOT / "docs" / "tasks" / "MASTER_TASK_LEDGER.jsonl"
GOALS = ROOT / "docs" / "tasks" / "GOALS.md"
DOCS_README = ROOT / "docs" / "README.md"

PASS, FAIL = "[✓]", "[✗]"

#: Frozen epic ranges per §8.x subsection (spec C2).
COMPONENT_SECTIONS: dict[str, str] = {
    "8.1": "T-00031..T-00040",
    "8.2": "T-00041..T-00050",
    "8.3": "T-00051..T-00060",
    "8.4": "T-00061..T-00070",
    "8.5": "T-00071..T-00080",
    "8.6": "T-00081..T-00090",
}

#: Labels printed by main(), in order.
CHECKS: list[str] = ["C1", "C2", "C3", "C4", "C5", "C6"]

#: Scaffold transition flag: False until T-00094 fills the bodies.
IS_IMPLEMENTED: bool = True

#: T-00098 (F2): living-doc read cap — a pathological multi-GB doc must
#: fail loudly instead of spiking memory. Generous vs. the real tree
#: (SPEC ≈ 26 KiB).
MAX_DOC_BYTES: int = 16 * 1024 * 1024


def _read(path: Path) -> str:
    """Read a required artifact with a hard size cap (T-00098/F2).
    Raises ValueError (named) on cap violation; FileNotFoundError for
    missing files (callers convert either to a named FAIL)."""
    size = path.stat().st_size
    if size > MAX_DOC_BYTES:
        raise ValueError(
            f"{path} too large ({size} bytes > cap {MAX_DOC_BYTES} bytes)")
    return path.read_text(encoding="utf-8")


def _try_read(path: Path) -> tuple[str | None, str]:
    """_read with errors converted to a named failure string."""
    try:
        return _read(path), ""
    except FileNotFoundError:
        return None, f"missing required artifact: {path}"
    except ValueError as exc:  # cap violation
        return None, str(exc)
    except OSError as exc:
        return None, f"unreadable {path}: {exc}"


def strip_fenced_blocks(text: str) -> str:
    """Remove ```-fenced regions so example paths are not scanned.
    An unterminated fence strips to the end of the text."""
    return re.sub(r"```.*?(?:```|\Z)", "", text, flags=re.S)


def check_c1_spec_exists() -> tuple[bool, str]:
    """SPEC exists at its canonical path and carries no TODO markers."""
    text, err = _try_read(SPEC)
    if text is None:
        return False, err
    if "TODO" in text:
        lines = [i for i, l in enumerate(text.splitlines(), 1) if "TODO" in l]
        return False, f"TODO markers at lines {lines} in {SPEC.name}"
    return True, ""


def check_c2_component_sections() -> tuple[bool, str]:
    """Each frozen ### 8.x subsection exists and names its epic range.
    Extra 8.x sections are allowed (monotonic growth)."""
    text, err = _try_read(SPEC)
    if text is None:
        return False, err
    missing = []
    for sec, rng in COMPONENT_SECTIONS.items():
        hit = next((l for l in text.splitlines()
                    if l.startswith(f"### {sec}") ), None)
        if hit is None:
            missing.append(f"{sec}: heading absent")
        elif rng not in hit:
            missing.append(f"{sec}: range {rng} not in heading {hit.strip()!r}")
    if missing:
        return False, "; ".join(missing)
    return True, ""


_PATH_TOKEN = re.compile(r"`([^`\n]+)`")


def check_c3_referenced_paths() -> tuple[bool, str]:
    """Every backticked repo-relative path in SPEC resolves in-tree,
    excluding fenced blocks and the documented example placeholder."""
    raw, err = _try_read(SPEC)
    if raw is None:
        return False, err
    text = strip_fenced_blocks(raw)
    missing = []
    for tok in _PATH_TOKEN.findall(text):
        t = tok.strip()
        if t == "docs/tasks/evidence/x.md":      # example placeholder
            continue
        if not re.fullmatch(r"(?:docs|code|ci|tools)/[A-Za-z0-9_./-]+", t):
            continue                              # not a repo-relative path
        p = ROOT / t
        if "*" in t:
            import glob as _g
            if not _g.glob(str(p)):
                missing.append(t)
            continue
        if not p.exists():
            missing.append(t)
    if missing:
        return False, f"SPEC references missing paths: {sorted(set(missing))}"
    return True, ""


_PHASE_ROW = re.compile(
    r"^\|\s*Phase (\d+) — (.+?)\s*\|\s*T-(\d+) \.\. T-(\d+)\s*\|\s*$", re.M)


def check_c4_phase_map() -> tuple[bool, str]:
    """Index table rows equal the phases derived from the JSONL ledger
    (name equality; T-range equals min/max id per phase). Read-only:
    the generator owns both files."""
    if not INDEX_MD.is_file() or not LEDGER_JSONL.is_file():
        return False, f"missing {INDEX_MD.name} and/or {LEDGER_JSONL.name}"
    phases: dict[int, tuple[str, int, int]] = {}
    with open(LEDGER_JSONL, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rec = json.loads(line)
            num = int(rec["phase"].split("—")[0].strip().split()[1])
            name = rec["phase"].split("—", 1)[1].strip()
            tid = int(rec["id"])
            if num in phases:
                lo, hi, nm = phases[num][1], phases[num][2], phases[num][0]
                phases[num] = (nm, min(lo, tid), max(hi, tid))
            else:
                phases[num] = (name, tid, tid)
    index_text, err = _try_read(INDEX_MD)
    if index_text is None:
        return False, err
    rows = _PHASE_ROW.findall(index_text)
    problems = []
    if len(rows) != len(phases):
        problems.append(f"table has {len(rows)} rows, ledger has {len(phases)} phases")
    for num, name, lo, hi in rows:
        n = int(num)
        if n not in phases:
            problems.append(f"row Phase {n} absent from ledger")
            continue
        lname, lfirst, llast = phases[n]
        if lname != name.strip():
            problems.append(f"Phase {n}: name {name.strip()!r} != ledger {lname!r}")
        if (lfirst, llast) != (int(lo), int(hi)):
            problems.append(
                f"Phase {n}: range T-{int(lo)}..T-{int(hi)} != ledger "
                f"T-{lfirst}..T-{llast}")
    if problems:
        return False, "; ".join(problems)
    return True, ""


_MD_LINK = re.compile(r"\]\(([^)\s]+)\)")


def check_c5_index_health() -> tuple[bool, str]:
    """GOALS.md / docs/README.md: no TODO; README relative links resolve."""
    problems = []
    for path in (GOALS, DOCS_README):
        text, err = _try_read(path)
        if text is None:
            problems.append(err)
            continue
        if "TODO" in text:
            problems.append(f"TODO in {path.relative_to(ROOT)}")
    if DOCS_README.is_file():
        readme_text, err = _try_read(DOCS_README)
        if readme_text is None:
            problems.append(err)
        else:
            for link in _MD_LINK.findall(readme_text):
                target = link.split("#", 1)[0]
                if not target or target.startswith(("http://", "https://",
                                                    "mailto:")):
                    continue
                # T-00098 (F1): containment boundary is the REPO ROOT
                # (parent-relative links like ../START_HERE.md are
                # legitimate). Anything resolving outside the checkout —
                # absolute system paths, deep .. escapes, or symlinks
                # leaving the tree — is FLAGGED, never silently passed.
                resolved = (DOCS_README.parent / target).resolve()
                root_resolved = Path(ROOT).resolve()
                inside = resolved == root_resolved or root_resolved in resolved.parents
                if not inside:
                    problems.append(f"external link: ({link}) -> {resolved}")
                    continue
                if not (DOCS_README.parent / target).exists():
                    problems.append(f"broken link: ({link})")
    if problems:
        return False, "; ".join(problems)
    return True, ""


_VOLATILE = re.compile(r"CI \d+/\d+")


def check_c6_no_volatile_counts() -> tuple[bool, str]:
    """Living docs must not embed suite-count snapshots (they rot;
    observed in research). Frozen evidence files are out of scope."""
    hits = []
    for path in (SPEC, GOALS, DOCS_README):
        text, err = _try_read(path)
        if text is None:
            hits.append(f"{err}")
            continue
        for i, line in enumerate(text.splitlines(), 1):
            if _VOLATILE.search(line):
                hits.append(f"{path.relative_to(ROOT)}:{i}")
    if hits:
        return False, f"volatile 'CI n/n' counts in living docs: {hits}"
    return True, ""


def main() -> int:
    """Run C1..C6 collect-all; print [✓]/[✗] lines; exit 0/1."""
    failures = 0
    results = [
        ("C1 spec-health", check_c1_spec_exists()),
        ("C2 component sections", check_c2_component_sections()),
        ("C3 referenced paths", check_c3_referenced_paths()),
        ("C4 phase map", check_c4_phase_map()),
        ("C5 index health", check_c5_index_health()),
        ("C6 no volatile counts", check_c6_no_volatile_counts()),
    ]
    for label, (ok, detail) in results:
        print(f"{PASS if ok else FAIL} {label}")
        if not ok:
            print("   ", detail)
            failures += 1
    if failures:
        print(f"\n{failures} check(s) failed")
        return 1
    print("\nPASS: task docs criteria (C1..C6)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
