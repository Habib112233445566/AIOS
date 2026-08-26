# SPEC — Audit Ring Retention (Sprint 3)

**Status:** IMPLEMENTED (2026-08-21)
**Research:** `docs/research/AIOS-AUDIT-RING-RETENTION-2026-08-21.md`
**ADR:** `mostimportanAIfolder/ADR-0036-audit-ring-retention.md`
**Constitution:** P-2 (immutability), O-2 (one row per action), O-4
(hash chain, truncation detection), C-4 (auditability gate)

Retention for the `audit_ring` hash chain: checkpointed segment
rotation (RFC 9162 §4.13 pattern) with JSONL archives and per-segment
Bloom filters for historical membership queries.

---

## 1. Data model

### 1.1 New table `audit_segments` (one row per rotated segment)

```sql
CREATE TABLE IF NOT EXISTS audit_segments (
  segment_id        INTEGER PRIMARY KEY,
  closed_at         TEXT NOT NULL,          -- ISO-8601 UTC
  first_row_id      INTEGER NOT NULL,
  last_row_id       INTEGER NOT NULL,
  row_count         INTEGER NOT NULL,
  genesis_prev_hash TEXT NOT NULL,          -- prev_hash of first archived row
  head_hash         TEXT NOT NULL,          -- hash of last archived row
  archive_path      TEXT NOT NULL,          -- absolute path to segment JSONL
  archive_sha256    TEXT NOT NULL,          -- sha256 of archive file bytes
  bloom_m_bits      INTEGER NOT NULL,
  bloom_k           INTEGER NOT NULL,
  bloom_hex         TEXT NOT NULL           -- little-endian bit array as hex
);
```

Identical DDL in both substrates
(`code/aiosh-cli/src/retention.ts:ensureSegmentsSchema`,
`code/aiosh-mcp/aiosh_mcp/retention.py:ensure_segments_schema`).
Schema creation is idempotent and runs on every open, same migration
pattern as the Sprint-2 classifier columns.

### 1.2 Archive files

`$AIOSH_HOME/audit-archive/segment-<segment_id>.jsonl`

One line per archived row: canonical JSON (same serializer as the
chain hash) of the row's `to_dict()` shape:

```
{args, c_flags, command, actor, actor_id, constitution_rev,
 grant_token, hash, id, outcome, outcome_detail, prev_hash, target,
 tool, ts, [policy_revision, classify_rule_ids, classify_evidence,
  classify_overall_verdict, classify_verdict_reason if present]}
```

- Base fields are always present (null when unset), classifier fields
  only when present — byte-identical to the proto that was hashed, so
  every archived line re-verifies offline:
  `hash == sha256(prev_hash || canonical(line minus id/hash))`.
- File is written atomically (tmp file + fsync + rename) before the
  DB transaction commits; a crash between file write and DB commit
  loses nothing (the file is simply overwritten on retry).

## 2. `rotate(keep_rows=0)`

Steps (single substrate-side implementation, same order in Rust / TS /
Python — Rust is the shipping substrate, `code/aiosh-rust/aiosh-core/src/
retention.rs`):

1. `verify()` the live chain from the current anchor. **Refuse to
   rotate a broken chain** (rotation must never hide tampering).
2. Read all live rows ascending by id. If `count <= keep_rows` → no-op
   result `{rotated: false, reason: "nothing to rotate"}`.
3. `to_archive = rows[:count-keep_rows]`.
4. `segment_id = COALESCE(MAX(segment_id), 0) + 1`.
5. Serialize archive lines, compute `archive_sha256`, build the Bloom
   filter over `row.hash` values.
6. Write the archive file atomically.
7. One DB transaction:
   a. `INSERT INTO audit_segments ...`
   b. `DELETE FROM audit_ring WHERE id <= last_row_id`
   c. Append the `audit.rotate` row with `prev_hash` =
      `head_hash` (explicit override, NOT the `head_hash()` helper,
      since the table is empty at this point). This row is the first
      live row of the new segment and records
      `{segment_id, first_row_id, last_row_id, row_count, head_hash,
      archive_sha256, archive_path, keep_rows}`.
   d. `COMMIT`.

Invariants after rotation:

- Live table row count == `keep_rows + 1` (the +1 is the rotation row).
- First live row `prev_hash` == latest checkpoint `head_hash`.
- No row content was mutated; archived rows are byte-preserved.
- Ids keep increasing (SQLite `sqlite_sequence` high-water survives).

## 3. `verify(full=False)`

