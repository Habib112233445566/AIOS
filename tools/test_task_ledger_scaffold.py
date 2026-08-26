"""Scaffold/interface test for tools/task_ledger.py (T-00013/T-00014).

Proves: the module imports cleanly and every spec'd interface exists with
the right signature shape. Behavioral tests live in
tools/test_task_ledger.py.
"""
import importlib.util
import inspect
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MOD = os.path.join(HERE, "task_ledger.py")

spec = importlib.util.spec_from_file_location("task_ledger", MOD)
assert spec is not None and spec.loader is not None, f"cannot load {MOD}"
tl = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tl)

REQUIRED = {
    "load_state": 0,
    "save_state_atomic": 1,
    "append_event": 1,
    "read_events": 0,
    "rebuild_state": 0,
    "find_task_in_ledger": 1,
    "assert_ledger_invariants": 0,
    "acquire_lock": 0,
    "complete_task": 1,
    "block_task": 2,
    "unblock_task": 2,
    "skip_task": 2,
    "main": 0,
}

fails = []
for name, min_pos in REQUIRED.items():
    fn = getattr(tl, name, None)
    if fn is None:
        fails.append(f"missing: {name}")
        continue
    sig = inspect.signature(fn)
    pos = sum(1 for p in sig.parameters.values()
              if p.kind in (p.POSITIONAL_ONLY, p.POSITIONAL_OR_KEYWORD))
    if pos < min_pos:
        fails.append(f"{name}: positional params {pos} < required {min_pos}")

# constants from spec
assert tl.SCHEMA_VERSION == 2, "schema_version must be 2"
assert set(tl.VALID_EVENTS) == {"completed", "blocked", "unblocked", "pointer_reset"}

if fails:
    print("FAIL:")
    for f in fails:
        print("  -", f)
    sys.exit(1)
print("PASS: task_ledger scaffold — all interfaces present")
