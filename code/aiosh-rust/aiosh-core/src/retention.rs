//! Sprint 3 — audit-ring retention: checkpointed segment rotation.
//!
//! Port of `retention.py` / `retention.ts`. A rotation freezes the
//! oldest rows into an immutable JSONL archive, records a checkpoint in
//! `audit_segments` (ids, head hash, archive sha256, bloom filter), and
//! removes the archived rows from the live table only. Entries are never
//! destroyed (Constitution P-2); the live chain re-anchors at the newest
//! checkpoint head.
//!
//! Cross-substrate contract (must stay byte-identical with the legacy
//! implementations):
//!   - `audit_segments` DDL
//!   - archive line = canonical(row.to_dict())
//!   - bloom indices: sha256("{i}:{item}")[:8] as big-endian u64 % m,
//!     little-endian bit order inside bytes, stored as lowercase hex
//!   - rotation row: tool="audit.rotate", no classifier fields

use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::{Path, PathBuf};

use crate::audit::{AuditRing, AuditRowInput};
use crate::canonical::{canonical, sha256_hex, sha256_hex_bytes, utcnow_iso};
use crate::types::{AuditRow, CFlags, GENESIS_HASH};

pub const BLOOM_BITS_PER_ITEM: usize = 16;
pub const BLOOM_MIN_BITS: usize = 1024;
pub const BLOOM_K: usize = 8;

#[derive(Debug, Clone)]
pub struct SegmentRecord {
    pub segment_id: i64,
    pub closed_at: String,
    pub first_row_id: i64,
    pub last_row_id: i64,
    pub row_count: i64,
    pub genesis_prev_hash: String,
    pub head_hash: String,
    pub archive_path: String,
    pub archive_sha256: String,
    pub bloom_m_bits: i64,
    pub bloom_k: i64,
    pub bloom_hex: String,
}

fn segment_from_row(row: &Row) -> rusqlite::Result<SegmentRecord> {
    Ok(SegmentRecord {
        segment_id: row.get("segment_id")?,
        closed_at: row.get("closed_at")?,
        first_row_id: row.get("first_row_id")?,
        last_row_id: row.get("last_row_id")?,
        row_count: row.get("row_count")?,
        genesis_prev_hash: row.get("genesis_prev_hash")?,
        head_hash: row.get("head_hash")?,
        archive_path: row.get("archive_path")?,
        archive_sha256: row.get("archive_sha256")?,
        bloom_m_bits: row.get("bloom_m_bits")?,
        bloom_k: row.get("bloom_k")?,
        bloom_hex: row.get("bloom_hex")?,
    })
}

// ---------------------------------------------------------------------
// Bloom filter — deterministic, cross-language identical.
// ---------------------------------------------------------------------

pub fn bloom_params(n: usize) -> (usize, usize) {
    let mut m = std::cmp::max(BLOOM_MIN_BITS, n * BLOOM_BITS_PER_ITEM);
    m = ((m + 7) / 8) * 8;
    (m, BLOOM_K)
}

fn bloom_indices(item: &str, m: usize, k: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(k);
    for i in 0..k {
        let digest = sha256_hex(&format!("{}:{}", i, item));
        let first16 = &digest[..16];
        let big = u64::from_str_radix(first16, 16).unwrap_or(0);
        out.push((big % m as u64) as usize);
    }
    out
}

pub fn bloom_add(bits: &mut [u8], m: usize, k: usize, item: &str) {
    for idx in bloom_indices(item, m, k) {
        bits[idx >> 3] |= 1 << (idx & 7);
    }
}