- **Anchor** = latest `audit_segments.head_hash` if any segment
  exists, else `GENESIS_HASH` (`0`×64).
- Walk the live table ascending by id: first row's `prev_hash` must
  equal the anchor; each hash recomputed via the canonical proto rule
  (Sprint-0 invariant, incl. conditional Sprint-2 classifier fields).
- Returns `{ok, checked, broken_at, segments, anchor, mode: "live"}`.
  A live chain whose first row matches neither the anchor nor GENESIS
  reports `broken_at` on that row — this is how O-4's "truncation
  triggers alert" works post-rotation.
- **`full=True`** additionally replays every archive in `segment_id`
  order before the live walk:
  1. file exists and `sha256(file bytes) == archive_sha256`;
  2. first line's `prev_hash == expected anchor` (GENESIS for
     segment 1, previous segment's `head_hash` after);
  3. every line re-hashes correctly and links to the next;
  4. final hash == recorded `head_hash`, line count == `row_count`.
  Returns `{ok, checked, archive_checked, segments, broken_at,
  broken_segment, mode: "full"}`.

## 4. Bloom filter (membership over cold segments)

Deterministic, cross-language identical:

- Params: `m = max(1024, ceil_to_8(n * 16))` bits, `k = 8`.
  At 16 bits/element and k=8, FPR ≈ (1−e^(−8/16))^8 ≈ 5.4e-4.
- Index derivation for item string `s` (row hash hex), `i` in `0..k-1`:
  `idx_i = int64_be(sha256(utf8(i + ":" + s))[0:8]) mod m`.
- Bit array serialized little-endian within each byte, as lowercase
  hex in `bloom_hex`.

`seen(hash_hex)` semantics (never a false negative):

- live hit: exact match in `audit_ring.hash` → `{found: "live"}`.
- Bloom positives per segment → `{found: "maybe", segments: [...]}`;
  a maybe answer is refined by scanning that segment's archive for the
  exact hash when `exact=True` is requested → `{found: "archive"}`.
- no hits → `{found: "no"}` (definitive).

## 5. Surfaces

### CLI (`aiosh audit ...`)

| Command | Notes |
|---|---|
| `audit rotate [--keep <n>] [--dry-run]` | User-driven; the `audit.rotate` row IS the emitted row (one-row rule, O-2) |
| `audit segments` | List checkpoints (read-only, emits `audit.segments` row) |
| `audit seen <hash> [--exact]` | Bloom + optional archive scan (emits `audit.seen` row) |
| `audit verify [--full]` | `--full` replays archives too |

### MCP tools (`server.py`)

| Tool | Gate |
|---|---|
| `aios.audit.rotate` | **require_grant=True** — mutates the audit store (C-3 irreversible), same explicit-grant pattern as `aios.fs.read`; classifier verdict must also pass (§D-4 order) |
| `aios.audit.seen` | read-only, no grant |
| `aios.audit.verify` | gains `full: bool` param; read-only |
| `aios.audit.segments` | read-only, no grant |

A grant for rotation is scoped e.g. `--tools audit.rotate`.

## 6. Cross-substrate parity requirements

- `audit_segments` DDL byte-compatible (both create-if-not-exists).
- Archive line bytes: `canonicalJson` (TS) == `json.dumps(sort_keys,
  separators)` (Python) — inherited Sprint-0 invariant; Rust `canonical()`
  (`code/aiosh-rust/aiosh-core/src/canonical.rs`) reproduces it
  byte-for-byte.
- Bloom index derivation identical (both use SHA-256 + 8-byte BE int).
- Rotation-row proto shape identical (no classifier fields → omitted,
  matching the Sprint-0 row shape).
- Proven by the Rust retention tests (`code/aiosh-rust`) and legacy
  `tests/test_retention_smoke.py`: Python rotates → TS verifies (live +
  full), TS rotates → Python verifies, Bloom parity
  for the same hashes from both substrates.

## 7. Failure modes

| Failure | Behavior |
|---|---|
| Live chain broken before rotate | rotate refuses, nothing touched |
| Archive file deleted/corrupted | `verify --full` fails with `broken_segment` + sha256 mismatch; live `verify` still passes (hot path unaffected) |
| Crash mid-rotate (after file write, before commit) | orphan archive file overwritten on retry; DB unchanged, chain intact |
| Concurrent rotates | SQLite WAL single-writer serializes; `segment_id` computed inside the write transaction |
| Rotate used as covert erasure | rotation row records segment id, counts, hashes, archive path+sha256; PEP grant required on the MCP surface |
