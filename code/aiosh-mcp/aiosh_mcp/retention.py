"""Sprint 3 — audit-ring retention: checkpointed segment rotation.

The audit ring is a hash chain (P-2: append-only, no rewrite/delete).
Unbounded growth is handled the Certificate-Transparency way (RFC 9162
§4.13 "Shutting Down a Log"): a segment is *frozen* at its head hash,
its rows are archived byte-identically to a JSONL file, and a checkpoint
row records {ids, head_hash, archive_sha256, bloom filter}. The live
table then only holds the retained tail, whose first row's prev_hash
equals the checkpoint head — so verify() stays anchored and the full
history remains re-verifiable from cold storage.

Research: docs/research/AIOS-AUDIT-RING-RETENTION-2026-08-21.md
Spec:     docs/SPEC-AUDIT-RETENTION.md

Cross-substrate contract (must match code/aiosh-cli/src/retention.ts):
  - `audit_segments` DDL
  - archive line = canonical(row.to_dict()) i.e. canonical JSON of the
    exact hashed proto + {id, hash}
  - bloom indices: sha256(f"{i}:{item}")[:8] as big-endian uint64 % m,
    little-endian bit order inside bytes, stored as lowercase hex
  - rotation row: tool="audit.rotate", no classifier fields
"""

from __future__ import annotations

import hashlib
import json
import os
import sqlite3
import tempfile
from pathlib import Path
from typing import Any

from . import audit_client as ac

_SEGMENTS_TABLE = "audit_segments"

_SEGMENTS_DDL = f"""
CREATE TABLE IF NOT EXISTS {_SEGMENTS_TABLE} (
  segment_id        INTEGER PRIMARY KEY,
  closed_at         TEXT NOT NULL,
  first_row_id      INTEGER NOT NULL,
  last_row_id       INTEGER NOT NULL,
  row_count         INTEGER NOT NULL,
  genesis_prev_hash TEXT NOT NULL,
  head_hash         TEXT NOT NULL,
  archive_path      TEXT NOT NULL,
  archive_sha256    TEXT NOT NULL,
  bloom_m_bits      INTEGER NOT NULL,
  bloom_k           INTEGER NOT NULL,
  bloom_hex         TEXT NOT NULL
);
"""


def ensure_segments_schema(conn: sqlite3.Connection) -> None:
    conn.execute(_SEGMENTS_DDL)
    conn.commit()


def default_archive_root(conn: sqlite3.Connection) -> Path:
    db_file = None
    try:
        for r in conn.execute("PRAGMA database_list").fetchall():
            if r["name"] == "main" and r["file"]:
                db_file = r["file"]
                break
    except (sqlite3.Error, IndexError, KeyError):
        db_file = None
    if db_file and db_file != ":memory:":
        return Path(db_file).parent / "audit-archive"
    base = os.environ.get(
        "AIOSH_HOME", f"{os.environ.get('HOME', '/tmp')}/.aios")
    return Path(base) / "audit-archive"


# ----------------------------------------------------------------------
# Bloom filter — deterministic, cross-language identical.
# ----------------------------------------------------------------------

BLOOM_BITS_PER_ITEM = 16
BLOOM_MIN_BITS = 1024
BLOOM_K = 8