pub fn bloom_test(bits: &[u8], m: usize, k: usize, item: &str) -> bool {
    for idx in bloom_indices(item, m, k) {
        if (bits[idx >> 3] & (1 << (idx & 7))) == 0 {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------
// Segments
// ---------------------------------------------------------------------


pub fn ensure_segments_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(crate::audit::SEGMENTS_SCHEMA)?;
    Ok(())
}

pub fn list_segments(conn: &Connection) -> rusqlite::Result<Vec<SegmentRecord>> {
    ensure_segments_schema(conn)?;
    let mut stmt = conn.prepare("SELECT * FROM audit_segments ORDER BY segment_id ASC")?;
    let rows = stmt
        .query_map([], segment_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn next_segment_id(conn: &Connection) -> rusqlite::Result<i64> {
    // COALESCE handles the empty-table case: MAX(...) is NULL and
    // NULL+1 stays NULL, which rusqlite would surface as a column-type
    // error on `Option<i64>`.
    let n: i64 = conn.query_row(
        "SELECT COALESCE(MAX(segment_id), 0) + 1 FROM audit_segments",
        [],
        |r| r.get(0),
    )?;
    Ok(n)
}

fn default_archive_root(conn: &Connection) -> PathBuf {
    if let Some(p) = conn_db_path(conn) {
        if let Some(parent) = Path::new(&p).parent() {
            return parent.join("audit-archive");
        }
    }
    let home = std::env::var("AIOSH_HOME")
        .unwrap_or_else(|_| format!("{}/.aios", std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())));
    PathBuf::from(home).join("audit-archive")
}

fn conn_db_path(conn: &Connection) -> Option<String> {
    let mut stmt = conn.prepare("PRAGMA database_list").ok()?;
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0).unwrap_or_default(), r.get::<_, String>(2).unwrap_or_default()))
        })
        .ok()?;
    for row in rows.flatten() {
        if row.0 == "main" && !row.1.is_empty() && row.1 != ":memory:" {
            return Some(row.1);
        }
    }
    None
}

