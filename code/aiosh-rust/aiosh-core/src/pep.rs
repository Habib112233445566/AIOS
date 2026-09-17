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

/// Lexically normalize path to collapse redundant slashes and `.` / `..` traversal components.
pub fn normalize_path_str(p: &str) -> String {
    let p_clean = p.replace('\\', "/");
    let is_abs = p_clean.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for seg in p_clean.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            parts.pop();
        } else {
            parts.push(seg);
        }
    }
    let joined = parts.join("/");
    if is_abs {
        format!("/{}", joined)
    } else {
        joined
    }
}

/// Attempts `resolve_existing_prefix` will make before keying a path lexically.
/// Each attempt is one `canonicalize` that fails fast once the path stops existing, so
/// this only bounds the work a hostile, very deep path can ask for.
const MAX_CANONICAL_ASCENTS: usize = 64;

/// Canonical, comparable form of a path for `scope.paths` matching.
///
/// The allow/deny lists are strings, so a deny entry only survives contact with an
/// adversary if both sides are first reduced to the form the **filesystem** resolves them
/// to. Otherwise one directory can be spelled several ways and the denied spelling loses
/// (T-01537 S-18/S-19):
///
/// * **case** — a case-insensitive filesystem has one `C:\Secret`, not two;
/// * **8.3 short names** — `C:\PROGRA~1` *is* `C:\Program Files`;
/// * **trailing dots/spaces** — Windows drops them from a component;
/// * **symlinks/junctions** — a link to a denied directory is that directory.
///
/// The longest **existing** prefix is therefore resolved through the filesystem, and only
/// the tail that does not exist yet is kept lexically — and that tail is the normal case,
/// because a `store_path` usually names a file that is about to be created. Case folding
/// and dot/space stripping are applied **only** where the platform aliases them: POSIX
/// paths stay case- and space-sensitive, so no legitimate Linux or macOS name is refused
/// on Windows' account.
///
/// Deliberately stricter than the write path in one place: trailing dots/spaces are
/// stripped from *every* component, including intermediate ones Windows would treat as
/// distinct names. Over-matching a deny entry fails closed, which is the safe direction.
pub fn canonical_path_key(p: &str) -> String {
    let mut key = normalize_path_str(p);
    #[cfg(windows)]
    {
        key = key
            .split('/')
            .map(|seg| seg.trim_end_matches(['.', ' ']))
            .collect::<Vec<_>>()
            .join("/");
    }
    let key = resolve_existing_prefix(&key);
    #[cfg(windows)]
    {
        return key.to_lowercase();
    }
    #[cfg(not(windows))]
    key
}

/// Replace the longest existing prefix of an already separator-normalized path with its
/// filesystem-resolved form, re-appending the non-existent tail. Returns the input
/// unchanged when nothing along the path resolves (which is why a purely lexical
/// comparison still holds for paths that do not exist yet on either side).
fn resolve_existing_prefix(normalized: &str) -> String {
    let mut tail: Vec<String> = Vec::new();
    let mut head = normalized.to_string();
    for _ in 0..MAX_CANONICAL_ASCENTS {
        if let Ok(real) = std::fs::canonicalize(&head) {
            let mut out = canonical_to_key(&real.to_string_lossy());
            for seg in tail.iter().rev() {
                if !out.ends_with('/') {
                    out.push('/');
                }
                out.push_str(seg);
            }
            return out;
        }
        // Ascend one component, but never above a root or a drive prefix: `C:` alone
        // means "current directory on C:", which is not the same place as `C:\`.
        match head.rfind('/') {
            Some(i) if i > 0 && !head[..i].ends_with(':') => {
                tail.push(head[i + 1..].to_string());
                head.truncate(i);
            }
            _ => break,
        }
    }
    normalized.to_string()
}

/// Strip the extended-length prefix `std::fs::canonicalize` returns on Windows and
/// normalize separators, so keys from resolved and unresolved paths stay comparable.
fn canonical_to_key(real: &str) -> String {
    let s = real.replace('\\', "/");
    if let Some(rest) = s.strip_prefix("//?/UNC/") {
        format!("//{rest}")
    } else if let Some(rest) = s.strip_prefix("//?/") {
        rest.to_string()
    } else {
        s
    }
}

