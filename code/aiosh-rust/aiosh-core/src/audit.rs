//! Audit ring — append-only SQLite WAL table with a SHA-256 hash chain.
//!
//! Every row carries `prev_hash` (previous row's hash, or genesis) and
//! `hash = sha256(prev_hash || canonical_json(proto))` where proto is
//! the row without `id`/`hash`. Sprint-2 classifier columns are added
//! via an idempotent migration; old rows with NULL classifier fields
//! keep their original hashes because the verifier omits NULL fields
//! from the recomputed proto (same rule as the legacy TS/Python).

use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::Path;

use crate::canonical::{canonical, sha256_hex, utcnow_iso};
use crate::types::{AuditRow, CFlags, GENESIS_HASH};

/// Schema for the `audit_ring` table (matches legacy TS/Python exactly).
const AUDIT_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS audit_ring (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ts              TEXT NOT NULL,
    actor           TEXT NOT NULL,
    actor_id        TEXT NOT NULL,
    tool            TEXT NOT NULL,
    command         TEXT NOT NULL,
    args_json       TEXT NOT NULL,
    target          TEXT,
    outcome         TEXT NOT NULL,
    outcome_detail  TEXT,
    constitution_rev TEXT,
    grant_token     TEXT,
    c1              INTEGER NOT NULL DEFAULT 0,
    c2              INTEGER NOT NULL DEFAULT 0,
    c3              INTEGER NOT NULL DEFAULT 0,
    c4              INTEGER NOT NULL DEFAULT 0,
    prev_hash       TEXT NOT NULL,
    hash            TEXT NOT NULL UNIQUE
);
CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_ring(ts);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_ring(actor);
CREATE INDEX IF NOT EXISTS idx_audit_tool ON audit_ring(tool);
"#;

/// Sprint-2 classifier columns (idempotent ALTER TABLE migration).
const CLASSIFIER_COLUMNS: &[(&str, &str)] = &[
    ("policy_revision", "TEXT"),
    ("classify_rule_ids_json", "TEXT"),
    ("classify_evidence_json", "TEXT"),
    ("classify_overall_verdict", "TEXT"),
    ("classify_verdict_reason", "TEXT"),
];

/// Sprint-3 segments table (rotation checkpoints).
pub const SEGMENTS_SCHEMA: &str = r#"
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
"#;

pub struct OpenOptions {
    /// Path to the .db file. Defaults to $AIOSH_HOME/audit.db or
    /// $HOME/.aios/audit.db.
    pub path: Option<String>,
    /// Override home for the default path.
    pub home: Option<String>,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self { path: None, home: None }
    }
}

fn default_db_path(home_override: Option<&str>) -> String {
    let home = home_override
        .map(|s| s.to_string())
        .or_else(|| std::env::var("AIOSH_HOME").ok())
        .unwrap_or_else(|| format!("{}/.aios", std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())));
    format!("{}/audit.db", home)
}

fn row_to_audit(row: &Row) -> rusqlite::Result<AuditRow> {
    let args_json: String = row.get("args_json")?;
    let c1: i64 = row.get("c1")?;
    let c2: i64 = row.get("c2")?;
    let c3: i64 = row.get("c3")?;
    let c4: i64 = row.get("c4")?;

    // Sprint-2 classifier fields — parse when present.
    let policy_revision: Option<String> = row.get("policy_revision").unwrap_or(None);
    let rule_ids_json: Option<String> =
        row.get("classify_rule_ids_json").unwrap_or(None);
    let evidence_json: Option<String> =
        row.get("classify_evidence_json").unwrap_or(None);
    let overall: Option<String> =
        row.get("classify_overall_verdict").unwrap_or(None);
    let reason: Option<String> =
        row.get("classify_verdict_reason").unwrap_or(None);

    let classify_rule_ids = rule_ids_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());
    let classify_evidence = evidence_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

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
        c_flags: CFlags {
            c1: c1 == 1,
            c2: c2 == 1,
            c3: c3 == 1,
            c4: c4 == 1,
        },
        policy_revision,
        classify_rule_ids,
        classify_evidence,
        classify_overall_verdict: overall,
        classify_verdict_reason: reason,
        prev_hash: row.get("prev_hash")?,
        hash: row.get("hash")?,
    })
}

pub struct AuditRing {
    conn: Connection,
    path: String,
}

impl AuditRing {
    /// Wrap an existing connection (tests / retention full-verify).
    pub fn from_conn(conn: Connection, path: String) -> Self {
        Self { conn, path }
    }