fn rotate_row_proto(
    rotated: bool,
    actor: &str,
    actor_id: &str,
    constitution_rev: Option<&str>,
    grant_token: Option<&str>,
    args: serde_json::Value,
    outcome: &str,
    outcome_detail: Option<&str>,
) -> AuditRowInput {
    AuditRowInput {
        ts: utcnow_iso(),
        actor: actor.into(),
        actor_id: actor_id.into(),
        tool: "audit.rotate".into(),
        command: "audit.rotate".into(),
        args,
        target: None,
        outcome: outcome.into(),
        outcome_detail: outcome_detail.map(|s| s.into()),
        constitution_rev: constitution_rev.map(|s| s.into()),
        grant_token: grant_token.map(|s| s.into()),
        c_flags: CFlags {
            c1: false,
            c2: false,
            c3: rotated,
            c4: true,
        },
        ..Default::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RotateResult {
    pub ok: bool,
    pub rotated: bool,
    pub dry_run: bool,
    pub error: Option<String>,
    pub audit_id: Option<i64>,
    pub segment_id: Option<i64>,
    pub archived_rows: Option<i64>,
    pub keep_rows: Option<i64>,
    pub live_rows: Option<i64>,
    pub would_archive: Option<i64>,
    pub next_segment_id: Option<i64>,
    pub archive_path: Option<String>,
    pub archive_sha256: Option<String>,
    pub head_hash: Option<String>,
}

impl RotateResult {
    pub fn to_json(&self) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        m.insert("ok".into(), self.ok.into());
        m.insert("rotated".into(), self.rotated.into());
        if self.dry_run {
            m.insert("dry_run".into(), true.into());
        }
        if let Some(e) = &self.error {
            m.insert("error".into(), e.clone().into());
        }
        if let Some(v) = self.audit_id {
            m.insert("audit_id".into(), v.into());
        }
        if let Some(v) = self.segment_id {
            m.insert("segment_id".into(), v.into());
        }
        if let Some(v) = self.archived_rows {
            m.insert("archived_rows".into(), v.into());
        }
        if let Some(v) = self.keep_rows {
            m.insert("keep_rows".into(), v.into());
        }
        if let Some(v) = self.live_rows {
            m.insert("live_rows".into(), v.into());
        }
        if let Some(v) = self.would_archive {
            m.insert("would_archive".into(), v.into());
        }
        if let Some(v) = self.next_segment_id {
            m.insert("next_segment_id".into(), v.into());
        }
        if let Some(v) = &self.archive_path {
            m.insert("archive_path".into(), v.clone().into());
        }
        if let Some(v) = &self.archive_sha256 {
            m.insert("archive_sha256".into(), v.clone().into());
        }
        if let Some(v) = &self.head_hash {
            m.insert("head_hash".into(), v.clone().into());
        }
        serde_json::Value::Object(m)
    }
}

pub struct RotateOptions {
    pub keep_rows: i64,
    pub dry_run: bool,
    pub archive_root: Option<String>,
    pub actor: String,
    pub actor_id: String,
    pub grant_token: Option<String>,
    pub constitution_rev: Option<String>,
}

impl Default for RotateOptions {
    fn default() -> Self {
        Self {
            keep_rows: 0,
            dry_run: false,
            archive_root: None,
            actor: "system".into(),
            actor_id: "system:retention".into(),
            grant_token: None,
            constitution_rev: None,
        }
    }
}

pub fn rotate(
    conn: &Connection,
    ring: &mut AuditRing,
    opts: RotateOptions,
) -> rusqlite::Result<RotateResult> {
    ensure_segments_schema(conn)?;
    ring.ensure_schema()?;

    // 1. Live chain must verify before rotation.
    let live = ring.verify()?;
    if !live.ok {
        let err = format!("refusing to rotate: live chain broken at row {:?}", live.broken_at);
        if opts.dry_run {
            return Ok(RotateResult {
                ok: false,
                rotated: false,
                dry_run: true,
                error: Some(err),
                ..Default::default()
            });
        }
        let row = ring.write(rotate_row_proto(
            false,
            &opts.actor,
            &opts.actor_id,
            opts.constitution_rev.as_deref(),
            opts.grant_token.as_deref(),
            serde_json::json!({"rotated": false, "reason": "chain broken"}),
            "refused",
            Some(&err),
        ))?;
        return Ok(RotateResult {
            ok: false,
            rotated: false,
            error: Some(err),
            audit_id: Some(row.id),
            ..Default::default()
        });
    }

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_ring", [], |r| r.get(0))?;
    let keep = opts.keep_rows.max(0);
    if count <= keep {
        if opts.dry_run {
            return Ok(RotateResult {
                ok: true,
                rotated: false,
                dry_run: true,
                live_rows: Some(count),
                would_archive: Some(0),
                keep_rows: Some(keep),
                ..Default::default()
            });
        }
        let row = ring.write(rotate_row_proto(
            false,
            &opts.actor,
            &opts.actor_id,
            opts.constitution_rev.as_deref(),
            opts.grant_token.as_deref(),
            serde_json::json!({"rotated": false, "reason": "nothing to rotate",
                               "live_rows": count, "keep_rows": keep}),
            "ok",
            None,
        ))?;
        return Ok(RotateResult {
            ok: true,
            rotated: false,
            live_rows: Some(count),
            audit_id: Some(row.id),
            ..Default::default()
        });
    }

    let archive_count = count - keep;
    if opts.dry_run {
        let next = next_segment_id(conn)?;
        return Ok(RotateResult {
            ok: true,
            rotated: false,
            dry_run: true,
            live_rows: Some(count),
            would_archive: Some(archive_count),
            keep_rows: Some(keep),
            next_segment_id: Some(next),
            ..Default::default()
        });
    }

    // 2. Fetch the oldest rows to archive.
    let mut stmt = conn.prepare("SELECT * FROM audit_ring ORDER BY id ASC LIMIT ?1")?;
    let raw_rows = stmt
        .query_map(params![archive_count], row_to_audit)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let first_id = raw_rows[0].id;
    let last_id = raw_rows[raw_rows.len() - 1].id;
    let genesis_prev = raw_rows[0].prev_hash.clone();
    let head = raw_rows[raw_rows.len() - 1].hash.clone();

    let segment_id = next_segment_id(conn)?;

    let root = match &opts.archive_root {
        Some(r) => PathBuf::from(r),
        None => default_archive_root(conn),
    };
    std::fs::create_dir_all(&root).map_err(io_err)?;
    let archive_path = root.join(format!("segment-{:06}.jsonl", segment_id));

    // 3. Serialize archive content (canonical rows).
    let mut lines: Vec<String> = Vec::new();
    let mut hashes: Vec<String> = Vec::new();
    for row in &raw_rows {
        lines.push(canonical(&row.to_dict()));
        hashes.push(row.hash.clone());
    }
    let content = format!("{}\n", lines.join("\n"));
    let content_bytes = content.as_bytes();
    let archive_sha = sha256_hex_bytes(content_bytes);

    let (m, k) = bloom_params(hashes.len());
    let mut bits = vec![0u8; (m + 7) / 8];
    for h in &hashes {
        bloom_add(&mut bits, m, k, h);
    }
    let bloom_hex = bits.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    // 4. Atomic archive write (unique tmp, 0600, refuse overwrite).
    if archive_path.exists() {
        return Err(io_err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("refusing to overwrite existing archive: {}", archive_path.display()),
        )));
    }
    let tmp_path = archive_path.with_extension(format!(
        "jsonl.{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .map_err(io_err)?;
        f.write_all(content_bytes).map_err(io_err)?;
        f.sync_all().map_err(io_err)?;
        // 0600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600))
                .map_err(io_err)?;
        }
    }
    if let Err(e) = std::fs::rename(&tmp_path, &archive_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(io_err(e));
    }

    // 5. DB transaction: insert segment, delete archived rows, write
    //    rotation row. On failure, remove the archive.
    let rotation_input = rotate_row_proto(
        true,
        &opts.actor,
        &opts.actor_id,
        opts.constitution_rev.as_deref(),
        opts.grant_token.as_deref(),
        serde_json::json!({
            "rotated": true,
            "segment_id": segment_id,
            "first_row_id": first_id,
            "last_row_id": last_id,
            "row_count": raw_rows.len() as i64,
            "keep_rows": keep,
            "head_hash": head,
            "archive_path": archive_path.to_string_lossy().to_string(),
            "archive_sha256": archive_sha,
            "bloom_m_bits": m as i64,
            "bloom_k": k as i64,
        }),
        "ok",
        None,
    );

    // One real SQLite transaction: segment checkpoint + live-row delete
    // + rotation audit row either all commit or none do. The rotation
    // row is written on the SAME connection so it participates in the
    // transaction; its prev_hash is the live head AFTER the delete
    // (kept-tail hash, or — when keep_rows == 0 — the head of the
    // segment we just inserted).
    let tx_result = (|| -> rusqlite::Result<i64> {
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            r#"INSERT INTO audit_segments
               (segment_id, closed_at, first_row_id, last_row_id, row_count,
                 genesis_prev_hash, head_hash, archive_path, archive_sha256,
                 bloom_m_bits, bloom_k, bloom_hex)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
            params![
                segment_id,
                utcnow_iso(),
                first_id,
                last_id,
                raw_rows.len() as i64,
                genesis_prev,
                head,
                archive_path.to_string_lossy().to_string(),
                archive_sha,
                m as i64,
                k as i64,
                bloom_hex,
            ],
        )?;
        tx.execute("DELETE FROM audit_ring WHERE id <= ?1", params![last_id])?;
        let prev_hash: String = tx
            .query_row(
                "SELECT hash FROM audit_ring ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .or_else(|| {
                tx.query_row(
                    "SELECT head_hash FROM audit_segments ORDER BY segment_id DESC LIMIT 1",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .ok()
            })
            .unwrap_or_else(|| GENESIS_HASH.to_string());
        let proto = rotation_input.hash_proto_with_prev(&prev_hash);
        let hash = sha256_hex(&format!("{}{}", prev_hash, canonical(&proto)));
        tx.execute(
            r#"INSERT INTO audit_ring (
                ts, actor, actor_id, tool, command, args_json, target,
                outcome, outcome_detail, constitution_rev, grant_token,
                c1, c2, c3, c4,
                policy_revision, classify_rule_ids_json, classify_evidence_json,
                classify_overall_verdict, classify_verdict_reason,
                prev_hash, hash
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22
            )"#,
            params![
                rotation_input.ts,
                rotation_input.actor,
                rotation_input.actor_id,
                rotation_input.tool,
                rotation_input.command,
                canonical(&rotation_input.args),
                rotation_input.target,
                rotation_input.outcome,
                rotation_input.outcome_detail,
                rotation_input.constitution_rev,
                rotation_input.grant_token,
                rotation_input.c_flags.c1 as i64,
                rotation_input.c_flags.c2 as i64,
                rotation_input.c_flags.c3 as i64,
                rotation_input.c_flags.c4 as i64,
                rotation_input.policy_revision.as_deref(),
                rotation_input.classify_rule_ids.as_ref().map(|ids| canonical(&serde_json::Value::Array(ids.iter().map(|s| serde_json::Value::String(s.clone())).collect()))),
                rotation_input.classify_evidence.as_ref().map(|ev| canonical(ev)),
                rotation_input.classify_overall_verdict,
                rotation_input.classify_verdict_reason,
                prev_hash,
                hash,
            ],
        )?;
        let audit_id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(audit_id)
    })();

    match tx_result {
        Ok(audit_id) => Ok(RotateResult {
            ok: true,
            rotated: true,
            segment_id: Some(segment_id),
            archived_rows: Some(raw_rows.len() as i64),
            keep_rows: Some(keep),
            archive_path: Some(archive_path.to_string_lossy().to_string()),
            archive_sha256: Some(archive_sha),
            head_hash: Some(head),
            audit_id: Some(audit_id),
            ..Default::default()
        }),
        Err(e) => {
            let _ = std::fs::remove_file(&archive_path);
            Err(e)
        }
    }
}

