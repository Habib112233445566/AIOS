//! Init & Service Supervision Data Model (SS1..SS5)
//!
//! Provides canonical data structures, state machines, and validation logic for
//! AIOS service supervision across supported Linux distribution targets (systemd, OpenRC).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// Service execution architecture type (aligned with systemd Service Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    Simple,
    Exec,
    Forking,
    Oneshot,
    Notify,
    Idle,
}

/// Runtime lifecycle state of a managed service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Active,
    Inactive,
    Activating,
    Deactivating,
    Failed,
    Reloading,
    Unknown,
}

/// Restart policy for the service process upon termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRestartPolicy {
    No,
    Always,
    OnSuccess,
    OnFailure,
    OnAbnormal,
    OnWatchdog,
    OnAbort,
}

/// Startup enablement mode (aligned with systemd unit enablement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStartupMode {
    Enabled,
    Disabled,
    Masked,
    Static,
}

/// Relationship type between services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceDependencyType {
    Requires,
    Wants,
    After,
    Before,
    Conflicts,
}

/// Directed dependency on another service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub name: String,
    pub dependency_type: ServiceDependencyType,
    pub optional: bool,
}

/// Health and process accounting telemetry for a service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub healthy: bool,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub restarts: u32,
    pub last_error: Option<String>,
}

/// Canonical specification of a managed service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    pub description: String,
    pub exec_start: String,
    pub exec_stop: Option<String>,
    pub exec_reload: Option<String>,
    pub service_type: ServiceType,
    pub restart_policy: ServiceRestartPolicy,
    pub startup_mode: ServiceStartupMode,
    pub user: Option<String>,
    pub group: Option<String>,
    pub working_dir: Option<String>,
    pub environment: BTreeMap<String, String>,
    pub dependencies: Vec<ServiceDependency>,
    pub timeout_start_secs: u64,
    pub timeout_stop_secs: u64,
}

/// Runtime status snapshot of a service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,
    pub startup_mode: ServiceStartupMode,
    pub pid: Option<u32>,
    pub health: ServiceHealth,
    pub started_at: Option<String>,
}

/// Administrative action performed on a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
    Mask,
    Unmask,
}

/// Query filter for listing and discovering services.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceQuery {
    pub name_pattern: Option<String>,
    pub state: Option<ServiceState>,
    pub startup_mode: Option<ServiceStartupMode>,
    pub limit: Option<usize>,
}

/// Validates service identifier syntax against upstream standards (SS1).
pub fn validate_service_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("service name cannot be empty".into());
    }
    if name.len() > 128 {
        return Err(format!(
            "service name exceeds 128 characters (was {})",
            name.len()
        ));
    }

    let bytes = name.as_bytes();
    let first = bytes[0];
    if !first.is_ascii_alphanumeric() {
        return Err(format!(
            "service name must start with an alphanumeric character: '{}'",
            name
        ));
    }

    for &b in bytes {
        let is_valid = b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.';
        if !is_valid {
            return Err(format!(
                "service name contains invalid character '{}' in '{}'",
                b as char, name
            ));
        }
    }

    Ok(())
}

