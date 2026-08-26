#!/usr/bin/env python3
"""Task 10 — Audit-ring performance baseline.

Generates synthetic rings at 10k and 100k rows, times:
  - verify (live walk)
  - rotate (archive + bloom build)
  - verify_full (archive replay + live walk)

Uses the real audit_client + retention modules. Synthetic rows are
minimal (no classifier fields) to match Sprint-0/1 row shape.
"""
import os
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from aiosh_mcp import audit_client as ac
from aiosh_mcp import retention as rt


def seed_ring(conn, n: int) -> None:
    """Insert n rows quickly by computing hashes in Python directly."""
    prev = ac.GENESIS
    ts_base = "2026-01-01T00:00:00.000000Z"
    for i in range(n):
        proto = {
            "ts": f"2026-01-01T00:{(i//60)%60:02d}:{i%60:02d}.{i%1000000:06d}Z",
            "actor": "system",
            "actor_id": "system:benchmark",
            "tool": "test.bench",
            "command": f"bench row {i}",
            "args": {"i": i},
            "target": None,
            "outcome": "ok",
            "outcome_detail": None,
            "constitution_rev": None,
            "grant_token": None,
            "c_flags": {"c1": False, "c2": False, "c3": False, "c4": True},
            "prev_hash": prev,
        }
        h = ac.sha256_hex(prev + ac.canonical(proto))
        conn.execute(
            """INSERT INTO audit_ring
               (ts, actor, actor_id, tool, command, args_json, target,
                outcome, outcome_detail, constitution_rev, grant_token,
                c1, c2, c3, c4, prev_hash, hash)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
            (proto["ts"], proto["actor"], proto["actor_id"], proto["tool"],
             proto["command"], ac.canonical(proto["args"]), None,
             proto["outcome"], None, None, None,
             0, 0, 0, 1, prev, h))
        prev = h
        if (i + 1) % 5000 == 0:
            conn.commit()
    conn.commit()


def bench(n_rows: int, keep_rows: int) -> dict:
    tmpdir = tempfile.mkdtemp(prefix=f"aiosh-perf-{n_rows}-")
    db_path = os.path.join(tmpdir, "audit.db")
    conn = ac.open_db(db_path)

    t0 = time.perf_counter()
    seed_ring(conn, n_rows)
    seed_time = time.perf_counter() - t0

    # Verify (live, full walk)
    t0 = time.perf_counter()
    v = ac.verify_live(conn)
    verify_time = time.perf_counter() - t0
    assert v["ok"], f"verify failed at {n_rows}: {v}"

    # Rotate
    t0 = time.perf_counter()
    res = rt.rotate(conn, keep_rows=keep_rows,
                    actor="system", actor_id="system:benchmark")
    rotate_time = time.perf_counter() - t0
    assert res["ok"], f"rotate failed at {n_rows}: {res}"

    # Verify after rotation (live only, anchored)
    t0 = time.perf_counter()
    v2 = ac.verify_live(conn)
    verify_post_time = time.perf_counter() - t0
    assert v2["ok"], f"post-rotate verify failed: {v2}"

    # Verify full (archive replay + live)
    t0 = time.perf_counter()
    vf = rt.verify_full(conn)
    verify_full_time = time.perf_counter() - t0
    assert vf["ok"], f"verify_full failed: {vf}"

    conn.close()
    return {
        "n_rows": n_rows,
        "keep_rows": keep_rows,
        "archived": res.get("archived_rows"),
        "seed_s": round(seed_time, 3),
        "verify_live_s": round(verify_time, 3),
        "rotate_s": round(rotate_time, 3),
        "verify_post_rotate_s": round(verify_post_time, 4),
        "verify_full_s": round(verify_full_time, 3),
        "archive_checked": vf.get("archive_checked"),
        "live_checked_after": v2["checked"],
    }


def main():
    results = []
    print("Seeding and benchmarking 10,000 rows...")
    results.append(bench(10_000, keep_rows=1000))
    print(f"  done: verify={results[-1]['verify_live_s']}s, "
          f"rotate={results[-1]['rotate_s']}s, "
          f"verify_full={results[-1]['verify_full_s']}s")

    print("Seeding and benchmarking 100,000 rows...")
    results.append(bench(100_000, keep_rows=1000))
    print(f"  done: verify={results[-1]['verify_live_s']}s, "
          f"rotate={results[-1]['rotate_s']}s, "
          f"verify_full={results[-1]['verify_full_s']}s")

    print("\n=== RESULTS ===")
    for r in results:
        print(r)

    # Write results for the evidence file
    out = Path(__file__).resolve().parent / "bench_results.json"
    import json
    out.write_text(json.dumps(results, indent=2))
    print(f"\nResults written to {out}")


if __name__ == "__main__":
    main()
