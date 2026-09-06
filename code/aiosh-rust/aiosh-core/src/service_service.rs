//! Init & Service Supervision Core Service & Store (CS1..CS5)
//!
//! Provides the in-memory ServiceStore, query engine, lifecycle action execution,
//! dependency order planning, and atomic persistence mechanisms for system services.

use crate::service::{
    validate_service_spec, validate_service_status, ServiceAction, ServiceDependency,
    ServiceDependencyType, ServiceHealth, ServiceQuery, ServiceRestartPolicy, ServiceSpec,
    ServiceStartupMode, ServiceState, ServiceStatus, ServiceType,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

/// Report detailing the outcome of a service management action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceActionReport {
    pub service_name: String,
    pub action: ServiceAction,
    pub previous_state: ServiceState,
    pub new_state: ServiceState,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: String,
}

/// In-memory repository of managed system service specifications and runtime statuses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStore {
    pub services: BTreeMap<String, ServiceSpec>,
    pub statuses: BTreeMap<String, ServiceStatus>,
}

impl Default for ServiceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceStore {
    /// Initializes a store pre-seeded with canonical reference services.
    pub fn new() -> Self {
        let mut store = Self::empty();

        let canonical_services = vec![
            ServiceSpec {
                name: "auditd.service".into(),
                description: "Security Audit Logging Daemon".into(),
                exec_start: "/sbin/auditd -s".into(),
                exec_stop: None,
                exec_reload: None,
                service_type: ServiceType::Forking,
                restart_policy: ServiceRestartPolicy::OnFailure,
                startup_mode: ServiceStartupMode::Enabled,
                user: Some("root".into()),
                group: Some("root".into()),
                working_dir: Some("/var/log/audit".into()),
                environment: BTreeMap::new(),
                dependencies: vec![],
                timeout_start_secs: 30,
                timeout_stop_secs: 30,
            },
            ServiceSpec {
                name: "dbus.service".into(),
                description: "D-Bus System Message Bus".into(),
                exec_start: "/usr/bin/dbus-daemon --system --nofork".into(),
                exec_stop: None,
                exec_reload: Some("/usr/bin/dbus-send --system --type=method_call --dest=org.freedesktop.DBus / org.freedesktop.DBus.ReloadConfig".into()),
                service_type: ServiceType::Simple,
                restart_policy: ServiceRestartPolicy::Always,
                startup_mode: ServiceStartupMode::Enabled,
                user: Some("messagebus".into()),
                group: Some("messagebus".into()),
                working_dir: Some("/var/run/dbus".into()),
                environment: BTreeMap::new(),
                dependencies: vec![],
                timeout_start_secs: 25,
                timeout_stop_secs: 25,
            },
            ServiceSpec {
                name: "systemd-journald.service".into(),
                description: "Journal Service".into(),
                exec_start: "/lib/systemd/systemd-journald".into(),
                exec_stop: None,
                exec_reload: None,
                service_type: ServiceType::Notify,
                restart_policy: ServiceRestartPolicy::Always,
                startup_mode: ServiceStartupMode::Static,
                user: Some("root".into()),
                group: Some("root".into()),
                working_dir: Some("/var/log/journal".into()),
                environment: BTreeMap::new(),
                dependencies: vec![],
                timeout_start_secs: 30,
                timeout_stop_secs: 30,
            },
            ServiceSpec {
                name: "network-manager.service".into(),
                description: "Network Management Service".into(),
                exec_start: "/usr/sbin/NetworkManager --no-daemon".into(),
                exec_stop: None,
                exec_reload: Some("/usr/bin/nmcli general reload".into()),
                service_type: ServiceType::Simple,
                restart_policy: ServiceRestartPolicy::OnFailure,
                startup_mode: ServiceStartupMode::Enabled,
                user: Some("root".into()),
                group: Some("root".into()),
                working_dir: Some("/var/lib/NetworkManager".into()),
                environment: BTreeMap::new(),
                dependencies: vec![ServiceDependency {
                    name: "dbus.service".into(),
                    dependency_type: ServiceDependencyType::Requires,
                    optional: false,
                }],
                timeout_start_secs: 45,
                timeout_stop_secs: 30,
            },
            ServiceSpec {
                name: "aios-securityd.service".into(),
                description: "AIOS Security Daemon supervising platform PEP and audit ring".into(),
                exec_start: "/usr/bin/aios-securityd --daemon --config /etc/aios/security.json".into(),
                exec_stop: Some("/usr/bin/aios-securityd --stop".into()),
                exec_reload: Some("/usr/bin/aios-securityd --reload".into()),
                service_type: ServiceType::Simple,
                restart_policy: ServiceRestartPolicy::Always,
                startup_mode: ServiceStartupMode::Enabled,
                user: Some("aios".into()),
                group: Some("aios".into()),
                working_dir: Some("/var/lib/aios".into()),
                environment: BTreeMap::new(),
                dependencies: vec![
                    ServiceDependency {
                        name: "auditd.service".into(),
                        dependency_type: ServiceDependencyType::Requires,
                        optional: false,
                    },
                    ServiceDependency {
                        name: "dbus.service".into(),
                        dependency_type: ServiceDependencyType::Requires,
                        optional: false,
                    },
                ],
                timeout_start_secs: 30,
                timeout_stop_secs: 30,
            },
            ServiceSpec {
                name: "ssh.service".into(),
                description: "OpenBSD Secure Shell server".into(),
                exec_start: "/usr/sbin/sshd -D".into(),
                exec_stop: None,
                exec_reload: Some("/bin/kill -HUP $MAINPID".into()),
                service_type: ServiceType::Simple,
                restart_policy: ServiceRestartPolicy::OnFailure,
                startup_mode: ServiceStartupMode::Enabled,
                user: Some("root".into()),
                group: Some("root".into()),
                working_dir: Some("/var/run/sshd".into()),
                environment: BTreeMap::new(),
                dependencies: vec![ServiceDependency {
                    name: "network-manager.service".into(),
                    dependency_type: ServiceDependencyType::After,
                    optional: true,
                }],
                timeout_start_secs: 30,
                timeout_stop_secs: 30,
            },
        ];

        for (i, spec) in canonical_services.into_iter().enumerate() {
            let pid = Some(200 + i as u32);
            let status = ServiceStatus {
                name: spec.name.clone(),
                state: ServiceState::Active,
                startup_mode: spec.startup_mode,
                pid,
                health: ServiceHealth {
                    healthy: true,
                    exit_code: None,
                    pid,
                    uptime_seconds: Some(3600),
                    restarts: 0,
                    last_error: None,
                },
                started_at: Some("2026-09-06T00:00:00Z".into()),
            };
            store.statuses.insert(spec.name.clone(), status);
            store.services.insert(spec.name.clone(), spec);
        }

        store
    }