fn row_to_audit(row: &Row) -> rusqlite::Result<AuditRow> {
    let args_json: String = row.get("args_json")?;
    let c1: i64 = row.get("c1")?;
    let c2: i64 = row.get("c2")?;
    let c3: i64 = row.get("c3")?;
    let c4: i64 = row.get("c4")?;
    let policy_revision: Option<String> = row.get("policy_revision").unwrap_or(None);
    let rule_ids_json: Option<String> = row.get("classify_rule_ids_json").unwrap_or(None);
    let evidence_json: Option<String> = row.get("classify_evidence_json").unwrap_or(None);
    let overall: Option<String> = row.get("classify_overall_verdict").unwrap_or(None);
    let reason: Option<String> = row.get("classify_verdict_reason").unwrap_or(None);
    Ok(AuditRow {
        id: row.get("id")?,
        ts: row.get("ts")?,
        actor: row.get("actor")?,
        actor_id: row.get("actor_id")?,
        tool: row.get("tool")?,
        command: row.get("command")?,
        args: serde_json::from_str(&args_json).unwrap_or(serde_json::json!({})),
        target: row.get("target")?,
        outcome: row.get("outcome")?,
        outcome_detail: row.get("outcome_detail")?,
        constitution_rev: row.get("constitution_rev")?,
        grant_token: row.get("grant_token")?,
        c_flags: CFlags { c1: c1 == 1, c2: c2 == 1, c3: c3 == 1, c4: c4 == 1 },
        policy_revision,
        classify_rule_ids: rule_ids_json.as_deref().and_then(|s| serde_json::from_str(s).ok()),
        classify_evidence: evidence_json.as_deref().and_then(|s| serde_json::from_str(s).ok()),
        classify_overall_verdict: overall,
        classify_verdict_reason: reason,
        prev_hash: row.get("prev_hash")?,
        hash: row.get("hash")?,
    })
}

