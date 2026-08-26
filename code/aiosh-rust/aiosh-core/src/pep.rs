//! PEP (Policy Enforcement Point) grant tokens.
//!
//! Port of `code/aiosh-cli/src/pep.ts` + the Python grant helpers in
//! `audit_client.py`. Grants live in their own table in the audit WAL
//! database; grant creation/revocation are themselves audit rows.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::canonical::{canonical, sha256_hex};
use crate::types::{GrantScope, PathScope};

/// Match a tool name against a scope-tools glob, e.g. "pentest.nmap"
/// against ["pentest.*"] → true.
pub fn tool_glob_match(tool: &str, globs: &[String]) -> bool {
    if globs.is_empty() {
        return false;
    }
    for glob in globs {
        if glob == tool {
            return true;
        }
        if let Some(prefix) = glob.strip_suffix(".*") {
            if tool.starts_with(prefix) {
                return true;
            }
        }
    }
    false
}

/// Path-based allow/deny check. Deny always wins.
pub fn path_allowed(target: Option<&str>, paths: &PathScope) -> bool {
    if paths.allow.is_empty() && paths.deny.is_empty() {
        return true;
    }
    let target = match target {
        Some(t) => t,
        None => return false,
    };
    for p in &paths.deny {
        if target == p || target.starts_with(&format!("{}/", p))
            || (p.ends_with('/') && target.starts_with(p))
        {
            return false;
        }
    }
    if paths.allow.is_empty() {
        // Deny-only policy: any non-denied target is allowed.
        return true;
    }
    for p in &paths.allow {
        if target == p || target.starts_with(&format!("{}/", p))
            || (p.ends_with('/') && target.starts_with(p))
        {
            return true;
        }
    }
    false
}

/// CIDR-aware network scope check. Hostname entries are exact-match.
pub fn network_allowed(target: Option<&str>, networks: &[String]) -> bool {
    if networks.is_empty() {
        return true;
    }
    let target = match target {
        Some(t) => t,
        None => return false,
    };
    // Try parsing as IP address.
    if let Ok(addr) = target.parse::<std::net::IpAddr>() {
        for item in networks {
            if let Ok(net) = item.parse::<ipnet::IpNet>() {
                if net.contains(&addr) {
                    return true;
                }
            } else if target == item {
                return true;
            }
        }
        return false;
    }
    // Hostname: exact match only.
    networks.iter().any(|n| n == target)
}

pub fn is_irreversible(tool: &str) -> bool {
    tool.starts_with("fs.write")
        || tool.starts_with("pentest.")
        || tool == "system.reboot"
        || tool == "system.shutdown"
}

#[derive(Debug, Clone)]
pub struct PepGrant {
    pub grant_id: String,
    pub issued_at: String,
    pub expires_at: String,
    pub issued_to: String,
    pub constitution_rev: String,
    pub scope: GrantScope,
}

pub struct PepStore {
    conn: Connection,
}

