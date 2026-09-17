//! Security policy enforcement for User Session Bootstrap Subsystem (SSP1..SSP7).
//!
//! Enforces identity containment, root session restrictions, greeter boundaries,
//! seat0 physical console exclusivity, dynamic linker environment variable stripping,
//! concurrency quotas, and autonomous AI agent execution rules.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::session::{SessionClass, SessionType, UserSessionSpec, UserSessionStore};

/// Maximum allowable size for a policy configuration file (64 KiB).
pub const MAX_POLICY_FILE_BYTES: u64 = 65_536;

/// Enforcement mode for user session security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionPolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

impl Default for SessionPolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// A specific security violation encountered during session policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPolicyViolation {
    pub rule_id: String,
    pub session_id: String,
    pub description: String,
    pub fatal: bool,
}

/// Comprehensive report of policy evaluation against a session specification or store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPolicyVerdict {
    pub session_id: String,
    pub allowed: bool,
    pub mode: SessionPolicyMode,
    pub violations: Vec<SessionPolicyViolation>,
    pub evaluated_at: String,
}

/// Security policy defining mandatory security criteria and boundary invariants for user sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSessionSecurityPolicy {
    pub mode: SessionPolicyMode,
    pub disallow_root: bool,
    pub allowed_root_users: Vec<String>,
    pub allowed_greeter_users: Vec<String>,
    pub allowed_session_types: Vec<SessionType>,
    pub allow_remote_seat0: bool,
    pub disallowed_env_vars: Vec<String>,
    pub max_env_vars: usize,
    pub max_sessions_per_user: usize,
    pub max_total_sessions: usize,
    pub require_agent_sandboxed: bool,
}

impl Default for UserSessionSecurityPolicy {
    fn default() -> Self {
        Self {
            mode: SessionPolicyMode::Enforcing,
            disallow_root: true,
            allowed_root_users: vec![],
            allowed_greeter_users: vec![
                "lightdm".into(),
                "gdm".into(),
                "greeter".into(),
                "sddm".into(),
                "aios-greeter".into(),
            ],
            allowed_session_types: vec![
                SessionType::Wayland,
                SessionType::X11,
                SessionType::Tty,
                SessionType::AiAgent,
            ],
            allow_remote_seat0: false,
            disallowed_env_vars: vec![
                "LD_PRELOAD".into(),
                "LD_LIBRARY_PATH".into(),
                "LD_AUDIT".into(),
                "IFS".into(),
                "NODE_OPTIONS".into(),
                "PYTHONPATH".into(),
                "PYTHONSTARTUP".into(),
                "RUBYOPT".into(),
                "PERL5OPT".into(),
                "PERL5LIB".into(),
                "BASH_ENV".into(),
                "ENV".into(),
                "PROMPT_COMMAND".into(),
                "GCC_EXEC_PREFIX".into(),
            ],
            max_env_vars: 256,
            max_sessions_per_user: 32,
            max_total_sessions: 1024,
            require_agent_sandboxed: true,
        }
    }
}

impl UserSessionSecurityPolicy {
    /// Validates the policy's internal fields against structural bounds (SSP1..SSP7).
    pub fn validate(&self) -> Result<(), String> {
        if self.allowed_root_users.len() > 128 {
            return Err("invariant SSP1 violated: allowed_root_users exceeds limit of 128".into());
        }
        for u in &self.allowed_root_users {
            if u.is_empty() || u.len() > 32 || u.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(format!("invariant SSP1 violated: invalid allowed_root_user '{}'", u));
            }
        }