/// Validates complete service specification against formal invariants (SS1..SS5).
pub fn validate_service_spec(spec: &ServiceSpec) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // SS1: Service naming syntax
    if let Err(err) = validate_service_name(&spec.name) {
        errors.push(err);
    }

    // SS2: Execution command & path validation
    if spec.exec_start.is_empty() {
        errors.push("exec_start cannot be empty".into());
    } else if spec.exec_start.len() > 4096 {
        errors.push(format!(
            "exec_start exceeds 4096 characters (was {})",
            spec.exec_start.len()
        ));
    }

    if let Some(ref stop) = spec.exec_stop {
        if stop.is_empty() {
            errors.push("exec_stop cannot be empty string when specified".into());
        } else if stop.len() > 4096 {
            errors.push(format!(
                "exec_stop exceeds 4096 characters (was {})",
                stop.len()
            ));
        }
    }

    if let Some(ref reload) = spec.exec_reload {
        if reload.is_empty() {
            errors.push("exec_reload cannot be empty string when specified".into());
        } else if reload.len() > 4096 {
            errors.push(format!(
                "exec_reload exceeds 4096 characters (was {})",
                reload.len()
            ));
        }
    }

    if let Some(ref dir) = spec.working_dir {
        if dir.is_empty() {
            errors.push("working_dir cannot be empty string when specified".into());
        } else if dir.len() > 1024 {
            errors.push(format!(
                "working_dir exceeds 1024 characters (was {})",
                dir.len()
            ));
        } else {
            // Must be absolute path (starts with '/' on Unix or drive letter '[A-Za-z]:\' on Windows)
            let is_unix_abs = dir.starts_with('/');
            let is_win_abs = dir.len() >= 3
                && dir.as_bytes()[0].is_ascii_alphabetic()
                && dir.as_bytes()[1] == b':'
                && (dir.as_bytes()[2] == b'\\' || dir.as_bytes()[2] == b'/');
            if !is_unix_abs && !is_win_abs {
                errors.push(format!(
                    "working_dir must be an absolute path: '{}'",
                    dir
                ));
            }
            if dir.contains("..") {
                errors.push(format!(
                    "working_dir contains path traversal sequence '..': '{}'",
                    dir
                ));
            }
        }
    }

    // SS3: Dependency hygiene & acyclicity
    if spec.dependencies.len() > 128 {
        errors.push(format!(
            "dependencies list exceeds 128 items (was {})",
            spec.dependencies.len()
        ));
    }

    let mut seen_deps = HashSet::new();
    for dep in &spec.dependencies {
        if dep.name == spec.name {
            errors.push(format!("service cannot depend on itself: '{}'", dep.name));
        }
        if !seen_deps.insert(&dep.name) {
            errors.push(format!("duplicate dependency detected: '{}'", dep.name));
        }
        if let Err(err) = validate_service_name(&dep.name) {
            errors.push(format!("invalid dependency name: {}", err));
        }
    }

    // SS4: Resource & field limits
    if spec.description.len() > 4096 {
        errors.push(format!(
            "description exceeds 4096 characters (was {})",
            spec.description.len()
        ));
    }

    if spec.timeout_start_secs == 0 || spec.timeout_start_secs > 86400 {
        errors.push(format!(
            "timeout_start_secs out of range [1, 86400] (was {})",
            spec.timeout_start_secs
        ));
    }

    if spec.timeout_stop_secs == 0 || spec.timeout_stop_secs > 86400 {
        errors.push(format!(
            "timeout_stop_secs out of range [1, 86400] (was {})",
            spec.timeout_stop_secs
        ));
    }

    if spec.environment.len() > 256 {
        errors.push(format!(
            "environment variables map exceeds 256 entries (was {})",
            spec.environment.len()
        ));
    }

    for (k, v) in &spec.environment {
        if k.is_empty() {
            errors.push("environment variable key cannot be empty".into());
        } else if k.len() > 256 {
            errors.push(format!(
                "environment variable key exceeds 256 characters (was {})",
                k.len()
            ));
        } else if k.contains('=') || k.contains('\0') {
            errors.push(format!(
                "environment variable key contains illegal characters: '{}'",
                k
            ));
        }

        if v.len() > 4096 {
            errors.push(format!(
                "environment variable value exceeds 4096 characters for key '{}'",
                k
            ));
        }
    }

    if let Some(ref u) = spec.user {
        if u.is_empty() || u.len() > 64 {
            errors.push(format!("user name length out of range [1, 64] (was {})", u.len()));
        } else if !u.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            errors.push(format!("user name contains invalid characters: '{}'", u));
        }
    }

    if let Some(ref g) = spec.group {
        if g.is_empty() || g.len() > 64 {
            errors.push(format!("group name length out of range [1, 64] (was {})", g.len()));
        } else if !g.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            errors.push(format!("group name contains invalid characters: '{}'", g));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates runtime service status consistency against formal invariants (SS5).
pub fn validate_service_status(status: &ServiceStatus) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Err(err) = validate_service_name(&status.name) {
        errors.push(err);
    }

    // SS5: State & mode consistency
    if status.state == ServiceState::Failed && status.health.healthy {
        errors.push("service state is 'failed' but health reports healthy == true".into());
    }

    if status.startup_mode == ServiceStartupMode::Masked && status.state == ServiceState::Active {
        errors.push("service is masked but reports state == 'active'".into());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_valid_spec() -> ServiceSpec {
        let mut env = BTreeMap::new();
        env.insert("AIOS_ENV".to_string(), "production".to_string());
        env.insert("LOG_LEVEL".to_string(), "info".to_string());

        ServiceSpec {
            name: "aios-securityd.service".to_string(),
            description: "AIOS Security Daemon".to_string(),
            exec_start: "/usr/bin/aios-securityd --daemon".to_string(),
            exec_stop: Some("/usr/bin/aios-securityd --stop".to_string()),
            exec_reload: Some("/usr/bin/aios-securityd --reload".to_string()),
            service_type: ServiceType::Simple,
            restart_policy: ServiceRestartPolicy::Always,
            startup_mode: ServiceStartupMode::Enabled,
            user: Some("aios".to_string()),
            group: Some("aios".to_string()),
            working_dir: Some("/var/lib/aios".to_string()),
            environment: env,
            dependencies: vec![ServiceDependency {
                name: "auditd.service".to_string(),
                dependency_type: ServiceDependencyType::Requires,
                optional: false,
            }],
            timeout_start_secs: 30,
            timeout_stop_secs: 30,
        }
    }

    #[test]
    fn test_valid_service_spec_happy_path() {
        let spec = sample_valid_spec();
        assert!(validate_service_spec(&spec).is_ok());
    }

    #[test]
    fn test_service_naming_syntax_ss1() {
        assert!(validate_service_name("aios-securityd").is_ok());
        assert!(validate_service_name("aios-securityd.service").is_ok());
        assert!(validate_service_name("dbus").is_ok());
        assert!(validate_service_name("systemd-journald.socket").is_ok());

        // Negative cases
        assert!(validate_service_name("").is_err());
        assert!(validate_service_name(" aios").is_err());
        assert!(validate_service_name("aios;rm").is_err());
        assert!(validate_service_name("aios/securityd").is_err());
        assert!(validate_service_name("aios\\securityd").is_err());
        assert!(validate_service_name(".hidden").is_err());
        assert!(validate_service_name("-invalid").is_err());

        let long_name = "a".repeat(129);
        assert!(validate_service_name(&long_name).is_err());
    }

    #[test]
    fn test_exec_commands_and_working_dir_ss2() {
        let mut spec = sample_valid_spec();
        spec.exec_start = String::new();
        let errs = validate_service_spec(&spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("exec_start cannot be empty")));

        let mut spec = sample_valid_spec();
        spec.working_dir = Some("relative/path".to_string());
        let errs = validate_service_spec(&spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("must be an absolute path")));

        let mut spec = sample_valid_spec();
        spec.working_dir = Some("/var/lib/../etc".to_string());
        let errs = validate_service_spec(&spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("path traversal sequence '..'")));
    }

    #[test]
    fn test_dependency_hygiene_ss3() {
        let mut spec = sample_valid_spec();
        spec.dependencies.push(ServiceDependency {
            name: "aios-securityd.service".to_string(),
            dependency_type: ServiceDependencyType::Requires,
            optional: false,
        });
        let errs = validate_service_spec(&spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("service cannot depend on itself")));

        let mut spec = sample_valid_spec();
        spec.dependencies.push(ServiceDependency {
            name: "auditd.service".to_string(),
            dependency_type: ServiceDependencyType::Wants,
            optional: true,
        });
        let errs = validate_service_spec(&spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("duplicate dependency detected")));
    }

    #[test]
    fn test_resource_bounds_ss4() {
        let mut spec = sample_valid_spec();
        spec.timeout_start_secs = 0;
        let errs = validate_service_spec(&spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("timeout_start_secs out of range")));

        let mut spec = sample_valid_spec();
        spec.timeout_stop_secs = 90000;
        let errs = validate_service_spec(&spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("timeout_stop_secs out of range")));

        let mut spec = sample_valid_spec();
        spec.environment.insert("INVALID=KEY".to_string(), "val".to_string());
        let errs = validate_service_spec(&spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("contains illegal characters")));
    }

    #[test]
    fn test_status_consistency_ss5() {
        let status = ServiceStatus {
            name: "aios-securityd.service".to_string(),
            state: ServiceState::Active,
            startup_mode: ServiceStartupMode::Enabled,
            pid: Some(1234),
            health: ServiceHealth {
                healthy: true,
                exit_code: None,
                pid: Some(1234),
                uptime_seconds: Some(3600),
                restarts: 0,
                last_error: None,
            },
            started_at: Some("2026-09-05T00:00:00Z".to_string()),
        };
        assert!(validate_service_status(&status).is_ok());

        // Failed state with healthy == true should error
        let mut bad_status = status.clone();
        bad_status.state = ServiceState::Failed;
        bad_status.health.healthy = true;
        let errs = validate_service_status(&bad_status).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("service state is 'failed' but health reports healthy")));

        // Masked service active should error
        let mut masked_status = status.clone();
        masked_status.startup_mode = ServiceStartupMode::Masked;
        masked_status.state = ServiceState::Active;
        let errs = validate_service_status(&masked_status).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("service is masked but reports state == 'active'")));
    }
}
