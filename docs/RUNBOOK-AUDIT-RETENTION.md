# RUNBOOK — Audit-Ring Retention Operations

**Sprint 3 · ADR-0036 · SPEC-AUDIT-RETENTION.md**
**Last updated:** 2026-08-21

This runbook covers day-2 operations for the checkpointed segment-rotation
retention system. All commands assume the working directory is the repo root
(`/content/AIOS_MERGED`) unless otherwise noted.

---

## 1. Rotate — archive the oldest live rows into a sealed segment

### What it does

1. Verifies the live chain (refuses if broken).
2. Archives the oldest `count − keep_rows` rows to a JSONL file at
   `$AIOSH_HOME/audit-archive/segment-NNNNNN.jsonl`.
3. Records a checkpoint row in `audit_segments` (head hash, row count,
   archive sha256, bloom filter).
4. Deletes archived rows from the live `audit_ring` table.
5. Appends exactly one `audit.rotate` row to the live ring (O-2).

### CLI

```bash
# Preview (no writes, no audit row)
node code/aiosh-cli/dist/cli.js audit rotate --keep 100 --dry-run

# Execute: keep the newest 100 rows live, archive everything older
node code/aiosh-cli/dist/cli.js audit rotate --keep 100

# Archive everything (keep 0 live rows; chain continues from checkpoint)
node code/aiosh-cli/dist/cli.js audit rotate --keep 0
```

Environment: set `AIOSH_HOME` to control where `audit.db` and
`audit-archive/` live. Default: `~/.aios/`.

### MCP tool

```json
{
  "tool": "aios.audit.rotate",
  "arguments": { "keep_rows": 100, "grant_id": "gr_..." }
}
```

**Requires a PEP grant** scoped to `audit.rotate` or `audit.*`.
Without a valid grant, the call is refused at the PEP gate and the
refusal is itself audited.

Issue a grant via CLI:
```bash
node code/aiosh-cli/dist/cli.js grant create \
  --to agent:ops-bot \
  --tools audit.rotate \
  --ttl 3600
```

### Alarms / refusal reasons

| Symptom | Meaning | Action |
|---|---|---|
| `refusing to rotate: live chain broken at row N` | The hash chain is already corrupted. Rotation cannot proceed because it would seal a broken chain. | Investigate row N. Do NOT force-rotate. See §5 Disaster Recovery. |
| `rotated: false, reason: "nothing to rotate"` | Live row count ≤ keep_rows. No-op. | Expected; no action needed. An audit row is still written. |
| MCP: `gate: "pep"`, reason mentions grant | No valid grant for `audit.rotate`. | Issue a grant scoped to `audit.rotate` or `audit.*`. |
| MCP: `gate: "classifier"` | Rule-pack refused the call (e.g. prompt-injection text in args). | Inspect the refusal row's `classify_rule_ids`. |

### What rotation writes

The `audit.rotate` row in the live ring carries:
```json
{
  "tool": "audit.rotate",
  "args": {
    "rotated": true,
    "segment_id": 1,
    "first_row_id": 1,
    "last_row_id": 42,
    "row_count": 42,
    "keep_rows": 100,
    "head_hash": "<sha256 of last archived row>",
    "archive_path": "/home/user/.aios/audit-archive/segment-000001.jsonl",
    "archive_sha256": "<sha256 of the JSONL file bytes>",
    "bloom_m_bits": 1024,
    "bloom_k": 8
  },
  "c_flags": { "c1": false, "c2": false, "c3": true, "c4": true }
}
```
`c3=true` marks this as an irreversible-state-change action.

---

## 2. Verify — check chain integrity

### Live-only (default)

```bash
node code/aiosh-cli/dist/cli.js audit verify
```

Walks the live `audit_ring` table anchored at the newest segment checkpoint
(or genesis if no rotation has occurred). Returns:
```json
{ "ok": true, "checked": 47, "anchor": "<head_hash or genesis>", "segments": 2, "mode": "live" }
```

### Full (replays all archives)

```bash
node code/aiosh-cli/dist/cli.js audit verify --full
```

Replays every archived segment file in order, then the live table. Returns:
```json
{
  "ok": true, "checked": 512, "mode": "full",
  "segments": 2, "archive_checked": 465, "live_checked": 47,
  "anchor": "<genesis>"
}
```

### MCP tool

```json
{ "tool": "aios.audit.verify", "arguments": { "full": true } }
```

### Alarms

| Symptom | Meaning | Action |
|---|---|---|
| `ok: false`, `broken_at: <id>`, mode live | A live row's prev_hash or recomputed hash doesn't match. | Row `<id>` or its predecessor was tampered with or corrupted. Inspect manually. |
| `ok: false`, `error: "archive missing: <path>"`, `broken_segment: N` | The JSONL file for segment N is not at the recorded path. | Restore from backup or relocate. See §5. |
| `ok: false`, `error: "archive sha256 mismatch"`, `broken_segment: N` | The archive file exists but its content hash doesn't match the checkpoint. | File was modified after rotation. Restore from backup. |
| `ok: false`, `error: "segment N genesis_prev_hash does not link"` | Segments are out of order or a segment was deleted. | Check `audit_segments` table ordering. |
| `ok: false`, `error: "archive hash recompute mismatch"`, `broken_at: <id>` | A specific archived row's hash doesn't recompute. | That row was tampered with inside the archive file. |
| `ok: false`, `error: "segment N head_hash mismatch"` | The last row in the archive doesn't match the recorded head. | Archive file truncated or modified. |