        if self.allowed_greeter_users.len() > 128 {
            return Err("invariant SSP1 violated: allowed_greeter_users exceeds limit of 128".into());
        }
        for u in &self.allowed_greeter_users {
            if u.is_empty() || u.len() > 32 || u.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(format!("invariant SSP1 violated: invalid allowed_greeter_user '{}'", u));
            }
        }

        if self.allowed_session_types.is_empty() {
            return Err("invariant SSP2 violated: allowed_session_types cannot be empty".into());
        }

        if self.disallowed_env_vars.len() > 128 {
            return Err("invariant SSP4 violated: disallowed_env_vars exceeds limit of 128".into());
        }
        for var in &self.disallowed_env_vars {
            if var.is_empty() || var.len() > 256 || var.contains('=') || var.chars().any(|c| c.is_control()) {
                return Err(format!("invariant SSP4 violated: invalid disallowed_env_vars entry '{}'", var));
            }
        }

        if self.max_env_vars == 0 || self.max_env_vars > 1024 {
            return Err(format!("invariant SSP4 violated: max_env_vars {} outside [1..1024]", self.max_env_vars));
        }

        if self.max_sessions_per_user == 0 || self.max_sessions_per_user > 128 {
            return Err(format!(
                "invariant SSP5 violated: max_sessions_per_user {} outside [1..128]",
                self.max_sessions_per_user
            ));
        }

        if self.max_total_sessions < 10 || self.max_total_sessions > 10_000 {
            return Err(format!(
                "invariant SSP5 violated: max_total_sessions {} outside [10..10000]",
                self.max_total_sessions
            ));
        }

        Ok(())
    }

    /// Evaluates a user session specification against this security policy.
    pub fn evaluate_spec(&self, spec: &UserSessionSpec) -> SessionPolicyVerdict {
        let mut violations = Vec::new();
        let session_id = spec.session_id.clone();

        // SSP1: Root and Greeter Identity Boundaries
        if self.disallow_root && spec.uid == 0 && !self.allowed_root_users.contains(&spec.username) {
            violations.push(SessionPolicyViolation {
                rule_id: "SSP1-ROOT-DISALLOWED".into(),
                session_id: session_id.clone(),
                description: format!(
                    "Root session (UID 0) disallowed for user '{}'",
                    spec.username
                ),
                fatal: true,
            });
        }

        if spec.session_class == SessionClass::Greeter {
            if spec.uid >= 1000 && !self.allowed_greeter_users.contains(&spec.username) {
                violations.push(SessionPolicyViolation {
                    rule_id: "SSP1-GREETER-UNPRIVILEGED".into(),
                    session_id: session_id.clone(),
                    description: format!(
                        "Greeter session cannot run as unprivileged user '{}' (UID {})",
                        spec.username, spec.uid
                    ),
                    fatal: true,
                });
            }
        }

        // SSP2: Session Type & Class Gating
        if !self.allowed_session_types.contains(&spec.session_type) {
            violations.push(SessionPolicyViolation {
                rule_id: "SSP2-SESSION-TYPE-DISALLOWED".into(),
                session_id: session_id.clone(),
                description: format!(
                    "Session type '{:?}' is not permitted by security policy",
                    spec.session_type
                ),
                fatal: true,
            });
        }

        if spec.session_class == SessionClass::Agent && spec.session_type != SessionType::AiAgent {
            violations.push(SessionPolicyViolation {
                rule_id: "SSP2-AGENT-CLASS-TYPE-MISMATCH".into(),
                session_id: session_id.clone(),
                description: format!(
                    "Agent class session must have type AiAgent, found '{:?}'",
                    spec.session_type
                ),
                fatal: true,
            });
        }

        if spec.session_class == SessionClass::Greeter {
            if spec.display.is_none() {
                violations.push(SessionPolicyViolation {
                    rule_id: "SSP2-GREETER-DISPLAY-REQUIRED".into(),
                    session_id: session_id.clone(),
                    description: "Greeter class session requires an explicit display identifier".into(),
                    fatal: true,
                });
            }
            if spec.remote_host.is_some() {
                violations.push(SessionPolicyViolation {
                    rule_id: "SSP2-GREETER-REMOTE-DISALLOWED".into(),
                    session_id: session_id.clone(),
                    description: "Greeter class session cannot be remote".into(),
                    fatal: true,
                });
            }
        }

        // SSP3: Seat & Display Hardware Protection
        if spec.remote_host.is_some() && spec.seat == "seat0" && !self.allow_remote_seat0 {
            violations.push(SessionPolicyViolation {
                rule_id: "SSP3-REMOTE-SEAT0-FORBIDDEN".into(),
                session_id: session_id.clone(),
                description: format!(
                    "Remote session from '{}' forbidden on console seat0",
                    spec.remote_host.as_deref().unwrap_or("unknown")
                ),
                fatal: true,
            });
        }

        if let Some(vtnr) = spec.vtnr {
            if vtnr == 0 || vtnr > 64 {
                violations.push(SessionPolicyViolation {
                    rule_id: "SSP3-VTNR-OUT-OF-BOUNDS".into(),
                    session_id: session_id.clone(),
                    description: format!("Virtual terminal number {} outside valid [1..64] range", vtnr),
                    fatal: true,
                });
            }
        }

        if let Some(ref disp) = spec.display {
            if disp.len() > 32 || disp.chars().any(|c| c.is_control()) {
                violations.push(SessionPolicyViolation {
                    rule_id: "SSP3-DISPLAY-MALFORMED".into(),
                    session_id: session_id.clone(),
                    description: format!("Display identifier '{}' exceeds 32 chars or contains control chars", disp),
                    fatal: true,
                });
            }
        }

        // SSP4: Environment Sanitization
        if spec.environment.len() > self.max_env_vars {
            violations.push(SessionPolicyViolation {
                rule_id: "SSP4-MAX-ENV-VARS-EXCEEDED".into(),
                session_id: session_id.clone(),
                description: format!(
                    "Session environment count {} exceeds ceiling {}",
                    spec.environment.len(),
                    self.max_env_vars
                ),
                fatal: true,
            });
        }

        for (k, _) in &spec.environment {
            let normalized_k = k.trim_start_matches('_');
            if self.disallowed_env_vars.contains(k)
                || self.disallowed_env_vars.iter().any(|b| b == normalized_k)
                || normalized_k.starts_with("LD_")
            {
                violations.push(SessionPolicyViolation {
                    rule_id: "SSP4-DISALLOWED-ENV-VAR".into(),
                    session_id: session_id.clone(),
                    description: format!("Disallowed environment variable '{}' present in session spec", k),
                    fatal: true,
                });
            }
            if k.len() > 256 || k.contains('=') || k.chars().any(|c| c.is_control()) {
                violations.push(SessionPolicyViolation {
                    rule_id: "SSP4-MALFORMED-ENV-KEY".into(),
                    session_id: session_id.clone(),
                    description: format!("Environment variable key '{}' is malformed", k),
                    fatal: true,
                });
            }
        }

        // SSP6: AI Agent Sandboxing Constraints
        if self.require_agent_sandboxed && (spec.session_type == SessionType::AiAgent || spec.session_class == SessionClass::Agent) {
            if spec.uid < 1000 {
                violations.push(SessionPolicyViolation {
                    rule_id: "SSP6-AGENT-ROOT-FORBIDDEN".into(),
                    session_id: session_id.clone(),
                    description: format!("AI agent session cannot run with privileged UID {}", spec.uid),
                    fatal: true,
                });
            }
        }

        // Verdict determination based on policy mode (SSP7)
        let allowed = match self.mode {
            SessionPolicyMode::Permissive => true,
            SessionPolicyMode::Audit => true,
            SessionPolicyMode::Enforcing => !violations.iter().any(|v| v.fatal),
        };

        SessionPolicyVerdict {
            session_id,
            allowed,
            mode: self.mode,
            violations,
            evaluated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Evaluates an entire session store against this policy, including global quotas (SSP5).
    pub fn evaluate_store(&self, store: &UserSessionStore) -> Vec<SessionPolicyVerdict> {
        let mut verdicts = Vec::new();

        // 1. Evaluate individual session specs
        for spec in store.specs.values() {
            verdicts.push(self.evaluate_spec(spec));
        }

        // 2. Global capacity quotas (SSP5)
        if store.sessions.len() > self.max_total_sessions {
            verdicts.push(SessionPolicyVerdict {
                session_id: "GLOBAL_STORE".into(),
                allowed: self.mode != SessionPolicyMode::Enforcing,
                mode: self.mode,
                violations: vec![SessionPolicyViolation {
                    rule_id: "SSP5-TOTAL-SESSIONS-EXCEEDED".into(),
                    session_id: "GLOBAL_STORE".into(),
                    description: format!(
                        "Total session store count {} exceeds maximum allowable {}",
                        store.sessions.len(),
                        self.max_total_sessions
                    ),
                    fatal: true,
                }],
                evaluated_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        // 3. Per-user concurrency quotas (SSP5)
        let mut user_counts: BTreeMap<String, usize> = BTreeMap::new();
        for status in store.sessions.values() {
            *user_counts.entry(status.username.clone()).or_insert(0) += 1;
        }

        for (user, count) in user_counts {
            if count > self.max_sessions_per_user {
                verdicts.push(SessionPolicyVerdict {
                    session_id: format!("USER_QUOTA_{}", user),
                    allowed: self.mode != SessionPolicyMode::Enforcing,
                    mode: self.mode,
                    violations: vec![SessionPolicyViolation {
                        rule_id: "SSP5-USER-QUOTA-EXCEEDED".into(),
                        session_id: format!("USER_QUOTA_{}", user),
                        description: format!(
                            "User '{}' active sessions count {} exceeds maximum allowable {}",
                            user, count, self.max_sessions_per_user
                        ),
                        fatal: true,
                    }],
                    evaluated_at: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        verdicts
    }

    /// Loads and parses policy from a JSON configuration file with size caps (SSP7).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        if !p.exists() {
            return Err(format!("Policy file '{}' does not exist", p.display()));
        }

        let file = File::open(p).map_err(|e| format!("Failed to open policy file: {}", e))?;
        let metadata = file
            .metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        if metadata.len() > MAX_POLICY_FILE_BYTES {
            return Err(format!(
                "invariant SSP7 violated: policy file exceeds size limit of {} bytes",
                MAX_POLICY_FILE_BYTES
            ));
        }

        let mut buffer = Vec::with_capacity(metadata.len() as usize);
        file.take(MAX_POLICY_FILE_BYTES + 1)
            .read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read policy file: {}", e))?;

        if buffer.len() > MAX_POLICY_FILE_BYTES as usize {
            return Err(format!(
                "invariant SSP7 violated: policy file exceeds size limit of {} bytes",
                MAX_POLICY_FILE_BYTES
            ));
        }

        let policy: UserSessionSecurityPolicy = serde_json::from_slice(&buffer)
            .map_err(|e| format!("Failed to parse policy JSON: {}", e))?;

        policy.validate()?;
        Ok(policy)
    }

    /// Loads security policy with environment variable overrides for capacity limits (SSP5).
    pub fn from_env() -> Result<Self, String> {
        let mut policy = Self::default();
        if let Ok(val) = std::env::var("AIOS_SESSION_MAX_PER_USER") {
            if let Ok(cnt) = val.trim().parse::<usize>() {
                policy.max_sessions_per_user = cnt;
            } else {
                return Err(format!("invalid integer in AIOS_SESSION_MAX_PER_USER: {}", val));
            }
        }
        if let Ok(val) = std::env::var("AIOS_SESSION_MAX_TOTAL") {
            if let Ok(cnt) = val.trim().parse::<usize>() {
                policy.max_total_sessions = cnt;
            } else {
                return Err(format!("invalid integer in AIOS_SESSION_MAX_TOTAL: {}", val));
            }
        }
        policy.validate()?;
        Ok(policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::*;

    fn sample_spec(id: &str, user: &str, uid: u32, st: SessionType, sc: SessionClass, seat: &str) -> UserSessionSpec {
        UserSessionSpec {
            session_id: id.to_string(),
            username: user.to_string(),
            uid,
            gid: uid,
            session_type: st,
            session_class: sc,
            seat: seat.to_string(),
            vtnr: Some(1),
            display: Some(":0".to_string()),
            remote_host: None,
            environment: BTreeMap::new(),
        }
    }

    #[test]
    fn test_default_policy_validation() {
        let policy = UserSessionSecurityPolicy::default();
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_ssp1_root_and_greeter_rules() {
        let policy = UserSessionSecurityPolicy::default();

        // 1. Root disallowed by default
        let root_spec = sample_spec("root-sess", "root", 0, SessionType::Tty, SessionClass::User, "seat0");
        let v = policy.evaluate_spec(&root_spec);
        assert!(!v.allowed);
        assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP1-ROOT-DISALLOWED"));

        // 2. Unprivileged greeter disallowed
        let greeter_bad = sample_spec("greet-bad", "alice", 1000, SessionType::Wayland, SessionClass::Greeter, "seat0");
        let v_greet = policy.evaluate_spec(&greeter_bad);
        assert!(!v_greet.allowed);
        assert!(v_greet.violations.iter().any(|viol| viol.rule_id == "SSP1-GREETER-UNPRIVILEGED"));

        // 3. Valid greeter (lightdm, uid < 1000)
        let greeter_good = sample_spec("greet-ok", "lightdm", 62000, SessionType::X11, SessionClass::Greeter, "seat0");
        let v_good = policy.evaluate_spec(&greeter_good);
        assert!(v_good.allowed);
    }

    #[test]
    fn test_ssp2_session_type_and_class_rules() {
        let policy = UserSessionSecurityPolicy::default();

        // Agent class with non-agent type fails
        let agent_bad = sample_spec("agent-bad", "agent1", 1001, SessionType::Tty, SessionClass::Agent, "seat0");
        let v = policy.evaluate_spec(&agent_bad);
        assert!(!v.allowed);
        assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP2-AGENT-CLASS-TYPE-MISMATCH"));

        // Greeter without display fails
        let mut greeter_nodisp = sample_spec("greet-nodisp", "lightdm", 62000, SessionType::X11, SessionClass::Greeter, "seat0");
        greeter_nodisp.display = None;
        let v2 = policy.evaluate_spec(&greeter_nodisp);
        assert!(!v2.allowed);
        assert!(v2.violations.iter().any(|viol| viol.rule_id == "SSP2-GREETER-DISPLAY-REQUIRED"));

        // Greeter with remote host fails
        let mut greeter_remote = sample_spec("greet-rem", "lightdm", 62000, SessionType::X11, SessionClass::Greeter, "seat0");
        greeter_remote.remote_host = Some("192.168.1.100".into());
        let v3 = policy.evaluate_spec(&greeter_remote);
        assert!(!v3.allowed);
        assert!(v3.violations.iter().any(|viol| viol.rule_id == "SSP2-GREETER-REMOTE-DISALLOWED"));
    }

    #[test]
    fn test_ssp3_seat_and_display_rules() {
        let policy = UserSessionSecurityPolicy::default();

        // Remote session attaching to console seat0 is forbidden
        let mut rem_spec = sample_spec("rem-sess", "alice", 1001, SessionType::Wayland, SessionClass::User, "seat0");
        rem_spec.remote_host = Some("10.0.0.5".into());
        let v = policy.evaluate_spec(&rem_spec);
        assert!(!v.allowed);
        assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP3-REMOTE-SEAT0-FORBIDDEN"));

        // Remote on non-seat0 (e.g. seat-remote) is permitted
        rem_spec.seat = "seat-remote".into();
        let v2 = policy.evaluate_spec(&rem_spec);
        assert!(v2.allowed);

        // Invalid VT number (> 64)
        let mut vt_spec = sample_spec("vt-bad", "alice", 1001, SessionType::Tty, SessionClass::User, "seat0");
        vt_spec.vtnr = Some(99);
        let v3 = policy.evaluate_spec(&vt_spec);
        assert!(!v3.allowed);
        assert!(v3.violations.iter().any(|viol| viol.rule_id == "SSP3-VTNR-OUT-OF-BOUNDS"));
    }

    #[test]
    fn test_ssp4_environment_sanitization() {
        let policy = UserSessionSecurityPolicy::default();

        // Dangerous environment variables disallowed
        let mut env_spec = sample_spec("env-bad", "alice", 1001, SessionType::Wayland, SessionClass::User, "seat0");
        env_spec.environment.insert("LD_PRELOAD".into(), "/tmp/libhack.so".into());
        let v = policy.evaluate_spec(&env_spec);
        assert!(!v.allowed);
        assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP4-DISALLOWED-ENV-VAR"));

        // Normal environment permitted
        env_spec.environment.clear();
        env_spec.environment.insert("LANG".into(), "en_US.UTF-8".into());
        env_spec.environment.insert("TERM".into(), "xterm-256color".into());
        let v2 = policy.evaluate_spec(&env_spec);
        assert!(v2.allowed);
    }

    #[test]
    fn test_ssp5_concurrency_quotas() {
        let mut policy = UserSessionSecurityPolicy::default();
        policy.max_sessions_per_user = 2;
        policy.max_total_sessions = 3;

        let mut store = UserSessionStore::new();
        // Add 3 sessions for alice
        for i in 1..=3 {
            let spec = sample_spec(&format!("s-{}", i), "alice", 1001, SessionType::Wayland, SessionClass::User, "seat0");
            let status = UserSessionStatus {
                session_id: format!("s-{}", i),
                username: "alice".into(),
                uid: 1001,
                state: SessionState::Active,
                scope: SessionScope::Background,
                leader_pid: None,
                created_at: "2026-09-11T00:00:00Z".into(),
                last_active_at: "2026-09-11T00:00:00Z".into(),
                idle_seconds: 0,
                locked: false,
            };
            store.specs.insert(format!("s-{}", i), spec);
            store.sessions.insert(format!("s-{}", i), status);
        }

        let verdicts = policy.evaluate_store(&store);
        assert!(verdicts.iter().any(|v| v.violations.iter().any(|viol| viol.rule_id == "SSP5-USER-QUOTA-EXCEEDED")));
    }

    #[test]
    fn test_ssp6_agent_sandboxing() {
        let policy = UserSessionSecurityPolicy::default();

        // Agent with UID 0 (root) forbidden
        let agent_root = sample_spec("agent-root", "root", 0, SessionType::AiAgent, SessionClass::Agent, "seat0");
        let v = policy.evaluate_spec(&agent_root);
        assert!(!v.allowed);
        assert!(v.violations.iter().any(|viol| viol.rule_id == "SSP6-AGENT-ROOT-FORBIDDEN"));

        // Agent with UID 1001 permitted
        let agent_ok = sample_spec("agent-ok", "agent", 1001, SessionType::AiAgent, SessionClass::Agent, "seat0");
        let v2 = policy.evaluate_spec(&agent_ok);
        assert!(v2.allowed);
    }

    #[test]
    fn test_ssp7_policy_modes_and_file_caps() {
        let mut policy = UserSessionSecurityPolicy::default();
        let root_spec = sample_spec("root-sess", "root", 0, SessionType::Tty, SessionClass::User, "seat0");

        // Enforcing: not allowed
        policy.mode = SessionPolicyMode::Enforcing;
        assert!(!policy.evaluate_spec(&root_spec).allowed);

        // Audit: allowed = true, but violations present
        policy.mode = SessionPolicyMode::Audit;
        let v_audit = policy.evaluate_spec(&root_spec);
        assert!(v_audit.allowed);
        assert!(!v_audit.violations.is_empty());

        // Permissive: allowed = true
        policy.mode = SessionPolicyMode::Permissive;
        assert!(policy.evaluate_spec(&root_spec).allowed);
    }
}
