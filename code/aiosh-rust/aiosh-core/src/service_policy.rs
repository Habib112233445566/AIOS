//! Security policy enforcement for AIOS Init & Service Supervision Subsystem (SP1..SP6).
//!
//! Provides validation of service specifications (`ServiceSpec`) and service stores
//! (`ServiceStore`) against mandatory security criteria and privilege restrictions.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::service::{ServiceSpec, ServiceType};
use crate::service_service::ServiceStore;

/// Maximum allowable size for a policy configuration file (64 KiB).
pub const MAX_POLICY_FILE_BYTES: u64 = 65_536;

/// Enforcement mode for service supervision security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServicePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

impl Default for ServicePolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Security policy defining mandatory security criteria for supervised services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceSecurityPolicy {
    pub mode: ServicePolicyMode,
    pub prohibited_services: Vec<String>,
    pub prohibited_exec_paths: Vec<String>,
    pub disallow_root: bool,
    pub allowed_root_services: Vec<String>,
    pub require_service_user: bool,
    pub disallow_env_vars: Vec<String>,
    pub allowed_service_types: Vec<ServiceType>,
    pub max_env_vars: usize,
    pub max_timeout_secs: u64,
}

impl Default for ServiceSecurityPolicy {
    fn default() -> Self {
        Self {
            mode: ServicePolicyMode::Enforcing,
            prohibited_services: vec![
                "telnet.service".into(),
                "rsh.service".into(),
                "rlogin.service".into(),
                "rexec.service".into(),
                "tftp.service".into(),
                "xinetd.service".into(),
                "ypserv.service".into(),
                "ypbind.service".into(),
            ],
            prohibited_exec_paths: vec![
                "/tmp".into(),
                "/var/tmp".into(),
                "/dev/shm".into(),
                "/run/user".into(),
            ],
            disallow_root: false,
            allowed_root_services: vec![
                "systemd-journald.service".into(),
                "aios-securityd.service".into(),
            ],
            require_service_user: false,
            disallow_env_vars: vec![
                "LD_PRELOAD".into(),
                "LD_LIBRARY_PATH".into(),
                "IFS".into(),
            ],
            allowed_service_types: vec![
                ServiceType::Simple,
                ServiceType::Exec,
                ServiceType::Forking,
                ServiceType::Oneshot,
                ServiceType::Notify,
                ServiceType::Idle,
            ],
            max_env_vars: 256,
            max_timeout_secs: 3600,
        }
    }
}

/// A specific security violation found during service policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServicePolicyViolation {
    pub rule_id: String,
    pub service_name: String,
    pub description: String,
    pub fatal: bool,
}

/// Complete report of policy evaluation against a service spec or store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServicePolicyVerdict {
    pub service_name: String,
    pub allowed: bool,
    pub mode: ServicePolicyMode,
    pub violations: Vec<ServicePolicyViolation>,
    pub evaluated_at: String,
}

impl ServiceSecurityPolicy {
    /// Validates policy parameters and bounds (SP1).
    pub fn validate(&self) -> Result<(), String> {
        if self.prohibited_services.len() > 1024 {
            return Err("invariant SP1 violated: prohibited_services exceeds limit of 1024".into());
        }
        for svc in &self.prohibited_services {
            if svc.is_empty() || svc.len() > 128 || svc.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(format!("invariant SP1 violated: invalid prohibited service identifier '{}'", svc));
            }
        }

        if self.prohibited_exec_paths.len() > 128 {
            return Err("invariant SP1 violated: prohibited_exec_paths exceeds limit of 128".into());
        }
        for path in &self.prohibited_exec_paths {
            if path.is_empty() || path.len() > 1024 || path.chars().any(|c| c.is_control()) {
                return Err(format!("invariant SP1 violated: invalid prohibited exec path '{}'", path));
            }
            let is_unix_abs = path.starts_with('/');
            let is_win_abs = path.len() >= 3
                && path.as_bytes()[0].is_ascii_alphabetic()
                && path.as_bytes()[1] == b':'
                && (path.as_bytes()[2] == b'\\' || path.as_bytes()[2] == b'/');
            if !is_unix_abs && !is_win_abs {
                return Err(format!("invariant SP1 violated: prohibited_exec_paths must be absolute paths: '{}'", path));
            }
        }

