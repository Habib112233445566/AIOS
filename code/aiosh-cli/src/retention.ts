/**
 * AIOS shell — Sprint 3 audit-ring retention: checkpointed segment rotation.
 *
 * Mirror of `code/aiosh-mcp/aiosh_mcp/retention.py`. Both substrates must
 * stay behaviourally identical on the same SQLite DB (see
 * docs/SPEC-AUDIT-RETENTION.md §6 and tests/test_retention_smoke.py).
 *
 * Model (RFC 9162 §4.13-style log rotation): a rotation freezes the
 * oldest rows into an immutable JSONL archive, records a checkpoint in
 * `audit_segments` (ids, head hash, archive sha256, bloom filter), and
 * removes the archived rows from the live table only. Entries are never
 * destroyed (Constitution P-2); the live chain simply re-anchors at the
 * newest checkpoint head, which `AuditRing.verify`/`tail` honour.
 */

import type Database from "better-sqlite3";
import {
  existsSync, mkdirSync, readFileSync, renameSync, unlinkSync, writeFileSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";
import {
  AuditRing, auditRowFromSql, canonicalJson, hashProtoFromRow, sha256Hex,
} from "./audit.js";
import { GENESIS_HASH } from "./types.js";
import type { AuditRow } from "./types.js";

export const BLOOM_BITS_PER_ITEM = 16;
export const BLOOM_MIN_BITS = 1024;
export const BLOOM_K = 8;

export interface SegmentRecord {
  segment_id: number;
  closed_at: string;
  first_row_id: number;
  last_row_id: number;
  row_count: number;
  genesis_prev_hash: string;
  head_hash: string;
  archive_path: string;
  archive_sha256: string;
  bloom_m_bits: number;
  bloom_k: number;
  bloom_hex: string;
}

export function ensureSegmentsSchema(db: Database.Database): void {
  db.exec(`
    CREATE TABLE IF NOT EXISTS audit_segments (
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
  `);
}

export function listSegments(db: Database.Database): SegmentRecord[] {
  ensureSegmentsSchema(db);
  return db.prepare(
    "SELECT * FROM audit_segments ORDER BY segment_id ASC"
  ).all() as SegmentRecord[];
}

// ---------------------------------------------------------------------
// Bloom filter — deterministic and byte-identical with retention.py.
// index_i = BE-uint64(sha256(`${i}:${item}`)) mod m; little-endian bits.
// ---------------------------------------------------------------------

export function bloomParams(n: number): { m: number; k: number } {
  let m = Math.max(BLOOM_MIN_BITS, n * BLOOM_BITS_PER_ITEM);
  m = Math.ceil(m / 8) * 8;
  return { m, k: BLOOM_K };
}

function bloomIndices(item: string, m: number, k: number): number[] {
  const out: number[] = [];
  const mBig = BigInt(m);
  for (let i = 0; i < k; i++) {
    const digest = sha256Hex(`${i}:${item}`);
    const big = BigInt("0x" + digest.slice(0, 16));
    out.push(Number(big % mBig));
  }
  return out;
}

function bloomAdd(bits: Buffer, m: number, k: number, item: string): void {
  for (const idx of bloomIndices(item, m, k)) {
    bits[idx >> 3] = (bits[idx >> 3] ?? 0) | (1 << (idx & 7));
  }
}

export function bloomTest(
  bits: Buffer, m: number, k: number, item: string,
): boolean {
  for (const idx of bloomIndices(item, m, k)) {
    if (((bits[idx >> 3] ?? 0) & (1 << (idx & 7))) === 0) return false;
  }
  return true;
}

// ---------------------------------------------------------------------
// Archive line format — must equal audit_client.py AuditRow.to_dict():
// base fields always present (null when unset), classifier fields only
// when present. canonicalJson keeps key order identical cross-substrate.
// ---------------------------------------------------------------------

export function archiveDictFromRow(row: AuditRow): Record<string, unknown> {
  const d: Record<string, unknown> = {
    id: row.id,
    ts: row.ts,
    actor: row.actor,
    actor_id: row.actor_id,
    tool: row.tool,
    command: row.command,
    args: row.args,
    target: row.target ?? null,
    outcome: row.outcome,
    outcome_detail: row.outcome_detail ?? null,
    constitution_rev: row.constitution_rev ?? null,
    grant_token: row.grant_token ?? null,
    c_flags: row.c_flags,
    prev_hash: row.prev_hash,
    hash: row.hash,
  };
  if (row.policy_revision !== undefined) d["policy_revision"] = row.policy_revision;
  if (row.classify_rule_ids !== undefined) d["classify_rule_ids"] = row.classify_rule_ids;
  if (row.classify_evidence !== undefined) d["classify_evidence"] = row.classify_evidence;
  if (row.classify_overall_verdict !== undefined)
    d["classify_overall_verdict"] = row.classify_overall_verdict;
  if (row.classify_verdict_reason !== undefined)
    d["classify_verdict_reason"] = row.classify_verdict_reason;
  return d;
}

export function defaultArchiveRoot(dbPath: string): string {
  if (dbPath && dbPath !== ":memory:") {
    return join(dirname(dbPath), "audit-archive");
  }
  const home = process.env["AIOSH_HOME"]
    ?? `${process.env["HOME"] ?? "/tmp"}/.aios`;
  return join(home, "audit-archive");
}

function dbFilePath(db: Database.Database): string {
  const rows = db.pragma("database_list") as Array<{ name: string; file: string }>;
  const main = rows.find((r) => r.name === "main");
  return main?.file ?? "";
}

function rotateRowProto(opts: {
  rotated: boolean;
  actor: string;
  actorId: string;
  constitutionRev?: string;
  grantToken?: string;
  args: Record<string, unknown>;
  outcome: "ok" | "refused" | "error";
  outcomeDetail?: string;
}): Omit<AuditRow, "id" | "prev_hash" | "hash"> {
  return {
    ts: new Date().toISOString(),
    actor: opts.actor as AuditRow["actor"],
    actor_id: opts.actorId,
    tool: "audit.rotate",
    command: "audit.rotate",
    args: opts.args,
    target: undefined,
    outcome: opts.outcome,
    outcome_detail: opts.outcomeDetail,
    constitution_rev: opts.constitutionRev,
    grant_token: opts.grantToken,
    c_flags: { c1: false, c2: false, c3: opts.rotated, c4: true },
  };
}

// ---------------------------------------------------------------------
// rotate
// ---------------------------------------------------------------------

export interface RotateOptions {
  keepRows?: number;
  dryRun?: boolean;
  archiveRoot?: string;
  actor?: string;
  actorId?: string;
  grantToken?: string;
  constitutionRev?: string;
}

export interface RotateResult {
  ok: boolean;
  rotated: boolean;
  dry_run?: boolean;
  error?: string;
  audit_id?: number;
  segment_id?: number;
  archived_rows?: number;
  keep_rows?: number;
  live_rows?: number;
  would_archive?: number;
  next_segment_id?: number;
  archive_path?: string;
  archive_sha256?: string;
  head_hash?: string;
}

export function rotate(
  db: Database.Database,
  ring: AuditRing,
  opts: RotateOptions = {},
): RotateResult {
  ensureSegmentsSchema(db);
  const actor = opts.actor ?? "system";
  const actorId = opts.actorId ?? "system:retention";
  const keep = Math.max(0, Math.floor(opts.keepRows ?? 0));

  const live = ring.verify();
  if (!live.ok) {
    const err = `refusing to rotate: live chain broken at row ${live.brokenAt}`;
    if (opts.dryRun) return { ok: false, rotated: false, dry_run: true, error: err };
    const row = ring.write(rotateRowProto({
      rotated: false, actor, actorId,
      constitutionRev: opts.constitutionRev, grantToken: opts.grantToken,
      args: { rotated: false, reason: "chain broken" },
      outcome: "refused", outcomeDetail: err,
    }));
    return { ok: false, rotated: false, error: err, audit_id: row.id };
  }

  const count = (db.prepare(
    "SELECT COUNT(*) AS n FROM audit_ring"
  ).get() as { n: number }).n;

  if (count <= keep) {
    if (opts.dryRun) {
      return { ok: true, rotated: false, dry_run: true,
               live_rows: count, would_archive: 0, keep_rows: keep };
    }
    const row = ring.write(rotateRowProto({
      rotated: false, actor, actorId,
      constitutionRev: opts.constitutionRev, grantToken: opts.grantToken,
      args: { rotated: false, reason: "nothing to rotate",
              live_rows: count, keep_rows: keep },
      outcome: "ok",
    }));
    return { ok: true, rotated: false, live_rows: count, audit_id: row.id };
  }

  const archiveCount = count - keep;
  if (opts.dryRun) {
    const next = (db.prepare(
      "SELECT COALESCE(MAX(segment_id),0)+1 AS next FROM audit_segments"
    ).get() as { next: number }).next;
    return { ok: true, rotated: false, dry_run: true,
             live_rows: count, would_archive: archiveCount,
             keep_rows: keep, next_segment_id: next };
  }

  const rawRows = db.prepare(
    "SELECT * FROM audit_ring ORDER BY id ASC LIMIT ?"
  ).all(archiveCount) as Array<Record<string, unknown>>;
  const rows = rawRows.map(auditRowFromSql);
  const firstId = rows[0]!.id;
  const lastId = rows[rows.length - 1]!.id;
  const genesisPrev = rows[0]!.prev_hash;
  const head = rows[rows.length - 1]!.hash;

  const segmentId = (db.prepare(
    "SELECT COALESCE(MAX(segment_id),0)+1 AS next FROM audit_segments"
  ).get() as { next: number }).next;

  const root = opts.archiveRoot ?? defaultArchiveRoot(dbFilePath(db));
  mkdirSync(root, { recursive: true });
  const archivePath = join(root, `segment-${String(segmentId).padStart(6, "0")}.jsonl`);

  const lines: string[] = [];
  const hashes: string[] = [];
  for (const row of rows) {
    lines.push(canonicalJson(archiveDictFromRow(row)));
    hashes.push(row.hash);
  }
  const content = lines.join("\n") + "\n";
  const contentBuf = Buffer.from(content, "utf8");
  const archiveSha = sha256Hex(contentBuf);

  const { m, k } = bloomParams(hashes.length);
  const bits = Buffer.alloc((m + 7) >> 3);
  for (const h of hashes) bloomAdd(bits, m, k, h);
  const bloomHex = bits.toString("hex");

  // Durable archive before the DB transaction so rows never leave the
  // live table without a persisted copy. Unique tmp name (crash
  // leftovers never block a retry), 0o600 perms, and refuse to
  // overwrite an existing segment file (covert-overwrite guard).
  if (existsSync(archivePath)) {
    throw new Error(`refusing to overwrite existing archive: ${archivePath}`);
  }
  const tmpPath = `${archivePath}.${Date.now()}.tmp`;
  try {
    writeFileSync(tmpPath, contentBuf, { mode: 0o600, flag: "wx" });
    renameSync(tmpPath, archivePath);
  } catch (e) {
    try { unlinkSync(tmpPath); } catch { /* ignore */ }
    throw e;
  }

  const rotationInput = rotateRowProto({
    rotated: true, actor, actorId,
    constitutionRev: opts.constitutionRev, grantToken: opts.grantToken,
    args: {
      rotated: true,
      segment_id: segmentId,
      first_row_id: firstId,
      last_row_id: lastId,
      row_count: rows.length,
      keep_rows: keep,
      head_hash: head,
      archive_path: archivePath,
      archive_sha256: archiveSha,
      bloom_m_bits: m,
      bloom_k: k,
    },
    outcome: "ok",
  });

  let rotationId: number;
  const tx = db.transaction(() => {
    db.prepare(`
      INSERT INTO audit_segments (
        segment_id, closed_at, first_row_id, last_row_id, row_count,
        genesis_prev_hash, head_hash, archive_path, archive_sha256,
        bloom_m_bits, bloom_k, bloom_hex
      ) VALUES (
        @segment_id, @closed_at, @first_row_id, @last_row_id, @row_count,
        @genesis_prev_hash, @head_hash, @archive_path, @archive_sha256,
        @bloom_m_bits, @bloom_k, @bloom_hex
      )
    `).run({
      segment_id: segmentId,
      closed_at: new Date().toISOString(),
      first_row_id: firstId,
      last_row_id: lastId,
      row_count: rows.length,
      genesis_prev_hash: genesisPrev,
      head_hash: head,
      archive_path: archivePath,
      archive_sha256: archiveSha,
      bloom_m_bits: m,
      bloom_k: k,
      bloom_hex: bloomHex,
    });
    db.prepare("DELETE FROM audit_ring WHERE id <= ?").run(lastId);
    // ring.write picks up prev_hash from headHash_(), which falls back to
    // the newest checkpoint head when the live table is empty — keeping
    // the chain continuous across the segment boundary.
    const row = ring.write(rotationInput);
    rotationId = row.id;
  });
  try {
    tx();
  } catch (e) {
    try { unlinkSync(archivePath); } catch { /* ignore */ }
    throw e;
  }

  return {
    ok: true, rotated: true,
    segment_id: segmentId,
    archived_rows: rows.length,
    keep_rows: keep,
    archive_path: archivePath,
    archive_sha256: archiveSha,
    head_hash: head,
    audit_id: rotationId!,
  };
}

// ---------------------------------------------------------------------
// verifyFull — replay archives in segment order, then the live table.
// ---------------------------------------------------------------------

export interface VerifyFullResult {
  ok: boolean;
  checked: number;
  brokenAt?: number;
  brokenSegment?: number;
  error?: string;
  segments: number;
  archiveChecked?: number;
  liveChecked?: number;
  anchor?: string;
  mode: "full";
}

export function verifyFull(
  db: Database.Database,
  archiveRoot?: string,
): VerifyFullResult {
  ensureSegmentsSchema(db);
  const segments = listSegments(db);
  let anchor = GENESIS_HASH;
  let archiveChecked = 0;

  for (const seg of segments) {
    let path = seg.archive_path;
    if (!existsSync(path) && archiveRoot) path = join(archiveRoot, basename(path));
    if (!existsSync(path)) {
      return { ok: false, checked: archiveChecked, mode: "full",
               error: `archive missing: ${path}`,
               brokenSegment: seg.segment_id, segments: segments.length };
    }
    const buf = readFileSync(path);
    if (sha256Hex(buf) !== seg.archive_sha256) {
      return { ok: false, checked: archiveChecked, mode: "full",
               error: `archive sha256 mismatch: ${path}`,
               brokenSegment: seg.segment_id, segments: segments.length };
    }
    if (seg.genesis_prev_hash !== anchor) {
      return { ok: false, checked: archiveChecked, mode: "full",
               error: `segment ${seg.segment_id} genesis_prev_hash does not ` +
                      `link to previous anchor`,
               brokenSegment: seg.segment_id, segments: segments.length };
    }
    const lines = buf.toString("utf8").split("\n").filter((l) => l.length > 0);
    if (lines.length !== seg.row_count) {
      return { ok: false, checked: archiveChecked, mode: "full",
               error: `segment ${seg.segment_id} line count ${lines.length} ` +
                      `!= recorded ${seg.row_count}`,
               brokenSegment: seg.segment_id, segments: segments.length };
    }
    let prev = anchor;
    for (let i = 0; i < lines.length; i++) {
      const obj = JSON.parse(lines[i]!) as Record<string, unknown> & {
        id: number; hash: string; prev_hash: string;
      };
      if (i === 0 && obj.id !== seg.first_row_id) {
        return { ok: false, checked: archiveChecked, mode: "full",
                 error: `segment ${seg.segment_id} first id mismatch`,
                 brokenSegment: seg.segment_id, segments: segments.length };
      }
      if (obj.prev_hash !== prev) {
        return { ok: false, checked: archiveChecked, mode: "full",
                 error: "archive prev_hash link broken",
                 brokenAt: obj.id, brokenSegment: seg.segment_id,
                 segments: segments.length };
      }
      const proto: Record<string, unknown> = {};
      for (const key of Object.keys(obj)) {
        if (key === "id" || key === "hash") continue;
        proto[key] = obj[key];
      }
      const expected = sha256Hex(prev + canonicalJson(proto));
      if (expected !== obj.hash) {
        return { ok: false, checked: archiveChecked, mode: "full",
                 error: "archive hash recompute mismatch",
                 brokenAt: obj.id, brokenSegment: seg.segment_id,
                 segments: segments.length };
      }
      prev = obj.hash;
      archiveChecked++;
    }
    if (prev !== seg.head_hash) {
      return { ok: false, checked: archiveChecked, mode: "full",
               error: `segment ${seg.segment_id} head_hash mismatch`,
               brokenSegment: seg.segment_id, segments: segments.length };
    }
    anchor = seg.head_hash;
  }

  // Live walk from the last checkpoint anchor.
  const rows = db.prepare(
    "SELECT * FROM audit_ring ORDER BY id ASC"
  ).all() as Array<Record<string, unknown>>;
  let prev = anchor;
  let i = 0;
  for (const rec of rows) {
    const row = auditRowFromSql(rec);
    if (row.prev_hash !== prev) {
      return { ok: false, checked: archiveChecked + i, brokenAt: row.id,
               mode: "full", segments: segments.length,
               archiveChecked, liveChecked: i };
    }
    const expected = sha256Hex(prev + canonicalJson(hashProtoFromRow(row)));
    if (expected !== row.hash) {
      return { ok: false, checked: archiveChecked + i, brokenAt: row.id,
               mode: "full", segments: segments.length,
               archiveChecked, liveChecked: i };
    }
    prev = row.hash;
    i++;
  }
  return { ok: true, checked: archiveChecked + i, anchor,
           segments: segments.length, archiveChecked, liveChecked: i,
           mode: "full" };
}

// ---------------------------------------------------------------------
// seen — membership query over live table + archived segments.
// ---------------------------------------------------------------------

export interface SeenResult {
  found: "live" | "archive" | "maybe" | "no";
  id?: number;
  segments: number[];
  note?: string;
}

export function seen(
  db: Database.Database,
  hashHex: string,
  opts: { exact?: boolean; archiveRoot?: string } = {},
): SeenResult {
  ensureSegmentsSchema(db);
  const h = (hashHex ?? "").trim().toLowerCase();
  const live = db.prepare(
    "SELECT id FROM audit_ring WHERE hash = ?"
  ).get(h) as { id: number } | undefined;
  if (live) return { found: "live", id: live.id, segments: [] };

  const maybe: number[] = [];
  const confirmed: number[] = [];
  for (const seg of listSegments(db)) {
    const bits = Buffer.from(seg.bloom_hex, "hex");
    if (!bloomTest(bits, seg.bloom_m_bits, seg.bloom_k, h)) continue;
    maybe.push(seg.segment_id);
    if (opts.exact) {
      let path = seg.archive_path;
      if (!existsSync(path) && opts.archiveRoot) {
        path = join(opts.archiveRoot, basename(path));
      }
      if (existsSync(path)) {
        for (const line of readFileSync(path, "utf8").split("\n")) {
          if (!line) continue;
          try {
            const obj = JSON.parse(line) as { hash?: unknown };
            if (typeof obj.hash === "string"
                && obj.hash.toLowerCase() === h) {
              confirmed.push(seg.segment_id);
              break;
            }
          } catch {
            continue;
          }
        }
      }
    }
  }
  if (opts.exact && confirmed.length > 0) {
    return { found: "archive", segments: confirmed };
  }
  if (opts.exact && maybe.length > 0) {
    return { found: "maybe", segments: maybe,
             note: "bloom positive, exact scan inconclusive" };
  }
  if (maybe.length > 0) return { found: "maybe", segments: maybe };
  return { found: "no", segments: [] };
}