impl PepStore {
    pub fn new(conn: Connection) -> rusqlite::Result<Self> {
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    pub fn init(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(crate::audit::PEP_SCHEMA)?;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn create(
        &self,
        scope: &GrantScope,
        ttl_seconds: i64,
        issued_to: &str,
        constitution_rev: &str,
    ) -> rusqlite::Result<PepGrant> {
        let ttl = if ttl_seconds <= 0 { 3600 } else { ttl_seconds };
        let now = Utc::now();
        let expires = now + chrono::Duration::seconds(ttl);
        let grant_id = format!("gr_{}", random_hex(8)?);
        let scope_json = serde_json::to_string(scope).unwrap_or_else(|_| "{}".into());
        let scope_hash = sha256_hex(&canonical(&scope_to_json(scope)));

        self.conn.execute(
            r#"INSERT INTO pep_grants
               (grant_id, issued_at, expires_at, issued_to,
                constitution_rev, scope_json, scope_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                grant_id,
                now.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
                expires.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
                issued_to,
                constitution_rev,
                scope_json,
                scope_hash,
            ],
        )?;
        Ok(PepGrant {
            grant_id,
            issued_at: now.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
            expires_at: expires.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
            issued_to: issued_to.into(),
            constitution_rev: constitution_rev.into(),
            scope: scope.clone(),
        })
    }

    pub fn revoke(&self, grant_id: &str) -> rusqlite::Result<bool> {
        let n = self
            .conn
            .execute(
                "UPDATE pep_grants SET revoked_at = ?1 WHERE grant_id = ?2 AND revoked_at IS NULL",
                params![Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(), grant_id],
            )?;
        Ok(n > 0)
    }

    pub fn get(&self, grant_id: &str) -> rusqlite::Result<Option<PepGrant>> {
        let row = self
            .conn
            .query_row(
                "SELECT * FROM pep_grants WHERE grant_id = ?1",
                params![grant_id],
                |r| {
                    Ok((
                        r.get::<_, String>("grant_id")?,
                        r.get::<_, String>("issued_at")?,
                        r.get::<_, String>("expires_at")?,
                        r.get::<_, String>("issued_to")?,
                        r.get::<_, String>("constitution_rev")?,
                        r.get::<_, String>("scope_json")?,
                        r.get::<_, Option<String>>("revoked_at")?,
                    ))
                },
            )
            .optional()?;
        match row {
            None => Ok(None),
            Some((id, issued, expires, to, con_rev, scope_json, revoked)) => {
                if revoked.is_some() {
                    return Ok(None);
                }
                let scope: GrantScope =
                    serde_json::from_str(&scope_json).unwrap_or_default();
                Ok(Some(PepGrant {
                    grant_id: id,
                    issued_at: issued,
                    expires_at: expires,
                    issued_to: to,
                    constitution_rev: con_rev,
                    scope,
                }))
            }
        }
    }

    pub fn list(&self, active_only: bool) -> rusqlite::Result<Vec<PepGrant>> {
        let sql = if active_only {
            "SELECT * FROM pep_grants WHERE revoked_at IS NULL ORDER BY issued_at DESC"
        } else {
            "SELECT * FROM pep_grants ORDER BY issued_at DESC"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>("grant_id")?,
                    r.get::<_, String>("issued_at")?,
                    r.get::<_, String>("expires_at")?,
                    r.get::<_, String>("issued_to")?,
                    r.get::<_, String>("constitution_rev")?,
                    r.get::<_, String>("scope_json")?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows
            .into_iter()
            .map(|(id, issued, expires, to, con_rev, scope_json)| PepGrant {
                grant_id: id,
                issued_at: issued,
                expires_at: expires,
                issued_to: to,
                constitution_rev: con_rev,
                scope: serde_json::from_str(&scope_json).unwrap_or_default(),
            })
            .collect())
    }

    /// Authoritative gate. Returns Ok(()) or Err(reason).
    pub fn check(
        &self,
        grant_id: Option<&str>,
        tool: &str,
        target: Option<&str>,
    ) -> Result<(), String> {
        match grant_id {
            None => {
                if is_irreversible(tool) {
                    Err(format!(
                        "irreversible tool '{}' requires explicit PEP grant",
                        tool
                    ))
                } else {
                    Ok(())
                }
            }
            Some(id) => {
                let g = self
                    .get(id)
                    .map_err(|e| format!("grant lookup failed: {}", e))?
                    .ok_or_else(|| format!("unknown or revoked grant: {}", id))?;
                // Expiry — fail CLOSED: a malformed timestamp refuses the
                // grant rather than silently treating it as unexpired.
                let now = Utc::now();
                let expires =
                    chrono::DateTime::parse_from_rfc3339(&g.expires_at)
                        .map(|d| d.with_timezone(&Utc))
                        .map_err(|_| ())
                        .and_then(|d| if d < now { Err(()) } else { Ok(d) });
                match expires {
                    Err(()) => {
                        return Err(format!(
                            "grant {} expired or has malformed expires_at",
                            id
                        ))
                    }
                    Ok(_) => {}
                }
                // Tool scope.
                if !tool_glob_match(tool, &g.scope.tools) {
                    return Err(format!(
                        "tool '{}' not in grant scope.tools={}",
                        tool,
                        serde_json::to_string(&g.scope.tools).unwrap_or_default()
                    ));
                }
                // Network targets use scope.networks; paths otherwise.
                let is_network_target = tool.starts_with("pentest.") || tool.starts_with("network.");
                if target.is_some() && is_network_target && !g.scope.networks.is_empty() {
                    if !network_allowed(target, &g.scope.networks) {
                        return Err(format!(
                            "target '{}' blocked by grant scope.networks",
                            target.unwrap()
                        ));
                    }
                } else if target.is_some() && !path_allowed(target, &g.scope.paths) {
                    return Err(format!(
                        "target '{}' blocked by grant scope.paths",
                        target.unwrap()
                    ));
                }
                Ok(())
            }
        }
    }
}

fn scope_to_json(scope: &GrantScope) -> Value {
    serde_json::json!({
        "tools": scope.tools,
        "networks": scope.networks,
        "paths": {
            "allow": scope.paths.allow,
            "deny": scope.paths.deny,
        },
        "max_irreversible": scope.max_irreversible,
    })
}

/// Random hex string of `bytes` bytes (16 hex chars for gr_ ids).
///
/// Grant ids are bearer tokens authorizing irreversible actions — they
/// MUST come from the OS CSPRNG, never from a userspace PRNG seeded
/// with time/addresses.
fn random_hex(bytes: usize) -> Result<String, rusqlite::Error> {
    use std::io::Read;
    let mut buf = vec![0u8; bytes];
    let mut f = std::fs::File::open("/dev/urandom").map_err(|e| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
            e.kind(),
            format!("grant id entropy unavailable (/dev/urandom): {e}"),
        )))
    })?;
    f.read_exact(&mut buf).map_err(|e| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
            e.kind(),
            format!("grant id entropy read failed: {e}"),
        )))
    })?;
    Ok(buf.iter().map(|b| format!("{:02x}", b)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn store() -> PepStore {
        PepStore::new(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn scope(tools: &[&str]) -> GrantScope {
        GrantScope {
            tools: tools.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn create_get_revoke() {
        let s = store();
        let g = s
            .create(&scope(&["pentest.*"]), 3600, "agent:test", "abc123")
            .unwrap();
        assert!(g.grant_id.starts_with("gr_"));
        assert!(s.get(&g.grant_id).unwrap().is_some());
        assert!(s.revoke(&g.grant_id).unwrap());
        assert!(s.get(&g.grant_id).unwrap().is_none());
    }

    #[test]
    fn check_requires_grant_for_pentest() {
        let s = store();
        let r = s.check(None, "pentest.nmap", Some("10.0.0.5"));
        assert!(r.is_err());
        let r = s.check(None, "aios.audit.tail", None);
        assert!(r.is_ok());
    }

    #[test]
    fn check_enforces_tool_scope() {
        let s = store();
        let g = s
            .create(&scope(&["pentest.nmap"]), 3600, "agent:test", "abc123")
            .unwrap();
        assert!(s.check(Some(&g.grant_id), "pentest.nmap", Some("10.0.0.5")).is_ok());
        assert!(s.check(Some(&g.grant_id), "pentest.nikto", Some("10.0.0.5")).is_err());
    }

    #[test]
    fn check_enforces_network_scope() {
        let s = store();
        let mut sc = scope(&["pentest.*"]);
        sc.networks = vec!["10.0.0.0/8".into()];
        let g = s.create(&sc, 3600, "agent:test", "abc123").unwrap();
        assert!(s.check(Some(&g.grant_id), "pentest.nmap", Some("10.1.2.3")).is_ok());
        assert!(s.check(Some(&g.grant_id), "pentest.nmap", Some("192.168.1.1")).is_err());
        assert!(s.check(Some(&g.grant_id), "pentest.nmap", Some("shodan.io")).is_err());
    }

    #[test]
    fn check_enforces_paths() {
        let s = store();
        let mut sc = scope(&["pentest.aircrack-ng"]);
        sc.paths.allow = vec!["/tmp/captures".into()];
        let g = s.create(&sc, 3600, "agent:test", "abc123").unwrap();
        assert!(s
            .check(Some(&g.grant_id), "pentest.aircrack-ng", Some("/tmp/captures/a.pcap"))
            .is_ok());
        assert!(s
            .check(Some(&g.grant_id), "pentest.aircrack-ng", Some("/etc/passwd"))
            .is_err());
    }

    #[test]
    fn tool_glob_matches() {
        assert!(tool_glob_match("pentest.nmap", &["pentest.*".to_string()]));
        assert!(!tool_glob_match("pentest.nmap", &["pentest.metasploit".to_string()]));
        assert!(tool_glob_match("pentest.nmap", &["pentest.nmap".to_string()]));
        assert!(!tool_glob_match("pentest.nmap", &[]));
    }

    #[test]
    fn path_deny_wins() {
        let mut sc = PathScope::default();
        sc.allow = vec!["/tmp".into()];
        sc.deny = vec!["/tmp/secret".into()];
        assert!(path_allowed(Some("/tmp/ok"), &sc));
        assert!(!path_allowed(Some("/tmp/secret/x"), &sc));
    }
}