    /// Initializes an empty store.
    pub fn empty() -> Self {
        Self {
            services: BTreeMap::new(),
            statuses: BTreeMap::new(),
        }
    }

    /// List all service specifications sorted by name.
    pub fn list_services(&self) -> Vec<&ServiceSpec> {
        self.services.values().collect()
    }

    /// Retrieve service specification by name.
    pub fn get_service(&self, name: &str) -> Option<&ServiceSpec> {
        self.services.get(name)
    }

    /// Retrieve runtime service status by name.
    pub fn get_status(&self, name: &str) -> Option<&ServiceStatus> {
        self.statuses.get(name)
    }

    /// Register a new service specification into the store (CS1).
    pub fn register_service(&mut self, spec: ServiceSpec) -> Result<(), String> {
        validate_service_spec(&spec).map_err(|errs| errs.join("; "))?;
        if self.services.contains_key(&spec.name) {
            return Err(format!(
                "invariant CS1 violated: service '{}' is already registered",
                spec.name
            ));
        }

        let initial_status = ServiceStatus {
            name: spec.name.clone(),
            state: ServiceState::Inactive,
            startup_mode: spec.startup_mode,
            pid: None,
            health: ServiceHealth {
                healthy: true,
                exit_code: None,
                pid: None,
                uptime_seconds: None,
                restarts: 0,
                last_error: None,
            },
            started_at: None,
        };

        self.statuses.insert(spec.name.clone(), initial_status);
        self.services.insert(spec.name.clone(), spec);
        Ok(())
    }

    /// Unregister an existing service by name.
    pub fn unregister_service(&mut self, name: &str) -> Result<ServiceSpec, String> {
        self.statuses.remove(name);
        self.services
            .remove(name)
            .ok_or_else(|| format!("service '{}' not found in store", name))
    }