/// Path-based allow/deny check. Deny always wins.
///
/// Comparison happens on [`canonical_path_key`], so spelling the same location differently
/// (case, 8.3 short name, trailing dot/space, symlink) cannot move it across a deny entry.
pub fn path_allowed(target: Option<&str>, paths: &PathScope) -> bool {
    if paths.allow.is_empty() && paths.deny.is_empty() {
        return true;
    }
    let target = match target {
        Some(t) => t,
        None => return false,
    };
    let key_target = canonical_path_key(target);
    for p in &paths.deny {
        let key_p = canonical_path_key(p);
        if key_target == key_p
            || key_target.starts_with(&format!("{}/", key_p.trim_end_matches('/')))
        {
            return false;
        }
    }
    if paths.allow.is_empty() {
        // Deny-only policy: any non-denied target is allowed.
        return true;
    }
    for p in &paths.allow {
        let key_p = canonical_path_key(p);
        if key_target == key_p
            || key_target.starts_with(&format!("{}/", key_p.trim_end_matches('/')))
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
    // Read-only reconnaissance and discovery tools are reversible / non-destructive
    if tool == "pentest.network.interfaces"
        || tool == "pentest.wifi.scan"
        || tool == "pentest.arp.scan"
        || tool == "pentest.tshark"
        || tool.starts_with("network.")
        || tool.starts_with("wifi.scan")
        || tool.starts_with("aios.network.")
        || tool.starts_with("aios.wifi.scan")
    {
        return false;
    }

    tool.starts_with("fs.write")
        || tool.starts_with("pentest.")
        || tool.starts_with("aios.backup.")
        || tool.starts_with("aios.release.")
        || tool.starts_with("aios.toolchain.set")
        || tool.starts_with("toolchain.set")
        || tool.starts_with("aios.doc.set")
        || tool.starts_with("doc.set")
        || tool.starts_with("aios.evidence.record")
        || tool.starts_with("evidence.record")
        || tool.starts_with("aios.evidence.set")
        || tool.starts_with("evidence.set")
        || tool.starts_with("aios.session.action")
        || tool.starts_with("session.action")
        || tool.starts_with("aios.session.create")
        || tool.starts_with("session.create")
        // T-01537 S-3: the four state-changing Filesystem Layout tools persist to a
        // caller-named store, so they belong to the irreversible family. They were
        // reachable *only* because every call site also passes `require_grant = true`;
        // listing them here makes the PEP itself refuse an unauthenticated mutation
        // instead of depending on each caller remembering the flag. The read-only
        // `aios.fs_layout.*` tools stay reversible, so they are matched exactly.
        || tool == "aios.fs_layout.register"
        || tool == "aios.fs_layout.set_active"
        || tool == "aios.fs_layout.remove"
        || tool == "aios.fs_layout.import_fstab"
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
    ///
    /// [`check`](Self::check) for the common case where the call touches no
    /// filesystem path of its own. See [`check_with_paths`](Self::check_with_paths)
    /// for why that distinction exists.
    pub fn check(
        &self,
        grant_id: Option<&str>,
        tool: &str,
        target: Option<&str>,
    ) -> Result<(), String> {
        self.check_with_paths(grant_id, tool, target, &[])
    }

    /// Authoritative gate, extended with the **filesystem paths the call will
    /// actually touch** (T-01537 S-1).
    ///
    /// `target` is the value the audit rows are attributed to, and for tools whose
    /// operated-on object *is* a path (e.g. `pentest.aircrack-ng --capture`) it is
    /// also what `scope.paths` governs. That conflates two different things, and the
    /// `fs_layout` mutation surface is where the conflation bites: its audit target is
    /// a layout **id** (spec §9), so a `scope.paths` allow-list could never constrain
    /// the layout **store file** the call writes — and for `register`'s `spec` form the
    /// target is `None`, which skipped the path check entirely, so the same grant
    /// refused the inline form and silently authorized the spec form.
    ///
    /// `path_subjects` therefore carries the paths that identify themselves, and the
    /// rule is deliberately either/or: **if a call declares subjects, `scope.paths`
    /// governs those and `target` is treated as an attribution label, not a path.** A
    /// call that declares none keeps the historical reading of `target` unchanged. The
    /// alternative — checking both — is incoherent, because an opaque id such as
    /// `aios-uefi-standard-v1` can never be inside a directory allow-list, so a
    /// path-scoped grant would refuse `set_active`/`remove`/`import_fstab` no matter how
    /// correct their store path was. A grant with no `scope.paths` set is unaffected
    /// either way, because `path_allowed` treats an empty allow+deny list as
    /// unrestricted.
    pub fn check_with_paths(
        &self,
        grant_id: Option<&str>,
        tool: &str,
        target: Option<&str>,
        path_subjects: &[&str],
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
                } else if target.is_some()
                    && path_subjects.is_empty()
                    && !path_allowed(target, &g.scope.paths)
                {
                    return Err(format!(
                        "target '{}' blocked by grant scope.paths",
                        target.unwrap()
                    ));
                }
                // The paths this call will really write or read. Checked even when
                // `target` is `None` (register's `spec` form), which is exactly the
                // hole this closes.
                for subject in path_subjects {
                    if !path_allowed(Some(subject), &g.scope.paths) {
                        return Err(format!(
                            "path subject '{}' blocked by grant scope.paths",
                            subject
                        ));
                    }
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
    let mut buf = vec![0u8; bytes];
    #[cfg(unix)]
    {
        use std::io::Read;
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
    }
    #[cfg(not(unix))]
    {
        // OS CSPRNG via SQLite's underlying CryptGenRandom / BCryptGenRandom implementation
        unsafe {
            rusqlite::ffi::sqlite3_randomness(
                bytes as libc::c_int,
                buf.as_mut_ptr() as *mut libc::c_void,
            );
        }
    }
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

    // ---- T-01537 S-1: path subjects are checked, not just the audit target ----

    fn fs_layout_grant(paths: PathScope) -> (PepStore, String) {
        let s = store();
        let sc = GrantScope {
            tools: vec!["aios.fs_layout.*".into()],
            paths,
            ..Default::default()
        };
        let g = s.create(&sc, 3600, "agent:test", "abc123").unwrap();
        (s, g.grant_id)
    }

    #[test]
    fn path_subjects_are_blocked_outside_allow_list() {
        let (s, gid) = fs_layout_grant(PathScope {
            allow: vec!["/srv/layouts".into()],
            deny: vec![],
        });
        // The write target of a mutation is a policy subject.
        assert!(s
            .check_with_paths(
                Some(&gid),
                "aios.fs_layout.register",
                None,
                &["/srv/layouts/store.json"]
            )
            .is_ok());
        assert!(s
            .check_with_paths(
                Some(&gid),
                "aios.fs_layout.register",
                None,
                &["/etc/evil.json"]
            )
            .is_err());
        // A denied subject wins even when an allowed sibling is present.
        assert!(s
            .check_with_paths(
                Some(&gid),
                "aios.fs_layout.import_fstab",
                Some("layout-id"),
                &["/srv/layouts/store.json", "/root/secrets.json"]
            )
            .is_err());
        // With subjects declared, those govern: an in-scope store is allowed even
        // though the audit target is a layout id that is not a path. Checking both
        // would make every path-scoped grant unusable for this surface.
        assert!(s
            .check_with_paths(
                Some(&gid),
                "aios.fs_layout.set_active",
                Some("aios-uefi-standard-v1"),
                &["/srv/layouts/store.json"]
            )
            .is_ok());
        // Documented pre-fix hole, pinned so it cannot silently return: with no
        // subjects and a `None` target the path check is skipped entirely.
        assert!(s
            .check_with_paths(Some(&gid), "aios.fs_layout.register", None, &[])
            .is_ok());
    }

    #[test]
    fn path_subjects_leave_unscoped_grants_alone() {
        let (s, gid) = fs_layout_grant(PathScope::default());
        assert!(s
            .check_with_paths(
                Some(&gid),
                "aios.fs_layout.register",
                None,
                &["/anywhere/at/all/store.json"]
            )
            .is_ok());
        // The legacy 3-arg entry point keeps its exact behaviour.
        assert!(s.check(Some(&gid), "aios.fs_layout.register", None).is_ok());
    }

    // ---- T-01537 S-3: fs_layout mutations are irreversible to the PEP itself ----

    #[test]
    fn fs_layout_mutations_require_a_grant_without_the_call_site_flag() {
        let s = store();
        for tool in [
            "aios.fs_layout.register",
            "aios.fs_layout.set_active",
            "aios.fs_layout.remove",
            "aios.fs_layout.import_fstab",
        ] {
            assert!(
                s.check(None, tool, None).is_err(),
                "{} must be irreversible",
                tool
            );
        }
        // Read-only siblings stay ungated.
        for tool in [
            "aios.fs_layout.get",
            "aios.fs_layout.list",
            "aios.fs_layout.validate",
            "aios.fs_layout.fstab",
            "aios.fs_layout.probe",
            "aios.fs_layout.diff",
        ] {
            assert!(
                s.check(None, tool, None).is_ok(),
                "{} must stay reversible",
                tool
            );
        }
    }

    // ---- T-01537 S-18/S-19: scope.paths matching is canonical, not lexical ----

    #[test]
    fn path_keys_fold_platform_aliases() {
        // Spellings the filesystem treats as one directory must not slip past a deny
        // entry. These paths do not exist, so both sides stay lexical and the assertions
        // are deterministic on any machine.
        #[cfg(windows)]
        {
            let deny_only = PathScope {
                allow: vec![],
                deny: vec![r"C:\DenyDir".into()],
            };
            assert!(
                !path_allowed(Some(r"c:\denydir\x.json"), &deny_only),
                "case alias must still match the deny entry"
            );
            assert!(
                !path_allowed(Some(r"C:\DenyDir. \x.json"), &deny_only),
                "trailing dot/space alias must still match the deny entry"
            );
            assert!(!path_allowed(Some(r"C:\DenyDir\sub\..\x.json"), &deny_only));
            // And the allow list agrees, so a scoped grant is not refused for spelling
            // its own directory in a different case (the fail-closed half of the bug).
            let allow_only = PathScope {
                allow: vec![r"C:\AllowedDir".into()],
                deny: vec![],
            };
            assert!(path_allowed(Some(r"c:\alloweddir\store.json"), &allow_only));
        }
        #[cfg(not(windows))]
        {
            // POSIX names are case- and space-sensitive: those are different files, and
            // Windows' aliasing must not leak into POSIX policy decisions.
            assert!(path_allowed(
                Some("/DenyDir/x.json"),
                &PathScope { allow: vec![], deny: vec!["/denydir".into()] }
            ));
            assert!(path_allowed(
                Some("/DenyDir /x.json"),
                &PathScope { allow: vec![], deny: vec!["/DenyDir".into()] }
            ));
        }
    }

    #[test]
    fn path_keys_resolve_an_existing_prefix() {
        // 8.3 short names and symlinks can only be expanded by asking the filesystem, so
        // the key for a not-yet-existing tail must be built from the *resolved* form of
        // its longest existing prefix. That single property is what stops a short-name
        // spelling from escaping a deny entry.
        let dir = std::env::temp_dir();
        let tail = dir.join("pep-key-probe").join("nested").join("store.json");
        let key_dir = canonical_path_key(&dir.to_string_lossy());
        let key_tail = canonical_path_key(&tail.to_string_lossy());
        assert!(
            key_tail.starts_with(&format!("{}/", key_dir.trim_end_matches('/'))),
            "non-existent tail {} must stay under the resolved prefix {}",
            key_tail,
            key_dir
        );

        let deny_dir = PathScope {
            allow: vec![],
            deny: vec![dir.to_string_lossy().to_string()],
        };
        assert!(
            !path_allowed(Some(&tail.to_string_lossy()), &deny_dir),
            "a store below a denied existing directory must be denied"
        );
        let allow_dir = PathScope {
            allow: vec![dir.to_string_lossy().to_string()],
            deny: vec![],
        };
        assert!(path_allowed(Some(&tail.to_string_lossy()), &allow_dir));
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

    #[test]
    fn path_traversal_blocked() {
        let mut sc = PathScope::default();
        sc.allow = vec!["/tmp".into()];
        sc.deny = vec!["/etc".into()];
        assert!(!path_allowed(Some("/var/log/../../etc/shadow"), &sc));
        assert!(!path_allowed(Some("/etc//shadow"), &sc));
        assert!(!path_allowed(Some(r"\etc\shadow"), &sc));
    }
}
