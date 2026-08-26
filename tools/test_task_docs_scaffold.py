#!/usr/bin/env python3
"""Scaffold/interface test for tools/check_task_docs.py (T-00093).

Proves: the module imports cleanly, every spec'd interface exists with
the right shape, and unimplemented bodies fail LOUDLY
(NotImplementedError) until T-00094. Behavioral checks land in the
T-00095 unit suite.
"""
import importlib.util
import inspect
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MOD = os.path.join(HERE, "check_task_docs.py")

spec = importlib.util.spec_from_file_location("check_task_docs", MOD)
assert spec is not None and spec.loader is not None, f"cannot load {MOD}"
cd = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cd)

REQUIRED = {
    "strip_fenced_blocks": 1,
    "check_c1_spec_exists": 0,
    "check_c2_component_sections": 0,
    "check_c3_referenced_paths": 0,
    "check_c4_phase_map": 0,
    "check_c5_index_health": 0,
    "check_c6_no_volatile_counts": 0,
    "main": 0,
}

fails = []
for name, min_pos in REQUIRED.items():
    fn = getattr(cd, name, None)
    if fn is None:
        fails.append(f"missing: {name}")
        continue
    sig = inspect.signature(fn)
    pos = sum(1 for p in sig.parameters.values()
              if p.kind in (p.POSITIONAL_ONLY, p.POSITIONAL_OR_KEYWORD))
    if pos < min_pos:
        fails.append(f"{name}: positional params {pos} < required {min_pos}")

# Constants from spec §2.
assert cd.CHECKS == ["C1", "C2", "C3", "C4", "C5", "C6"], cd.CHECKS
assert set(cd.COMPONENT_SECTIONS) == {f"8.{i}" for i in range(1, 7)}
for artifact in (cd.SPEC, cd.INDEX_MD, cd.LEDGER_JSONL, cd.GOALS, cd.DOCS_README):
    if not artifact.is_absolute():
        fails.append(f"{artifact} must resolve to an absolute path")

# Transition-aware body contract: before T-00094 every check body must
# fail loudly (NotImplementedError); after implementation each must
# return a (bool, str) tuple. `IS_IMPLEMENTED` is the declared era.
implemented = getattr(cd, "IS_IMPLEMENTED", False)
for name in ("check_c1_spec_exists", "check_c2_component_sections",
             "check_c3_referenced_paths", "check_c4_phase_map",
             "check_c5_index_health", "check_c6_no_volatile_counts"):
    try:
        result = getattr(cd, name)()
    except NotImplementedError:
        if implemented:
            fails.append(f"{name}: still raises NotImplementedError but "
                         "IS_IMPLEMENTED is True")
        continue
    except Exception as e:
        fails.append(f"{name}: wrong failure type {type(e).__name__}: {e}")
        continue
    if not implemented:
        fails.append(f"{name}: returned {result!r} while IS_IMPLEMENTED is False")
    elif not (isinstance(result, tuple) and len(result) == 2
              and isinstance(result[0], bool) and isinstance(result[1], str)):
        fails.append(f"{name}: bad return shape {result!r} (want (bool, str))")

try:
    stripped = cd.strip_fenced_blocks("before```docs/x.md```after")
except NotImplementedError:
    if implemented:
        fails.append("strip_fenced_blocks: still raises NotImplementedError")
    stripped = None
if stripped is not None and stripped != "beforeafter":
    fails.append(f"strip_fenced_blocks: got {stripped!r}, want 'beforeafter'")

if fails:
    print("FAIL:")
    for f in fails:
        print("  -", f)
    sys.exit(1)
print("PASS: task-docs scaffold — interfaces present, bodies fail loudly")