    /// Query services matching filters in ServiceQuery.
    pub fn query(&self, query: &ServiceQuery) -> Vec<&ServiceSpec> {
        let mut results: Vec<&ServiceSpec> = self
            .services
            .values()
            .filter(|spec| {
                if let Some(ref pattern) = query.name_pattern {
                    let pat = pattern.to_lowercase();
                    if !spec.name.to_lowercase().contains(&pat)
                        && !spec.description.to_lowercase().contains(&pat)
                    {
                        return false;
                    }
                }
                if let Some(ref sm) = query.startup_mode {
                    if spec.startup_mode != *sm {
                        return false;
                    }
                }
                if let Some(ref state) = query.state {
                    match self.statuses.get(&spec.name) {
                        Some(status) if status.state == *state => {}
                        _ => return false,
                    }
                }
                true
            })
            .collect();

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }
        results
    }

    /// Execute a lifecycle action against a registered service (CS2).
    pub fn execute_action(
        &mut self,
        service_name: &str,
        action: ServiceAction,
    ) -> Result<ServiceActionReport, String> {
        let spec = self
            .services
            .get_mut(service_name)
            .ok_or_else(|| format!("service '{}' not found in store", service_name))?;
        let status = self
            .statuses
            .get_mut(service_name)
            .ok_or_else(|| format!("status for service '{}' not found in store", service_name))?;

        let previous_state = status.state;
        let ts = chrono::Utc::now().to_rfc3339();

        match action {
            ServiceAction::Start => {
                if spec.startup_mode == ServiceStartupMode::Masked {
                    return Err(format!(
                        "cannot start masked service '{}'; unmask unit first",
                        service_name
                    ));
                }
                status.state = ServiceState::Active;
                status.pid = status.pid.or(Some(1001));
                status.health.healthy = true;
                status.health.pid = status.pid;
                status.health.last_error = None;
                status.started_at = Some(ts.clone());

                Ok(ServiceActionReport {
                    service_name: service_name.to_string(),
                    action,
                    previous_state,
                    new_state: ServiceState::Active,
                    success: true,
                    error: None,
                    timestamp: ts,
                })
            }
            ServiceAction::Stop => {
                status.state = ServiceState::Inactive;
                status.pid = None;
                status.health.pid = None;
                status.health.uptime_seconds = None;
                status.started_at = None;

                Ok(ServiceActionReport {
                    service_name: service_name.to_string(),
                    action,
                    previous_state,
                    new_state: ServiceState::Inactive,
                    success: true,
                    error: None,
                    timestamp: ts,
                })
            }
            ServiceAction::Restart => {
                if spec.startup_mode == ServiceStartupMode::Masked {
                    return Err(format!(
                        "cannot restart masked service '{}'; unmask unit first",
                        service_name
                    ));
                }
                status.state = ServiceState::Active;
                status.pid = Some(1002);
                status.health.healthy = true;
                status.health.pid = Some(1002);
                status.health.restarts = status.health.restarts.saturating_add(1);
                status.health.last_error = None;
                status.started_at = Some(ts.clone());

                Ok(ServiceActionReport {
                    service_name: service_name.to_string(),
                    action,
                    previous_state,
                    new_state: ServiceState::Active,
                    success: true,
                    error: None,
                    timestamp: ts,
                })
            }
            ServiceAction::Reload => {
                if status.state != ServiceState::Active {
                    return Err(format!(
                        "cannot reload inactive service '{}'; service must be active",
                        service_name
                    ));
                }

                Ok(ServiceActionReport {
                    service_name: service_name.to_string(),
                    action,
                    previous_state,
                    new_state: ServiceState::Active,
                    success: true,
                    error: None,
                    timestamp: ts,
                })
            }
            ServiceAction::Enable => {
                if spec.startup_mode == ServiceStartupMode::Masked {
                    return Err(format!(
                        "cannot enable masked service '{}'; unmask unit first",
                        service_name
                    ));
                }
                spec.startup_mode = ServiceStartupMode::Enabled;
                status.startup_mode = ServiceStartupMode::Enabled;

                Ok(ServiceActionReport {
                    service_name: service_name.to_string(),
                    action,
                    previous_state,
                    new_state: status.state,
                    success: true,
                    error: None,
                    timestamp: ts,
                })
            }
            ServiceAction::Disable => {
                if spec.startup_mode == ServiceStartupMode::Masked {
                    return Err(format!(
                        "cannot disable masked service '{}'; unit is masked",
                        service_name
                    ));
                }
                spec.startup_mode = ServiceStartupMode::Disabled;
                status.startup_mode = ServiceStartupMode::Disabled;

                Ok(ServiceActionReport {
                    service_name: service_name.to_string(),
                    action,
                    previous_state,
                    new_state: status.state,
                    success: true,
                    error: None,
                    timestamp: ts,
                })
            }
            ServiceAction::Mask => {
                if status.state == ServiceState::Active || status.state == ServiceState::Activating
                {
                    return Err(format!(
                        "cannot mask active running service '{}'; stop unit first",
                        service_name
                    ));
                }
                spec.startup_mode = ServiceStartupMode::Masked;
                status.startup_mode = ServiceStartupMode::Masked;

                Ok(ServiceActionReport {
                    service_name: service_name.to_string(),
                    action,
                    previous_state,
                    new_state: status.state,
                    success: true,
                    error: None,
                    timestamp: ts,
                })
            }
            ServiceAction::Unmask => {
                if spec.startup_mode == ServiceStartupMode::Masked {
                    spec.startup_mode = ServiceStartupMode::Disabled;
                    status.startup_mode = ServiceStartupMode::Disabled;
                }

                Ok(ServiceActionReport {
                    service_name: service_name.to_string(),
                    action,
                    previous_state,
                    new_state: status.state,
                    success: true,
                    error: None,
                    timestamp: ts,
                })
            }
        }
    }

    /// Compute deterministic topological activation order for a service and its dependencies (CS3).
    pub fn plan_service_order(&self, target_service: &str) -> Result<Vec<String>, String> {
        if !self.services.contains_key(target_service) {
            return Err(format!("target service '{}' not found in store", target_service));
        }

        // Collect transitive closure of dependencies
        let mut closure = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(target_service.to_string());
        closure.insert(target_service.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(spec) = self.services.get(&current) {
                for dep in &spec.dependencies {
                    if dep.optional && !self.services.contains_key(&dep.name) {
                        continue;
                    }
                    if !self.services.contains_key(&dep.name) {
                        return Err(format!(
                            "unmet dependency: service '{}' requires '{}' which is not registered",
                            current, dep.name
                        ));
                    }
                    if closure.insert(dep.name.clone()) {
                        queue.push_back(dep.name.clone());
                    }
                }
            }
        }

        // Build adjacency graph: if A requires B or A is after B, then B must precede A (B -> A)
        let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
        let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for name in &closure {
            in_degree.insert(name.clone(), 0);
            adj.insert(name.clone(), BTreeSet::new());
        }

        for name in &closure {
            if let Some(spec) = self.services.get(name) {
                for dep in &spec.dependencies {
                    if closure.contains(&dep.name) {
                        // dep.name must start before `name`
                        if adj.get_mut(&dep.name).unwrap().insert(name.clone()) {
                            *in_degree.get_mut(name).unwrap() += 1;
                        }
                    }
                }
            }
        }

        // Kahn's algorithm
        let mut ready: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(k, _)| k.clone())
            .collect();

        let mut ordered = Vec::new();
        while let Some(node) = ready.pop_front() {
            ordered.push(node.clone());
            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        ready.push_back(neighbor.clone());
                    }
                }
            }
        }

        if ordered.len() != closure.len() {
            return Err("invariant CS3 violated: cyclic dependency detected in service graph".into());
        }

        Ok(ordered)
    }

    /// Atomically persist store to disk using temporary file rename (CS5).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self)
            .map_err(|e| format!("failed to serialize service store: {}", e))?;

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        if let Err(e) = std::fs::write(&tmp_path, content.as_bytes()) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("failed to write temp service store: {}", e));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o644));
        }

        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!(
                "failed to persist service store to '{}': {}",
                path.display(),
                e
            ));
        }

        Ok(())
    }

    /// Load and validate store from disk enforcing size ceilings (CS5).
    pub fn load_from_path(path: &Path) -> Result<ServiceStore, String> {
        if !path.exists() {
            return Err(format!("service store file '{}' does not exist", path.display()));
        }

        let meta = std::fs::metadata(path)
            .map_err(|e| format!("failed to read metadata for '{}': {}", path.display(), e))?;

        // 10 MiB ceiling
        if meta.len() > 10 * 1024 * 1024 {
            return Err(format!(
                "service store file '{}' exceeds 10 MiB ceiling (was {} bytes)",
                path.display(),
                meta.len()
            ));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read service store file '{}': {}", path.display(), e))?;

        let store: ServiceStore = serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse service store JSON: {}", e))?;

        // 10,000 entities ceiling
        if store.services.len() > 10_000 {
            return Err(format!(
                "service store exceeds 10,000 entities ceiling (was {})",
                store.services.len()
            ));
        }

        // Validate all specs
        for spec in store.services.values() {
            validate_service_spec(spec).map_err(|errs| {
                format!("service '{}' in store violates invariants: {}", spec.name, errs.join("; "))
            })?;
        }

        // Validate all statuses
        for status in store.statuses.values() {
            validate_service_status(status).map_err(|errs| {
                format!("status for '{}' in store violates invariants: {}", status.name, errs.join("; "))
            })?;
        }

        Ok(store)
    }
}