        if self.disallow_env_vars.len() > 128 {
            return Err("invariant SP1 violated: disallow_env_vars exceeds limit of 128".into());
        }
        for var in &self.disallow_env_vars {
            if var.is_empty() || var.len() > 256 || var.contains('=') || var.chars().any(|c| c.is_control()) {
                return Err(format!("invariant SP1 violated: invalid disallowed environment variable '{}'", var));
            }
        }

        if self.allowed_service_types.is_empty() {
            return Err("invariant SP1 violated: allowed_service_types cannot be empty".into());
        }

        if self.allowed_root_services.len() > 256 {
            return Err("invariant SP1 violated: allowed_root_services exceeds limit of 256".into());
        }
        for svc in &self.allowed_root_services {
            if svc.is_empty() || svc.len() > 128 || svc.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(format!("invariant SP1 violated: invalid allowed_root_services entry '{}'", svc));
            }
        }

        if self.max_env_vars < 1 || self.max_env_vars > 1024 {
            return Err(format!(
                "invariant SP1 violated: max_env_vars out of range [1, 1024] (was {})",
                self.max_env_vars
            ));
        }

        if self.max_timeout_secs < 1 || self.max_timeout_secs > 86400 {
            return Err(format!(
                "invariant SP1 violated: max_timeout_secs out of range [1, 86400] (was {})",
                self.max_timeout_secs
            ));
        }

        Ok(())
    }

    /// Evaluates a single service specification against this security policy.
    pub fn evaluate_spec(&self, spec: &ServiceSpec) -> ServicePolicyVerdict {
        let mut violations = Vec::new();

        // Control character check
        if spec.name.chars().any(|c| c.is_control()) {
            violations.push(ServicePolicyViolation {
                rule_id: "SP1-MALFORMED-NAME".into(),
                service_name: spec.name.clone(),
                description: "Service name contains prohibited control characters".into(),
                fatal: true,
            });
        }

        // SP2: Prohibited services check
        let name_lower = spec.name.to_lowercase();
        let name_no_suffix = name_lower.strip_suffix(".service").unwrap_or(&name_lower);

        for prohibited in &self.prohibited_services {
            let p_lower = prohibited.to_lowercase();
            let p_no_suffix = p_lower.strip_suffix(".service").unwrap_or(&p_lower);

            if name_lower == p_lower || name_no_suffix == p_no_suffix {
                violations.push(ServicePolicyViolation {
                    rule_id: "SP2-PROHIBITED-SERVICE".into(),
                    service_name: spec.name.clone(),
                    description: format!("Service '{}' is prohibited by security policy", spec.name),
                    fatal: true,
                });
                break;
            }
        }

        // Helper to check execution commands
        let check_command = |cmd: &str, field_name: &str, viols: &mut Vec<ServicePolicyViolation>| {
            let binary = cmd.split_whitespace().next().unwrap_or("");
            if binary.is_empty() {
                return;
            }

            let is_unix_abs = binary.starts_with('/');
            let is_win_abs = binary.len() >= 3
                && binary.as_bytes()[0].is_ascii_alphabetic()
                && binary.as_bytes()[1] == b':'
                && (binary.as_bytes()[2] == b'\\' || binary.as_bytes()[2] == b'/');

            if !is_unix_abs && !is_win_abs {
                viols.push(ServicePolicyViolation {
                    rule_id: "SP3-RELATIVE-PATH".into(),
                    service_name: spec.name.clone(),
                    description: format!(
                        "Field '{}' binary '{}' is not an absolute path",
                        field_name, binary
                    ),
                    fatal: true,
                });
            }

            if cmd.contains("..") {
                viols.push(ServicePolicyViolation {
                    rule_id: "SP3-PATH-TRAVERSAL".into(),
                    service_name: spec.name.clone(),
                    description: format!(
                        "Field '{}' contains prohibited directory traversal '..'",
                        field_name
                    ),
                    fatal: true,
                });
            }

            for prohibited_prefix in &self.prohibited_exec_paths {
                if binary.starts_with(prohibited_prefix) {
                    viols.push(ServicePolicyViolation {
                        rule_id: "SP3-PROHIBITED-PATH".into(),
                        service_name: spec.name.clone(),
                        description: format!(
                            "Field '{}' binary '{}' resides in prohibited directory '{}'",
                            field_name, binary, prohibited_prefix
                        ),
                        fatal: true,
                    });
                    break;
                }
            }
        };

        // SP3: Path hygiene on exec_start, exec_stop, exec_reload, and working_dir
        check_command(&spec.exec_start, "exec_start", &mut violations);
        if let Some(ref stop) = spec.exec_stop {
            check_command(stop, "exec_stop", &mut violations);
        }
        if let Some(ref reload) = spec.exec_reload {
            check_command(reload, "exec_reload", &mut violations);
        }

        if let Some(ref dir) = spec.working_dir {
            if dir.contains("..") {
                violations.push(ServicePolicyViolation {
                    rule_id: "SP3-PATH-TRAVERSAL".into(),
                    service_name: spec.name.clone(),
                    description: format!("working_dir '{}' contains directory traversal '..'", dir),
                    fatal: true,
                });
            }
            for prohibited_prefix in &self.prohibited_exec_paths {
                if dir.starts_with(prohibited_prefix) {
                    violations.push(ServicePolicyViolation {
                        rule_id: "SP3-PROHIBITED-PATH".into(),
                        service_name: spec.name.clone(),
                        description: format!("working_dir '{}' resides in prohibited directory '{}'", dir, prohibited_prefix),
                        fatal: true,
                    });
                    break;
                }
            }
        }

        // SP4: User privilege & root checks
        let is_allowed_root = self
            .allowed_root_services
            .iter()
            .any(|s| s.eq_ignore_ascii_case(&spec.name));

        if self.require_service_user {
            let has_user = spec.user.as_ref().map(|u| !u.trim().is_empty()).unwrap_or(false);
            if !has_user && !is_allowed_root {
                violations.push(ServicePolicyViolation {
                    rule_id: "SP4-UNPRIVILEGED-USER-REQUIRED".into(),
                    service_name: spec.name.clone(),
                    description: format!("Service '{}' requires an unprivileged user to be configured", spec.name),
                    fatal: true,
                });
            }
        }

        if self.disallow_root {
            let is_root = match spec.user {
                Some(ref u) => u == "root" || u == "0",
                None => true, // default execution user in Unix init is root
            };
            if is_root && !is_allowed_root {
                violations.push(ServicePolicyViolation {
                    rule_id: "SP4-ROOT-DISALLOWED".into(),
                    service_name: spec.name.clone(),
                    description: format!("Service '{}' is configured to execute as root which is disallowed", spec.name),
                    fatal: true,
                });
            }
        }

        // SP5: Environment & timeout & service type sanitization
        for (k, _) in &spec.environment {
            if self.disallow_env_vars.iter().any(|d| d.eq_ignore_ascii_case(k)) {
                violations.push(ServicePolicyViolation {
                    rule_id: "SP5-DANGEROUS-ENV-VAR".into(),
                    service_name: spec.name.clone(),
                    description: format!("Service '{}' sets prohibited environment variable '{}'", spec.name, k),
                    fatal: true,
                });
            }
        }

        if spec.environment.len() > self.max_env_vars {
            violations.push(ServicePolicyViolation {
                rule_id: "SP5-ENV-COUNT-EXCEEDED".into(),
                service_name: spec.name.clone(),
                description: format!(
                    "Service '{}' has {} environment variables exceeding maximum of {}",
                    spec.name,
                    spec.environment.len(),
                    self.max_env_vars
                ),
                fatal: false,
            });
        }

        if spec.timeout_start_secs > self.max_timeout_secs {
            violations.push(ServicePolicyViolation {
                rule_id: "SP5-TIMEOUT-EXCEEDED".into(),
                service_name: spec.name.clone(),
                description: format!(
                    "Service '{}' timeout_start_secs ({}s) exceeds maximum allowed ({}s)",
                    spec.name, spec.timeout_start_secs, self.max_timeout_secs
                ),
                fatal: false,
            });
        }

        if spec.timeout_stop_secs > self.max_timeout_secs {
            violations.push(ServicePolicyViolation {
                rule_id: "SP5-TIMEOUT-EXCEEDED".into(),
                service_name: spec.name.clone(),
                description: format!(
                    "Service '{}' timeout_stop_secs ({}s) exceeds maximum allowed ({}s)",
                    spec.name, spec.timeout_stop_secs, self.max_timeout_secs
                ),
                fatal: false,
            });
        }

        if !self.allowed_service_types.contains(&spec.service_type) {
            violations.push(ServicePolicyViolation {
                rule_id: "SP5-DISALLOWED-TYPE".into(),
                service_name: spec.name.clone(),
                description: format!(
                    "Service '{}' service_type '{:?}' is not in allowed types",
                    spec.name, spec.service_type
                ),
                fatal: true,
            });
        }

        // SP6: Tri-state mode evaluation
        let allowed = match self.mode {
            ServicePolicyMode::Enforcing => !violations.iter().any(|v| v.fatal),
            ServicePolicyMode::Audit => true,
            ServicePolicyMode::Permissive => !violations.iter().any(|v| v.rule_id == "SP2-PROHIBITED-SERVICE"),
        };

        ServicePolicyVerdict {
            service_name: spec.name.clone(),
            allowed,
            mode: self.mode,
            violations,
            evaluated_at: "2026-09-06T00:00:00Z".into(),
        }
    }

    /// Evaluates all services in the store against this security policy.
    pub fn evaluate_store(&self, store: &ServiceStore) -> Vec<ServicePolicyVerdict> {
        store
            .list_services()
            .into_iter()
            .map(|svc| self.evaluate_spec(svc))
            .collect()
    }

    /// Loads policy from JSON configuration file with size cap.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();
        if path_str.len() > 1024 || path_str.chars().any(|c| c.is_control()) {
            return Err("policy file path exceeds 1024 characters or contains control characters".into());
        }
        let file = File::open(path).map_err(|e| format!("failed to open policy file '{}': {}", path.display(), e))?;
        let metadata = file.metadata().map_err(|e| format!("failed to query metadata for '{}': {}", path.display(), e))?;

        if metadata.len() > MAX_POLICY_FILE_BYTES {
            return Err(format!(
                "policy file '{}' size ({} bytes) exceeds maximum allowable ({} bytes)",
                path.display(), metadata.len(), MAX_POLICY_FILE_BYTES
            ));
        }

        let mut reader = file.take(MAX_POLICY_FILE_BYTES + 1);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e| format!("failed to read policy file '{}': {}", path.display(), e))?;

        let policy: Self = serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse policy JSON in '{}': {}", path.display(), e))?;
        policy.validate()?;
        Ok(policy)
    }

    /// Helper for environment variable reading.
    pub fn from_source<F: Fn(&str) -> Option<String>>(lookup: F) -> Result<Self, String> {
        let mut policy = Self::default();

        if let Some(val) = lookup("AIOS_SERVICE_POLICY_MODE") {
            policy.mode = match val.to_lowercase().as_str() {
                "enforcing" => ServicePolicyMode::Enforcing,
                "audit" => ServicePolicyMode::Audit,
                "permissive" => ServicePolicyMode::Permissive,
                other => return Err(format!("unknown AIOS_SERVICE_POLICY_MODE '{}'", other)),
            };
        }

        if let Some(val) = lookup("AIOS_SERVICE_DISALLOW_ROOT") {
            policy.disallow_root = val == "1" || val.eq_ignore_ascii_case("true");
        }

        if let Some(val) = lookup("AIOS_SERVICE_REQUIRE_USER") {
            policy.require_service_user = val == "1" || val.eq_ignore_ascii_case("true");
        }

        if let Some(val) = lookup("AIOS_SERVICE_MAX_TIMEOUT_SECS") {
            policy.max_timeout_secs = val.parse::<u64>().map_err(|e| format!("invalid AIOS_SERVICE_MAX_TIMEOUT_SECS: {}", e))?;
        }

        policy.validate()?;
        Ok(policy)
    }

    /// Loads policy from environment variables.
    pub fn from_env() -> Result<Self, String> {
        Self::from_source(|k| std::env::var(k).ok())
    }

    /// Resolves policy following precedence: file > env > default.
    pub fn resolve(custom_path: Option<&str>) -> Result<Self, String> {
        if let Some(path) = custom_path {
            return Self::from_file(path);
        }
        if let Ok(env_policy) = Self::from_env() {
            return Ok(env_policy);
        }
        let default_policy = Self::default();
        default_policy.validate()?;
        Ok(default_policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::*;
    use std::collections::BTreeMap;

    fn sample_valid_spec(name: &str) -> ServiceSpec {
        let mut env = BTreeMap::new();
        env.insert("AIOS_ENV".into(), "production".into());

        ServiceSpec {
            name: name.into(),
            description: "Test Supervised Service".into(),
            exec_start: "/usr/bin/aios-agent --run".into(),
            exec_stop: Some("/usr/bin/aios-agent --stop".into()),
            exec_reload: None,
            service_type: ServiceType::Simple,
            restart_policy: ServiceRestartPolicy::Always,
            startup_mode: ServiceStartupMode::Enabled,
            user: Some("aios".into()),
            group: Some("aios".into()),
            working_dir: Some("/var/lib/aios".into()),
            environment: env,
            dependencies: vec![],
            timeout_start_secs: 30,
            timeout_stop_secs: 30,
        }
    }

    #[test]
    fn test_policy_default_validation() {
        let policy = ServiceSecurityPolicy::default();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.mode, ServicePolicyMode::Enforcing);
    }

    #[test]
    fn test_policy_prohibited_service_rejection() {
        let policy = ServiceSecurityPolicy::default();
        let telnet_spec = sample_valid_spec("telnet.service");
        let verdict = policy.evaluate_spec(&telnet_spec);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "SP2-PROHIBITED-SERVICE"));

        let telnet_bare = sample_valid_spec("telnet");
        let verdict_bare = policy.evaluate_spec(&telnet_bare);
        assert!(!verdict_bare.allowed);
        assert!(verdict_bare.violations.iter().any(|v| v.rule_id == "SP2-PROHIBITED-SERVICE"));
    }

    #[test]
    fn test_policy_prohibited_path_and_traversal() {
        let policy = ServiceSecurityPolicy::default();
        let mut bad_spec = sample_valid_spec("my-service.service");
        bad_spec.exec_start = "/tmp/bad-daemon --start".into();
        let verdict = policy.evaluate_spec(&bad_spec);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "SP3-PROHIBITED-PATH"));

        let mut traversal_spec = sample_valid_spec("traversal-service.service");
        traversal_spec.exec_start = "/usr/bin/../bin/daemon".into();
        let verdict_traversal = policy.evaluate_spec(&traversal_spec);
        assert!(!verdict_traversal.allowed);
        assert!(verdict_traversal.violations.iter().any(|v| v.rule_id == "SP3-PATH-TRAVERSAL"));
    }

    #[test]
    fn test_policy_dangerous_env_vars() {
        let policy = ServiceSecurityPolicy::default();
        let mut env_spec = sample_valid_spec("env-service.service");
        env_spec.environment.insert("LD_PRELOAD".into(), "/lib/libevil.so".into());
        let verdict = policy.evaluate_spec(&env_spec);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "SP5-DANGEROUS-ENV-VAR"));
    }

    #[test]
    fn test_policy_root_disallowed_rule() {
        let mut policy = ServiceSecurityPolicy::default();
        policy.disallow_root = true;

        let mut root_spec = sample_valid_spec("root-service.service");
        root_spec.user = Some("root".into());
        let verdict = policy.evaluate_spec(&root_spec);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "SP4-ROOT-DISALLOWED"));

        // Allowed root service exemption
        let mut journald_spec = sample_valid_spec("systemd-journald.service");
        journald_spec.user = Some("root".into());
        let verdict_journald = policy.evaluate_spec(&journald_spec);
        assert!(verdict_journald.allowed);
    }

    #[test]
    fn test_policy_audit_mode() {
        let mut policy = ServiceSecurityPolicy::default();
        policy.mode = ServicePolicyMode::Audit;
        let telnet_spec = sample_valid_spec("telnet.service");
        let verdict = policy.evaluate_spec(&telnet_spec);
        // Under audit mode, allowed is true but violations are present
        assert!(verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "SP2-PROHIBITED-SERVICE"));
    }
}