---

## 3. Seen — membership query over archived + live rows

### CLI

```bash
# Bloom-only check (fast, may have false positives)
node code/aiosh-cli/dist/cli.js audit seen <64-char-sha256-hex>

# Exact check (scans archive files for confirmation)
node code/aiosh-cli/dist/cli.js audit seen <64-char-sha256-hex> --exact
```

### MCP tool

```json
{ "tool": "aios.audit.seen", "arguments": { "hash_hex": "abc123...", "exact": true } }
```

### Response semantics

| `found` value | Meaning |
|---|---|
| `"live"` | Hash exists in the current live `audit_ring` table. `id` field gives the row. |
| `"archive"` | Hash confirmed present in an archived segment file (exact mode only). `segments` lists which. |
| `"maybe"` | Bloom filter says possibly present in segment(s) listed. Not confirmed. Use `--exact` / `exact: true` to confirm. |
| `"no"` | Definitive negative. Hash was never logged. (Bloom filters have no false negatives.) |

### Bloom filter parameters

- Bits per item: 16 (minimum 1024 bits per segment)
- Hash functions (k): 8
- Index derivation: `sha256(f"{i}:{row_hash}")[:8]` as big-endian uint64, mod m
- Storage: lowercase hex string in `audit_segments.bloom_hex`
- False-positive rate at capacity: ~0.05%

---

## 4. Segments — list archived checkpoints

### CLI

```bash
node code/aiosh-cli/dist/cli.js audit segments
```

### MCP tool

```json
{ "tool": "aios.audit.segments", "arguments": {} }
```

Returns all segment records ordered by `segment_id ASC`.

---

## 5. Disaster Recovery

### Scenario A: Archive file deleted or corrupted

1. Run `audit verify --full` to identify which segment(s) are affected.
2. If you have a backup of the JSONL file, restore it to the recorded
   `archive_path`. Re-run `verify --full`.
3. If no backup exists: the segment's rows are lost. The chain from
   genesis to that segment is broken. Options:
   - Accept the gap. The live chain remains verifiable from the segment
     *after* the lost one (its `genesis_prev_hash` records what was lost).
   - Manually insert a placeholder segment record noting the loss.
   - The bloom filter in `audit_segments` still exists, so `seen` queries
     can still report "maybe" for hashes in the lost segment.

### Scenario B: Live chain corrupted (verify fails, mode=live)

1. Identify `broken_at` row ID.
2. Inspect that row and its predecessor in the DB.
3. If a row was tampered: the chain is compromised. Do NOT rotate.
   Document the incident. Consider restoring the DB from backup.
4. If the DB itself is corrupted: restore from backup, re-run verify.

### Scenario C: DB file lost entirely

1. The archive JSONL files (if backed up separately) contain the full
   row data. The chain can be reconstructed from them.
2. The `audit_segments` table (if backed up) contains the checkpoint
   hashes and bloom filters.
3. Without both, the audit history is unrecoverable.

### Scenario D: Rotation interrupted mid-transaction

- The implementation uses SQLite transactions. If the process crashes
  between archive-file-write and DB commit, the archive file exists but
  the DB is unchanged. On next startup, the orphan archive file is
  harmless; the next rotation will overwrite it (same segment_id).
- If the crash occurs after DB commit, the state is consistent.

### Backup recommendations

- Back up `$AIOSH_HOME/audit.db` AND `$AIOSH_HOME/audit-archive/` together.
- The archive files are the durable record; the DB is the hot index.
- Periodically run `audit verify --full` as a backup-integrity check.

---

## 6. Operational cadence (recommended)

| Trigger | Action |
|---|---|
| Live ring exceeds ~5,000 rows | `audit rotate --keep 1000` |
| Weekly integrity check | `audit verify --full` |
| After any suspected tampering | `audit verify` (live) then `--full` |
| Before major system update | `audit rotate --keep 0` (seal everything, start fresh segment) |
| Investigating a past event | `audit seen <hash> --exact` |

Thresholds are recommendations. Adjust based on write volume and
storage constraints. The system imposes no automatic rotation — it is
always operator-initiated (CLI or MCP with grant).

---

## 7. Cross-substrate parity notes

Both `code/aiosh-cli/src/retention.ts` and
`code/aiosh-mcp/aiosh_mcp/retention.py` implement identical logic:

- Same `audit_segments` DDL
- Same archive line format (canonical JSON of `row.to_dict()`)
- Same bloom index derivation (sha256, big-endian uint64, mod m)
- Same rotation-row proto shape (tool="audit.rotate", no classifier fields)
- Same verify anchoring logic

Proven by `tests/test_retention_smoke.py` R6 (TS rotates → Python
verifies) and R7 (MCP gate + Python rotates → TS verifies).

If either substrate is modified, the cross-substrate smoke (R6) must
be re-run before merging.