fn io_err(e: std::io::Error) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(e))
}

// ---------------------------------------------------------------------
// verify_full — replay archives in segment order, then the live table.
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct VerifyFullResult {
    pub ok: bool,
    pub checked: usize,
    pub broken_at: Option<i64>,
    pub broken_segment: Option<i64>,
    pub error: Option<String>,
    pub segments: i64,
    pub archive_checked: usize,
    pub live_checked: usize,
    pub anchor: Option<String>,
    pub mode: String,
}

impl VerifyFullResult {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "ok": self.ok,
            "mode": self.mode,
            "checked": self.checked,
            "segments": self.segments,
            "archive_checked": self.archive_checked,
            "live_checked": self.live_checked,
            "anchor": self.anchor,
            "broken_at": self.broken_at,
            "broken_segment": self.broken_segment,
            "error": self.error,
        })
    }
}

pub fn verify_full(
    conn: &Connection,
    archive_root: Option<&str>,
) -> rusqlite::Result<VerifyFullResult> {
    ensure_segments_schema(conn)?;
    let segments = list_segments(conn)?;
    let mut anchor = GENESIS_HASH.to_string();
    let mut archive_checked = 0usize;

    for seg in &segments {
        let mut path = PathBuf::from(&seg.archive_path);
        if !path.exists() {
            if let Some(root) = archive_root {
                let candidate = PathBuf::from(root).join(
                    Path::new(&seg.archive_path).file_name().unwrap_or_default(),
                );
                if candidate.exists() {
                    path = candidate;
                }
            }
        }
        if !path.exists() {
            return Ok(VerifyFullResult {
                ok: false,
                mode: "full".into(),
                error: Some(format!("archive missing: {}", path.display())),
                broken_segment: Some(seg.segment_id),
                segments: segments.len() as i64,
                archive_checked,
                ..Default::default()
            });
        }
        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(e) => {
                return Ok(VerifyFullResult {
                    ok: false,
                    mode: "full".into(),
                    error: Some(format!("archive read failed: {}", e)),
                    broken_segment: Some(seg.segment_id),
                    segments: segments.len() as i64,
                    archive_checked,
                    ..Default::default()
                })
            }
        };
        if sha256_hex_bytes(&data) != seg.archive_sha256 {
            return Ok(VerifyFullResult {
                ok: false,
                mode: "full".into(),
                error: Some(format!("archive sha256 mismatch: {}", path.display())),
                broken_segment: Some(seg.segment_id),
                segments: segments.len() as i64,
                archive_checked,
                ..Default::default()
            });
        }
        if seg.genesis_prev_hash != anchor {
            return Ok(VerifyFullResult {
                ok: false,
                mode: "full".into(),
                error: Some(format!(
                    "segment {} genesis_prev_hash does not link to previous anchor",
                    seg.segment_id
                )),
                broken_segment: Some(seg.segment_id),
                segments: segments.len() as i64,
                archive_checked,
                ..Default::default()
            });
        }
        let text = String::from_utf8_lossy(&data);
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        if lines.len() as i64 != seg.row_count {
            return Ok(VerifyFullResult {
                ok: false,
                mode: "full".into(),
                error: Some(format!(
                    "segment {} line count {} != recorded {}",
                    seg.segment_id,
                    lines.len(),
                    seg.row_count
                )),
                broken_segment: Some(seg.segment_id),
                segments: segments.len() as i64,
                archive_checked,
                ..Default::default()
            });
        }
        let mut prev = anchor.clone();
        for (i, line) in lines.iter().enumerate() {
            let obj: serde_json::Value =
                match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => {
                        return Ok(VerifyFullResult {
                            ok: false,
                            mode: "full".into(),
                            error: Some("archive line is not valid JSON".into()),
                            broken_segment: Some(seg.segment_id),
                            segments: segments.len() as i64,
                            archive_checked,
                            ..Default::default()
                        })
                    }
                };
            let id = obj.get("id").and_then(|v| v.as_i64());
            let hash = obj.get("hash").and_then(|v| v.as_str()).unwrap_or("");
            let prev_hash = obj.get("prev_hash").and_then(|v| v.as_str()).unwrap_or("");
            if i == 0 && id != Some(seg.first_row_id) {
                return Ok(VerifyFullResult {
                    ok: false,
                    mode: "full".into(),
                    error: Some(format!("segment {} first id mismatch", seg.segment_id)),
                    broken_segment: Some(seg.segment_id),
                    segments: segments.len() as i64,
                    archive_checked,
                    ..Default::default()
                });
            }
            if prev_hash != prev {
                return Ok(VerifyFullResult {
                    ok: false,
                    mode: "full".into(),
                    error: Some("archive prev_hash link broken".into()),
                    broken_at: id,
                    broken_segment: Some(seg.segment_id),
                    segments: segments.len() as i64,
                    archive_checked,
                    ..Default::default()
                });
            }
            // Rebuild proto (strip id + hash) and recompute.
            let mut proto = match obj.clone() {
                serde_json::Value::Object(m) => m,
                _ => serde_json::Map::new(),
            };
            proto.remove("id");
            proto.remove("hash");
            let expected =
                sha256_hex(&format!("{}{}", prev, canonical(&serde_json::Value::Object(proto))));
            if expected != hash {
                return Ok(VerifyFullResult {
                    ok: false,
                    mode: "full".into(),
                    error: Some("archive hash recompute mismatch".into()),
                    broken_at: id,
                    broken_segment: Some(seg.segment_id),
                    segments: segments.len() as i64,
                    archive_checked,
                    ..Default::default()
                });
            }
            prev = hash.to_string();
            archive_checked += 1;
        }
        if prev != seg.head_hash {
            return Ok(VerifyFullResult {
                ok: false,
                mode: "full".into(),
                error: Some(format!("segment {} head_hash mismatch", seg.segment_id)),
                broken_segment: Some(seg.segment_id),
                segments: segments.len() as i64,
                archive_checked,
                ..Default::default()
            });
        }
        anchor = seg.head_hash.clone();
    }

    // Live walk from the last checkpoint anchor.
    let mut stmt = conn.prepare("SELECT * FROM audit_ring ORDER BY id ASC")?;
    let rows = stmt
        .query_map([], row_to_audit)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut prev = anchor.clone();
    let mut live_checked = 0usize;
    for row in &rows {
        if row.prev_hash != prev {
            return Ok(VerifyFullResult {
                ok: false,
                mode: "full".into(),
                broken_at: Some(row.id),
                segments: segments.len() as i64,
                archive_checked,
                live_checked,
                checked: archive_checked + live_checked,
                ..Default::default()
            });
        }
        let expected =
            sha256_hex(&format!("{}{}", prev, canonical(&row.hash_proto())));
        if expected != row.hash {
            return Ok(VerifyFullResult {
                ok: false,
                mode: "full".into(),
                broken_at: Some(row.id),
                segments: segments.len() as i64,
                archive_checked,
                live_checked,
                checked: archive_checked + live_checked,
                ..Default::default()
            });
        }
        prev = row.hash.clone();
        live_checked += 1;
    }
    Ok(VerifyFullResult {
        ok: true,
        checked: archive_checked + live_checked,
        anchor: Some(anchor),
        segments: segments.len() as i64,
        archive_checked,
        live_checked,
        mode: "full".into(),
        ..Default::default()
    })
}