def bloom_params(n: int) -> tuple[int, int]:
    m = max(BLOOM_MIN_BITS, n * BLOOM_BITS_PER_ITEM)
    m = ((m + 7) // 8) * 8
    return m, BLOOM_K


def _bloom_indices(item: str, m: int, k: int) -> list[int]:
    out = []
    for i in range(k):
        digest = hashlib.sha256(f"{i}:{item}".encode("utf-8")).digest()
        out.append(int.from_bytes(digest[:8], "big") % m)
    return out


def bloom_add(bits: bytearray, m: int, k: int, item: str) -> None:
    for idx in _bloom_indices(item, m, k):
        bits[idx >> 3] |= 1 << (idx & 7)


def bloom_test(bits: bytes, m: int, k: int, item: str) -> bool:
    for idx in _bloom_indices(item, m, k):
        if not (bits[idx >> 3] & (1 << (idx & 7))):
            return False
    return True


# ----------------------------------------------------------------------
# Segments
# ----------------------------------------------------------------------

def list_segments(conn: sqlite3.Connection) -> list[dict[str, Any]]:
    if not _table_exists(conn, _SEGMENTS_TABLE):
        return []
    cur = conn.execute(f"SELECT * FROM {_SEGMENTS_TABLE} ORDER BY segment_id ASC")
    return [dict(r) for r in cur.fetchall()]


def _table_exists(conn: sqlite3.Connection, name: str) -> bool:
    cur = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?", (name,))
    return cur.fetchone() is not None


def _rotate_row_proto(
    *, rotated: bool, actor: str, actor_id: str,
    constitution_rev: str | None, grant_token: str | None,
    args: dict[str, Any], outcome: str, outcome_detail: str | None,
) -> dict[str, Any]:
    return {
        "ts": ac.utcnow_iso(),
        "actor": actor,
        "actor_id": actor_id,
        "tool": "audit.rotate",
        "command": "audit.rotate",
        "args": args,
        "target": None,
        "outcome": outcome,
        "outcome_detail": outcome_detail,
        "constitution_rev": constitution_rev,
        "grant_token": grant_token,
        "c_flags": {"c1": False, "c2": False,
                    "c3": bool(rotated), "c4": True},
    }


def rotate(
    conn: sqlite3.Connection,
    *,
    keep_rows: int = 0,
    dry_run: bool = False,
    archive_root: str | None = None,
    actor: str = "system",
    actor_id: str = "system:retention",
    grant_token: str | None = None,
    constitution_rev: str | None = None,
) -> dict[str, Any]:
    """Archive the oldest (count - keep_rows) live rows into a frozen
    segment. Returns a result dict; on success a single `audit.rotate`
    row is appended to the live ring (O-2: one row per action)."""
    ensure_segments_schema(conn)
    ac.ensure_audit_schema(conn)

    live = ac.verify_live(conn)
    if not live["ok"]:
        err = f"refusing to rotate: live chain broken at row {live.get('broken_at')}"
        if dry_run:
            return {"ok": False, "rotated": False, "dry_run": True, "error": err}
        row = ac.write_audit_row(conn, _rotate_row_proto(
            rotated=False, actor=actor, actor_id=actor_id,
            constitution_rev=constitution_rev, grant_token=grant_token,
            args={"rotated": False, "reason": "chain broken"},
            outcome="refused", outcome_detail=err))
        return {"ok": False, "rotated": False, "error": err, "audit_id": row.id}

    count = conn.execute("SELECT COUNT(*) AS n FROM audit_ring").fetchone()["n"]
    keep = max(0, int(keep_rows))
    if count <= keep:
        if dry_run:
            return {"ok": True, "rotated": False, "dry_run": True,
                    "live_rows": count, "would_archive": 0, "keep_rows": keep}
        row = ac.write_audit_row(conn, _rotate_row_proto(
            rotated=False, actor=actor, actor_id=actor_id,
            constitution_rev=constitution_rev, grant_token=grant_token,
            args={"rotated": False, "reason": "nothing to rotate",
                  "live_rows": count, "keep_rows": keep},
            outcome="ok", outcome_detail=None))
        return {"ok": True, "rotated": False, "live_rows": count,
                "audit_id": row.id}

    archive_count = count - keep
    if dry_run:
        nxt = conn.execute(
            f"SELECT COALESCE(MAX(segment_id),0)+1 AS next FROM {_SEGMENTS_TABLE}"
        ).fetchone()["next"]
        return {"ok": True, "rotated": False, "dry_run": True,
                "live_rows": count, "would_archive": archive_count,
                "keep_rows": keep, "next_segment_id": nxt}

    raw_rows = conn.execute(
        "SELECT * FROM audit_ring ORDER BY id ASC LIMIT ?", (archive_count,)
    ).fetchall()
    first_id = raw_rows[0]["id"]
    last_id = raw_rows[-1]["id"]
    genesis_prev = raw_rows[0]["prev_hash"]
    head = raw_rows[-1]["hash"]

    segment_id = conn.execute(
        f"SELECT COALESCE(MAX(segment_id),0)+1 AS next FROM {_SEGMENTS_TABLE}"
    ).fetchone()["next"]

    root = Path(archive_root) if archive_root else default_archive_root(conn)
    root.mkdir(parents=True, exist_ok=True)
    archive_path = root / f"segment-{segment_id:06d}.jsonl"

    lines: list[str] = []
    hashes: list[str] = []
    for raw in raw_rows:
        row = ac.AuditRow.from_sql(raw)
        lines.append(ac.canonical(row.to_dict()))
        hashes.append(row.hash)
    content = "\n".join(lines) + "\n"
    content_bytes = content.encode("utf-8")
    archive_sha = hashlib.sha256(content_bytes).hexdigest()

    m, k = bloom_params(len(hashes))
    bits = bytearray((m + 7) // 8)
    for h in hashes:
        bloom_add(bits, m, k, h)
    bloom_hex = bits.hex()

    # Atomic file write before the DB transaction so the archive is
    # durable before rows leave the live table. Unique tmp name (crash
    # leftovers never block a retry), 0600 perms, and refuse to
    # overwrite an existing segment file (covert-overwrite guard).
    if archive_path.exists():
        raise FileExistsError(f"refusing to overwrite existing archive: {archive_path}")
    fd, tmp_name = tempfile.mkstemp(prefix=archive_path.name + ".",
                                    suffix=".tmp", dir=str(archive_path.parent))
    tmp = Path(tmp_name)
    try:
        with os.fdopen(fd, "wb") as fh:
            fh.write(content_bytes)
        os.chmod(tmp, 0o600)
        os.replace(tmp, archive_path)
    except BaseException:
        try:
            tmp.unlink(missing_ok=True)
        except OSError:
            pass
        raise

    proto = _rotate_row_proto(
        rotated=True, actor=actor, actor_id=actor_id,
        constitution_rev=constitution_rev, grant_token=grant_token,
        args={
            "rotated": True,
            "segment_id": segment_id,
            "first_row_id": first_id,
            "last_row_id": last_id,
            "row_count": len(raw_rows),
            "keep_rows": keep,
            "head_hash": head,
            "archive_path": str(archive_path),
            "archive_sha256": archive_sha,
            "bloom_m_bits": m,
            "bloom_k": k,
        },
        outcome="ok", outcome_detail=None)

    try:
        conn.execute(
            f"""INSERT INTO {_SEGMENTS_TABLE}
                (segment_id, closed_at, first_row_id, last_row_id, row_count,
                 genesis_prev_hash, head_hash, archive_path, archive_sha256,
                 bloom_m_bits, bloom_k, bloom_hex)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (segment_id, ac.utcnow_iso(), first_id, last_id, len(raw_rows),
             genesis_prev, head, str(archive_path), archive_sha, m, k, bloom_hex))
        conn.execute("DELETE FROM audit_ring WHERE id <= ?", (last_id,))
        rotation_row = ac._insert_audit_row(conn, proto)
        conn.commit()
    except Exception:
        conn.rollback()
        try:
            archive_path.unlink(missing_ok=True)
        except OSError:
            pass
        raise

    return {
        "ok": True, "rotated": True,
        "segment_id": segment_id,
        "archived_rows": len(raw_rows),
        "keep_rows": keep,
        "archive_path": str(archive_path),
        "archive_sha256": archive_sha,
        "head_hash": head,
        "audit_id": rotation_row.id,
    }


# ----------------------------------------------------------------------
# verify_full — replay every archive segment, then the live table.
# ----------------------------------------------------------------------

def verify_full(conn: sqlite3.Connection,
                archive_root: str | None = None) -> dict[str, Any]:
    ensure_segments_schema(conn)
    segments = conn.execute(
        f"SELECT * FROM {_SEGMENTS_TABLE} ORDER BY segment_id ASC").fetchall()
    anchor = ac.GENESIS
    archive_checked = 0
    for seg in segments:
        path = Path(seg["archive_path"])
        if not path.exists() and archive_root:
            path = Path(archive_root) / path.name
        if not path.exists():
            return {"ok": False, "mode": "full",
                    "error": f"archive missing: {path}",
                    "broken_segment": seg["segment_id"],
                    "checked": archive_checked, "archive_checked": archive_checked,
                    "segments": len(segments)}
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != seg["archive_sha256"]:
            return {"ok": False, "mode": "full",
                    "error": f"archive sha256 mismatch: {path}",
                    "broken_segment": seg["segment_id"],
                    "checked": archive_checked, "archive_checked": archive_checked,
                    "segments": len(segments)}
        if seg["genesis_prev_hash"] != anchor:
            return {"ok": False, "mode": "full",
                    "error": f"segment {seg['segment_id']} genesis_prev_hash "
                             f"does not link to previous anchor",
                    "broken_segment": seg["segment_id"],
                    "checked": archive_checked, "archive_checked": archive_checked,
                    "segments": len(segments)}
        lines = [ln for ln in data.decode("utf-8").split("\n") if ln]
        if len(lines) != seg["row_count"]:
            return {"ok": False, "mode": "full",
                    "error": f"segment {seg['segment_id']} line count "
                             f"{len(lines)} != recorded {seg['row_count']}",
                    "broken_segment": seg["segment_id"],
                    "checked": archive_checked, "archive_checked": archive_checked,
                    "segments": len(segments)}
        prev = anchor
        for i, line in enumerate(lines):
            obj = json.loads(line)
            if i == 0 and obj.get("id") != seg["first_row_id"]:
                return {"ok": False, "mode": "full",
                        "error": f"segment {seg['segment_id']} first id mismatch",
                        "broken_segment": seg["segment_id"],
                        "checked": archive_checked, "archive_checked": archive_checked,
                        "segments": len(segments)}
            if obj.get("prev_hash") != prev:
                return {"ok": False, "mode": "full",
                        "error": "archive prev_hash link broken",
                        "broken_at": obj.get("id"),
                        "broken_segment": seg["segment_id"],
                        "checked": archive_checked, "archive_checked": archive_checked,
                        "segments": len(segments)}
            proto = {k: v for k, v in obj.items() if k not in {"id", "hash"}}
            expected = ac.sha256_hex(prev + ac.canonical(proto))
            if expected != obj.get("hash"):
                return {"ok": False, "mode": "full",
                        "error": "archive hash recompute mismatch",
                        "broken_at": obj.get("id"),
                        "broken_segment": seg["segment_id"],
                        "checked": archive_checked, "archive_checked": archive_checked,
                        "segments": len(segments)}
            prev = obj["hash"]
            archive_checked += 1
        if prev != seg["head_hash"]:
            return {"ok": False, "mode": "full",
                    "error": f"segment {seg['segment_id']} head_hash mismatch",
                    "broken_segment": seg["segment_id"],
                    "checked": archive_checked, "archive_checked": archive_checked,
                    "segments": len(segments)}
        anchor = seg["head_hash"]

    live = ac.verify_live(conn, anchor=anchor)
    result: dict[str, Any] = {
        "mode": "full",
        "segments": len(segments),
        "archive_checked": archive_checked,
        "live_checked": live["checked"],
        "checked": archive_checked + live["checked"],
        "anchor": anchor,
    }
    result.update({k: v for k, v in live.items() if k not in {"anchor", "checked"}})
    return result


# ----------------------------------------------------------------------
# seen — membership query over live table + archived segments.
# ----------------------------------------------------------------------

def seen(conn: sqlite3.Connection, hash_hex: str, *,
         exact: bool = False,
         archive_root: str | None = None) -> dict[str, Any]:
    h = (hash_hex or "").strip().lower()
    live = conn.execute(
        "SELECT id FROM audit_ring WHERE hash = ?", (h,)).fetchone()
    if live:
        return {"found": "live", "id": live["id"], "segments": []}
    maybe: list[int] = []
    confirmed: list[int] = []
    for seg in list_segments(conn):
        bits = bytes.fromhex(seg["bloom_hex"])
        if not bloom_test(bits, seg["bloom_m_bits"], seg["bloom_k"], h):
            continue
        maybe.append(seg["segment_id"])
        if exact:
            path = Path(seg["archive_path"])
            if not path.exists() and archive_root:
                path = Path(archive_root) / path.name
            if path.exists():
                for line in path.read_text(encoding="utf-8").split("\n"):
                    if not line:
                        continue
                    try:
                        obj = json.loads(line)
                    except (ValueError, TypeError):
                        continue
                    if isinstance(obj.get("hash"), str) and obj["hash"].lower() == h:
                        confirmed.append(seg["segment_id"])
                        break
    if exact and confirmed:
        return {"found": "archive", "segments": confirmed}
    if exact and maybe:
        # bloom hit but archive scan found nothing (or file missing)
        return {"found": "maybe", "segments": maybe,
                "note": "bloom positive, exact scan inconclusive"}
    if maybe:
        return {"found": "maybe", "segments": maybe}
    return {"found": "no", "segments": []}