    pub fn open(opts: OpenOptions) -> rusqlite::Result<Self> {
        let path = match &opts.path {
            Some(p) => p.clone(),
            None => default_db_path(opts.home.as_deref()),
        };
        if let Some(parent) = Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        let ring = Self { conn, path };
        ring.ensure_schema()?;
        Ok(ring)
    }

    /// Open an in-memory ring (tests).
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "journal_mode", "WAL").ok();
        conn.pragma_update(None, "synchronous", "FULL").ok();
        let ring = Self { conn, path: ":memory:".into() };
        ring.ensure_schema()?;
        Ok(ring)
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn ensure_schema(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(AUDIT_SCHEMA)?;
        // Idempotent Sprint-2 migration: add classifier columns.
        let existing: Vec<String> = self
            .conn
            .prepare("PRAGMA table_info(audit_ring)")?
            .query_map([], |r| r.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for (col, decl) in CLASSIFIER_COLUMNS {
            if !existing.iter().any(|c| c == col) {
                self.conn
                    .execute(&format!("ALTER TABLE audit_ring ADD COLUMN {} {}", col, decl), [])?;
            }
        }
        // Sprint-3 segments table.
        self.conn.execute_batch(SEGMENTS_SCHEMA)?;
        Ok(())
    }

    pub fn ensure_pep_schema(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(PEP_SCHEMA)?;
        Ok(())
    }

    /// Chain head: last live row's hash, else newest segment checkpoint
    /// head, else genesis.
    pub fn head_hash(&self) -> rusqlite::Result<String> {
        let row = self
            .conn
            .query_row(
                "SELECT hash FROM audit_ring ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        if let Some(h) = row {
            return Ok(h);
        }
        self.anchor_hash()
    }

    /// Sprint-3: chain anchor for the live segment.
    fn anchor_hash(&self) -> rusqlite::Result<String> {
        let has_segments: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_segments'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)
            .unwrap_or(false);
        if has_segments {
            let seg: Option<String> = self
                .conn
                .query_row(
                    "SELECT head_hash FROM audit_segments ORDER BY segment_id DESC LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(h) = seg {
                return Ok(h);
            }
        }
        Ok(GENESIS_HASH.to_string())
    }

    /// Latest segment checkpoint head (if any) — for Python-compatible
    /// `latest_segment_head`.
    pub fn latest_segment_head(&self) -> rusqlite::Result<Option<String>> {
        let row = self
            .conn
            .query_row(
                "SELECT head_hash FROM audit_segments ORDER BY segment_id DESC LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        Ok(row)
    }

    /// Append one row to the ring. Computes prev_hash from the head and
    /// the chain hash from the canonical proto.
    pub fn write(&mut self, input: AuditRowInput) -> rusqlite::Result<AuditRow> {
        let prev_hash = self.head_hash()?;
        let mut proto_map = match input.hash_proto_with_prev(&prev_hash) {
            serde_json::Value::Object(m) => m,
            _ => unreachable!(),
        };
        proto_map.insert("prev_hash".into(), serde_json::Value::String(prev_hash.clone()));
        let proto = serde_json::Value::Object(proto_map);
        let hash = sha256_hex(&format!("{}{}", prev_hash, canonical(&proto)));

        let args_canonical = canonical(&input.args);
        let rule_ids_json = input
            .classify_rule_ids
            .as_ref()
            .map(|ids| canonical(&serde_json::Value::Array(ids.iter().map(|s| serde_json::Value::String(s.clone())).collect())));
        let evidence_json = input
            .classify_evidence
            .as_ref()
            .map(|ev| canonical(ev));

        self.conn.execute(
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
                input.ts,
                input.actor,
                input.actor_id,
                input.tool,
                input.command,
                args_canonical,
                input.target,
                input.outcome,
                input.outcome_detail,
                input.constitution_rev,
                input.grant_token,
                input.c_flags.c1 as i64,
                input.c_flags.c2 as i64,
                input.c_flags.c3 as i64,
                input.c_flags.c4 as i64,
                input.policy_revision,
                rule_ids_json,
                evidence_json,
                input.classify_overall_verdict,
                input.classify_verdict_reason,
                prev_hash,
                hash,
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(AuditRow {
            id,
            ts: input.ts,
            actor: input.actor,
            actor_id: input.actor_id,
            tool: input.tool,
            command: input.command,
            args: input.args,
            target: input.target,
            outcome: input.outcome,
            outcome_detail: input.outcome_detail,
            constitution_rev: input.constitution_rev,
            grant_token: input.grant_token,
            c_flags: input.c_flags,
            policy_revision: input.policy_revision,
            classify_rule_ids: input.classify_rule_ids,
            classify_evidence: input.classify_evidence,
            classify_overall_verdict: input.classify_overall_verdict,
            classify_verdict_reason: input.classify_verdict_reason,
            prev_hash,
            hash,
        })
    }

    /// Tail the last n rows in ascending id order.
    pub fn tail(&self, n: i64) -> rusqlite::Result<Vec<AuditRow>> {
        let safe = n.clamp(1, 1024);
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM audit_ring ORDER BY id DESC LIMIT ?")?;
        let rows = stmt
            .query_map(params![safe], row_to_audit)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut rows = rows;
        rows.reverse();
        Ok(rows)
    }

    /// Verify the live ring anchored at the newest checkpoint head.
    pub fn verify(&self) -> rusqlite::Result<VerifyResult> {
        let anchor = self.anchor_hash()?;
        let segment_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM audit_segments", [], |r| r.get(0))
            .unwrap_or(0);
        let mut stmt = self.conn.prepare("SELECT * FROM audit_ring ORDER BY id ASC")?;
        let rows = stmt
            .query_map([], row_to_audit)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut prev = anchor.clone();
        let mut checked = 0usize;
        for row in &rows {
            if row.prev_hash != prev {
                return Ok(VerifyResult {
                    ok: false,
                    checked,
                    broken_at: Some(row.id),
                    anchor: Some(anchor),
                    segments: segment_count,
                    mode: "live".into(),
                    ..Default::default()
                });
            }
            let expected = sha256_hex(&format!("{}{}", prev, canonical(&row.hash_proto())));
            if expected != row.hash {
                return Ok(VerifyResult {
                    ok: false,
                    checked,
                    broken_at: Some(row.id),
                    anchor: Some(anchor),
                    segments: segment_count,
                    mode: "live".into(),
                    ..Default::default()
                });
            }
            prev = row.hash.clone();
            checked += 1;
        }
        Ok(VerifyResult {
            ok: true,
            checked,
            broken_at: None,
            anchor: Some(anchor),
            segments: segment_count,
            mode: "live".into(),
            ..Default::default()
        })
    }

    pub fn count(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM audit_ring", [], |r| r.get(0))
    }

    pub fn delete_rows_le(&self, last_id: i64) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM audit_ring WHERE id <= ?", params![last_id])?;
        Ok(())
    }

    /// Commit helper: ensure schemas are present (call before writes).
    pub fn prepare_for_write(&self) -> rusqlite::Result<()> {
        self.ensure_schema()?;
        self.ensure_pep_schema()?;
        Ok(())
    }

    pub fn close(self) {
        drop(self.conn);
    }
}

/// PEP grants schema (mirrors `pep.ts:PepStore.init_()`).
pub const PEP_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS pep_grants (
    grant_id          TEXT PRIMARY KEY,
    issued_at         TEXT NOT NULL,
    expires_at        TEXT NOT NULL,
    issued_to         TEXT NOT NULL,
    constitution_rev  TEXT NOT NULL,
    scope_json        TEXT NOT NULL,
    scope_hash        TEXT NOT NULL,
    revoked_at        TEXT
);
CREATE INDEX IF NOT EXISTS idx_grants_active
    ON pep_grants(issued_to) WHERE revoked_at IS NULL;
"#;

/// Input for `AuditRing::write` — everything except id/prev_hash/hash.
#[derive(Debug, Clone)]
pub struct AuditRowInput {
    pub ts: String,
    pub actor: String,
    pub actor_id: String,
    pub tool: String,
    pub command: String,
    pub args: serde_json::Value,
    pub target: Option<String>,
    pub outcome: String,
    pub outcome_detail: Option<String>,
    pub constitution_rev: Option<String>,
    pub grant_token: Option<String>,
    pub c_flags: CFlags,
    pub policy_revision: Option<String>,
    pub classify_rule_ids: Option<Vec<String>>,
    pub classify_evidence: Option<serde_json::Value>,
    pub classify_overall_verdict: Option<String>,
    pub classify_verdict_reason: Option<String>,
}

impl AuditRowInput {
    pub fn hash_proto_with_prev(&self, prev: &str) -> serde_json::Value {
        let row = AuditRow {
            id: 0,
            ts: self.ts.clone(),
            actor: self.actor.clone(),
            actor_id: self.actor_id.clone(),
            tool: self.tool.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            target: self.target.clone(),
            outcome: self.outcome.clone(),
            outcome_detail: self.outcome_detail.clone(),
            constitution_rev: self.constitution_rev.clone(),
            grant_token: self.grant_token.clone(),
            c_flags: self.c_flags.clone(),
            policy_revision: self.policy_revision.clone(),
            classify_rule_ids: self.classify_rule_ids.clone(),
            classify_evidence: self.classify_evidence.clone(),
            classify_overall_verdict: self.classify_overall_verdict.clone(),
            classify_verdict_reason: self.classify_verdict_reason.clone(),
            prev_hash: prev.to_string(),
            hash: String::new(),
        };
        row.hash_proto()
    }
}

impl Default for AuditRowInput {
    fn default() -> Self {
        Self {
            ts: utcnow_iso(),
            actor: "user".into(),
            actor_id: "user:anon@host".into(),
            tool: String::new(),
            command: String::new(),
            args: serde_json::json!({}),
            target: None,
            outcome: "ok".into(),
            outcome_detail: None,
            constitution_rev: None,
            grant_token: None,
            c_flags: CFlags::default(),
            policy_revision: None,
            classify_rule_ids: None,
            classify_evidence: None,
            classify_overall_verdict: None,
            classify_verdict_reason: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VerifyResult {
    pub ok: bool,
    pub checked: usize,
    pub broken_at: Option<i64>,
    pub anchor: Option<String>,
    pub segments: i64,
    pub mode: String,
    pub archive_checked: Option<i64>,
    pub live_checked: Option<i64>,
    pub error: Option<String>,
}

/// Active constitution revision — first 12 hex of sha256 of the file,
/// or the implicit `v0.0` when missing.
pub fn active_constitution_rev(path: Option<&str>) -> String {
    let p = path
        .map(|s| s.to_string())
        .or_else(|| std::env::var("AIOSH_CONSTITUTION").ok())
        .unwrap_or_else(|| "/content/AIOS_MERGED/mostimportanAIfolder/AI_CONSTITUTION.md".into());
    match std::fs::read(&p) {
        Ok(bytes) => crate::canonical::sha256_hex_bytes(&bytes)[..12].to_string(),
        Err(_) => "v0.0".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GENESIS_HASH;

    fn base_input(tool: &str) -> AuditRowInput {
        AuditRowInput {
            ts: "2026-08-21T06:59:00.000000Z".into(),
            actor: "user".into(),
            actor_id: "user:test@host".into(),
            tool: tool.into(),
            command: format!("{} cmd", tool),
            args: serde_json::json!({"target": "10.0.0.5"}),
            target: Some("10.0.0.5".into()),
            outcome: "ok".into(),
            constitution_rev: Some("v0.0".into()),
            c_flags: CFlags { c1: true, c4: true, ..Default::default() },
            ..Default::default()
        }
    }

    #[test]
    fn chain_extends_and_verifies() {
        let mut ring = AuditRing::open_in_memory().unwrap();
        let r1 = ring.write(base_input("pentest.nmap")).unwrap();
        assert_eq!(r1.id, 1);
        assert_eq!(r1.prev_hash, GENESIS_HASH);
        let r2 = ring.write(base_input("pentest.nmap")).unwrap();
        assert_eq!(r2.id, 2);
        assert_eq!(r2.prev_hash, r1.hash);
        assert_ne!(r1.hash, r2.hash);

        let v = ring.verify().unwrap();
        assert!(v.ok, "chain must verify: {:?}", v);
        assert_eq!(v.checked, 2);
    }

    #[test]
    fn verify_detects_tampering() {
        let mut ring = AuditRing::open_in_memory().unwrap();
        ring.write(base_input("pentest.nmap")).unwrap();
        let r2 = ring.write(base_input("pentest.nmap")).unwrap();
        // Tamper: mutate the stored args of row 2.
        ring.conn()
            .execute(
                "UPDATE audit_ring SET args_json = ? WHERE id = ?",
                params!["{\"target\":\"1.1.1.1\"}", r2.id],
            )
            .unwrap();
        let v = ring.verify().unwrap();
        assert!(!v.ok);
        assert_eq!(v.broken_at, Some(r2.id));
    }

    #[test]
    fn tail_is_ascending() {
        let mut ring = AuditRing::open_in_memory().unwrap();
        ring.write(base_input("a")).unwrap();
        ring.write(base_input("b")).unwrap();
        ring.write(base_input("c")).unwrap();
        let t = ring.tail(2).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].id, 2);
        assert_eq!(t[1].id, 3);
    }

    #[test]
    fn hash_matches_manual_computation() {
        let mut ring = AuditRing::open_in_memory().unwrap();
        let row = ring.write(base_input("pentest.nmap")).unwrap();
        let proto = row.hash_proto();
        let expected =
            sha256_hex(&format!("{}{}", GENESIS_HASH, canonical(&proto)));
        assert_eq!(row.hash, expected);
    }
}