// ---------------------------------------------------------------------
// seen — membership query over live + archived.
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SeenResult {
    pub found: String, // "live" | "archive" | "maybe" | "no"
    pub id: Option<i64>,
    pub segments: Vec<i64>,
    pub note: Option<String>,
}

impl SeenResult {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "found": self.found,
            "id": self.id,
            "segments": self.segments,
            "note": self.note,
        })
    }
}

pub fn seen(
    conn: &Connection,
    hash_hex: &str,
    exact: bool,
    archive_root: Option<&str>,
) -> rusqlite::Result<SeenResult> {
    ensure_segments_schema(conn)?;
    let h = hash_hex.trim().to_lowercase();
    let live: Option<i64> = conn
        .query_row("SELECT id FROM audit_ring WHERE hash = ?1", params![h], |r| r.get(0))
        .optional()?;
    if let Some(id) = live {
        return Ok(SeenResult { found: "live".into(), id: Some(id), segments: vec![], note: None });
    }

    let mut maybe: Vec<i64> = vec![];
    let mut confirmed: Vec<i64> = vec![];
    for seg in list_segments(conn)? {
        let bits = hex_to_bytes(&seg.bloom_hex);
        if !bloom_test(&bits, seg.bloom_m_bits as usize, seg.bloom_k as usize, &h) {
            continue;
        }
        maybe.push(seg.segment_id);
        if exact {
            let mut path = PathBuf::from(&seg.archive_path);
            if !path.exists() {
                if let Some(root) = archive_root {
                    let candidate = PathBuf::from(root).join(
                        Path::new(&seg.archive_path).file_name().unwrap_or_default(),
                    );
                    if candidate.exists() {
                        path = candidate;
                    }
                }
            }
            if path.exists() {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    for line in text.lines() {
                        if line.is_empty() {
                            continue;
                        }
                        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
                            if obj.get("hash").and_then(|v| v.as_str()).map(|s| s.to_lowercase())
                                == Some(h.clone())
                            {
                                confirmed.push(seg.segment_id);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    if exact && !confirmed.is_empty() {
        return Ok(SeenResult { found: "archive".into(), id: None, segments: confirmed, note: None });
    }
    if exact && !maybe.is_empty() {
        return Ok(SeenResult {
            found: "maybe".into(),
            id: None,
            segments: maybe,
            note: Some("bloom positive, exact scan inconclusive".into()),
        });
    }
    if !maybe.is_empty() {
        return Ok(SeenResult { found: "maybe".into(), id: None, segments: maybe, note: None });
    }
    Ok(SeenResult { found: "no".into(), id: None, segments: vec![], note: None })
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if let (Some(hi), Some(lo)) = (hex_val(bytes[i]), hex_val(bytes[i + 1])) {
            out.push((hi << 4) | lo);
        }
        i += 2;
    }
    out
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloom_filter_membership() {
        let (m, k) = bloom_params(100);
        let mut bits = vec![0u8; (m + 7) / 8];
        bloom_add(&mut bits, m, k, "abc123");
        bloom_add(&mut bits, m, k, "def456");
        assert!(bloom_test(&bits, m, k, "abc123"));
        assert!(bloom_test(&bits, m, k, "def456"));
        // Deterministic: same params, same item → same bits.
        let (m2, k2) = bloom_params(100);
        let mut bits2 = vec![0u8; (m2 + 7) / 8];
        bloom_add(&mut bits2, m2, k2, "abc123");
        bloom_add(&mut bits2, m2, k2, "def456");
        assert_eq!(bits, bits2);
    }

    /// Test helper: a ring backed by a temp-file DB plus a second
    /// connection to the same file (rusqlite 0.32 has no conn clone).
    fn ring_and_conn() -> (tempfile::TempDir, AuditRing, rusqlite::Connection) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.db");
        let ring =
            AuditRing::open(crate::audit::OpenOptions { path: Some(path.to_string_lossy().to_string()), home: None })
                .unwrap();
        ring.prepare_for_write().unwrap();
        let conn = rusqlite::Connection::open(&path).unwrap();
        (tmp, ring, conn)
    }

    #[test]
    fn rotate_archives_and_verifies_full() {
        let (_tmp, mut ring, conn) = ring_and_conn();
        for i in 0..10 {
            ring.write(AuditRowInput {
                tool: format!("pentest.nmap"),
                command: format!("nmap 10.0.0.{}", i),
                args: serde_json::json!({"target": format!("10.0.0.{}", i)}),
                target: Some(format!("10.0.0.{}", i)),
                outcome: "ok".into(),
                c_flags: CFlags { c1: true, c4: true, ..Default::default() },
                ..Default::default()
            })
            .unwrap();
        }
        assert_eq!(ring.count().unwrap(), 10);

        let tmp = tempfile::tempdir().unwrap();
        let res = rotate(
            &conn,
            &mut ring,
            RotateOptions {
                keep_rows: 3,
                archive_root: Some(tmp.path().to_string_lossy().to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(res.ok, "{:?}", res.error);
        assert!(res.rotated);
        assert_eq!(res.archived_rows, Some(7));
        assert_eq!(res.segment_id, Some(1));
        assert_eq!(ring.count().unwrap(), 4); // 3 kept + 1 rotation row

        // Live verify still passes (anchored at checkpoint).
        let v = ring.verify().unwrap();
        assert!(v.ok, "{:?}", v);

        // Full verify replays the archive.
        let full = verify_full(&conn, Some(tmp.path().to_string_lossy().as_ref())).unwrap();
        assert!(full.ok, "{:?}", full.error);
        assert_eq!(full.segments, 1);
        assert_eq!(full.archive_checked, 7);
        assert_eq!(full.live_checked, 4);

        // seen() finds an archived hash.
        let archived_hash = hashes_from_archive(&res).unwrap();
        let seen = seen(
            &conn,
            &archived_hash,
            true,
            Some(tmp.path().to_string_lossy().as_ref()),
        )
        .unwrap();
        assert_eq!(seen.found, "archive");
    }

    fn hashes_from_archive(res: &RotateResult) -> Option<String> {
        let p = res.archive_path.clone()?;
        let text = std::fs::read_to_string(p).ok()?;
        let first = text.lines().next()?;
        let obj: serde_json::Value = serde_json::from_str(first).ok()?;
        obj.get("hash").and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    #[test]
    fn rotate_refuses_broken_chain() {
        let (_tmp, mut ring, conn) = ring_and_conn();
        ring.write(AuditRowInput {
            tool: "a".into(),
            command: "a".into(),
            args: serde_json::json!({}),
            outcome: "ok".into(),
            ..Default::default()
        })
        .unwrap();
        // Tamper the chain (original args are `{}` — overwrite with a
        // different value so the recomputed hash diverges).
        conn.execute("UPDATE audit_ring SET args_json = '{\"tampered\":true}'", [])
            .unwrap();
        let tampered: String = conn.query_row("SELECT args_json FROM audit_ring WHERE id=1", [], |r| r.get(0)).unwrap();
        assert_eq!(tampered, "{\"tampered\":true}");
        assert!(!ring.verify().unwrap().ok);
        let res = rotate(
            &conn,
            &mut ring,
            RotateOptions {
                // Hermetic: never fall back to the shared home default.
                archive_root: Some(_tmp.path().join("archive").to_string_lossy().to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!res.ok);
        assert!(res.error.unwrap().contains("refusing to rotate"));
    }

    #[test]
    fn dry_run_writes_nothing() {
        let (_tmp, mut ring, conn) = ring_and_conn();
        for _ in 0..5 {
            ring.write(AuditRowInput {
                tool: "t".into(),
                command: "t".into(),
                args: serde_json::json!({}),
                outcome: "ok".into(),
                ..Default::default()
            })
            .unwrap();
        }
        let res = rotate(
            &conn,
            &mut ring,
            RotateOptions { keep_rows: 0, dry_run: true, ..Default::default() },
        )
        .unwrap();
        assert!(res.ok);
        assert!(res.dry_run);
        assert_eq!(res.would_archive, Some(5));
        assert_eq!(ring.count().unwrap(), 5);
        assert_eq!(list_segments(&conn).unwrap().len(), 0);
    }
}
