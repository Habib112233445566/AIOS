//! Observability and telemetry reports for AIOS Init & Service Supervision subsystem (SO1..SO6).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::service::{ServiceRestartPolicy, ServiceStartupMode, ServiceState, ServiceType};
use crate::service_policy::ServiceSecurityPolicy;
use crate::service_service::ServiceStore;

/// Canonical string representation for service execution architecture types.
pub fn service_type_to_str(st: ServiceType) -> &'static str {
    match st {
        ServiceType::Simple => "simple",
        ServiceType::Exec => "exec",
        ServiceType::Forking => "forking",
        ServiceType::Oneshot => "oneshot",
        ServiceType::Notify => "notify",
        ServiceType::Idle => "idle",
    }
}

/// Canonical string representation for service runtime states.
pub fn service_state_to_str(state: ServiceState) -> &'static str {
    match state {
        ServiceState::Active => "active",
        ServiceState::Inactive => "inactive",
        ServiceState::Activating => "activating",
        ServiceState::Deactivating => "deactivating",
        ServiceState::Failed => "failed",
        ServiceState::Reloading => "reloading",
        ServiceState::Unknown => "unknown",
    }
}

/// Canonical string representation for service startup enablement modes.
pub fn startup_mode_to_str(mode: ServiceStartupMode) -> &'static str {
    match mode {
        ServiceStartupMode::Enabled => "enabled",
        ServiceStartupMode::Disabled => "disabled",
        ServiceStartupMode::Masked => "masked",
        ServiceStartupMode::Static => "static",
    }
}

/// Canonical string representation for service process restart policies.
pub fn restart_policy_to_str(policy: ServiceRestartPolicy) -> &'static str {
    match policy {
        ServiceRestartPolicy::No => "no",
        ServiceRestartPolicy::Always => "always",
        ServiceRestartPolicy::OnSuccess => "on_success",
        ServiceRestartPolicy::OnFailure => "on_failure",
        ServiceRestartPolicy::OnAbnormal => "on_abnormal",
        ServiceRestartPolicy::OnWatchdog => "on_watchdog",
        ServiceRestartPolicy::OnAbort => "on_abort",
    }
}

/// Comprehensive observability report detailing supervised service inventory, states, health, and policy compliance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceObservabilityReport {
    pub total_services: usize,
    pub state_breakdown: BTreeMap<String, usize>,
    pub startup_mode_breakdown: BTreeMap<String, usize>,
    pub service_type_breakdown: BTreeMap<String, usize>,
    pub restart_policy_breakdown: BTreeMap<String, usize>,
    pub healthy_count: usize,
    pub unhealthy_count: usize,
    pub total_restarts: u32,
    pub failed_services: Vec<String>,
    pub dependency_distribution: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_services_found: Vec<String>,
    pub generated_at: String,
}

impl ServiceObservabilityReport {
    /// Generates an observability report from the provided ServiceStore and optional security policy (SO1..SO6).
    pub fn generate(
        store: &ServiceStore,
        policy_opt: Option<&ServiceSecurityPolicy>,
    ) -> Self {
        let services = store.list_services();
        let total_services = services.len();

        let mut state_breakdown = BTreeMap::new();
        let mut startup_mode_breakdown = BTreeMap::new();
        let mut service_type_breakdown = BTreeMap::new();
        let mut restart_policy_breakdown = BTreeMap::new();

        let mut dependency_distribution = BTreeMap::new();
        dependency_distribution.insert("0".into(), 0);
        dependency_distribution.insert("1-2".into(), 0);
        dependency_distribution.insert("3-5".into(), 0);
        dependency_distribution.insert("6+".into(), 0);

        let mut healthy_count = 0;
        let mut unhealthy_count = 0;
        let mut total_restarts: u32 = 0;
        let mut failed_services_set = BTreeSet::new();

        let default_policy = ServiceSecurityPolicy::default();
        let policy = policy_opt.unwrap_or(&default_policy);

        let mut policy_compliant_count = 0;
        let mut policy_violations_count = 0;
        let mut prohibited_found_set = BTreeSet::new();

        for svc in &services {
            // Retrieve current runtime status if available
            let (state_str, mode_str) = match store.get_status(&svc.name) {
                Some(status) => {
                    let s_str = service_state_to_str(status.state).to_string();
                    let m_str = startup_mode_to_str(status.startup_mode).to_string();

                    if status.health.healthy && status.state != ServiceState::Failed {
                        healthy_count += 1;
                    } else {
                        unhealthy_count += 1;
                        failed_services_set.insert(svc.name.clone());
                    }
                    total_restarts = total_restarts.saturating_add(status.health.restarts);

                    (s_str, m_str)
                }
                None => {
                    // Fallback to specification startup mode and inactive state
                    let s_str = service_state_to_str(ServiceState::Inactive).to_string();
                    let m_str = startup_mode_to_str(svc.startup_mode).to_string();
                    healthy_count += 1;
                    (s_str, m_str)
                }
            };

            // SO2: Categorical distributions
            *state_breakdown.entry(state_str).or_insert(0) += 1;
            *startup_mode_breakdown.entry(mode_str).or_insert(0) += 1;

            let type_str = service_type_to_str(svc.service_type).to_string();
            *service_type_breakdown.entry(type_str).or_insert(0) += 1;

            let policy_str = restart_policy_to_str(svc.restart_policy).to_string();
            *restart_policy_breakdown.entry(policy_str).or_insert(0) += 1;

            // SO4: Dependency complexity distribution
            let dep_count = svc.dependencies.len();
            let bucket = if dep_count == 0 {
                "0"
            } else if dep_count <= 2 {
                "1-2"
            } else if dep_count <= 5 {
                "3-5"
            } else {
                "6+"
            };
            *dependency_distribution.entry(bucket.to_string()).or_insert(0) += 1;

            // SO5: Security policy evaluation
            let verdict = policy.evaluate_spec(svc);
            if verdict.allowed {
                policy_compliant_count += 1;
            }
            policy_violations_count += verdict.violations.len();

            if verdict.violations.iter().any(|v| v.rule_id == "SP2-PROHIBITED-SERVICE") {
                prohibited_found_set.insert(svc.name.clone());
            }
        }

        // SO6: Deterministic report
        Self {
            total_services,
            state_breakdown,
            startup_mode_breakdown,
            service_type_breakdown,
            restart_policy_breakdown,
            healthy_count,
            unhealthy_count,
            total_restarts,
            failed_services: failed_services_set.into_iter().collect(),
            dependency_distribution,
            policy_compliant_count,
            policy_violations_count,
            prohibited_services_found: prohibited_found_set.into_iter().collect(),
            generated_at: "2026-09-06T00:00:00Z".into(),
        }
    }

    /// Serializes the report to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("failed to serialize observability report: {}", e))
    }

    /// Generates an observability report from optional store and policy file paths with strict input validation.
    pub fn generate_from_paths<P: AsRef<Path>, Q: AsRef<Path>>(
        store_path_opt: Option<P>,
        policy_path_opt: Option<Q>,
    ) -> Result<Self, String> {
        if let Some(ref sp) = store_path_opt {
            let sp_str = sp.as_ref().to_string_lossy();
            if sp_str.len() > 1024 || sp_str.chars().any(|c| c.is_control()) {
                return Err("store path exceeds 1024 characters or contains control characters".into());
            }
        }
        if let Some(ref pp) = policy_path_opt {
            let pp_str = pp.as_ref().to_string_lossy();
            if pp_str.len() > 1024 || pp_str.chars().any(|c| c.is_control()) {
                return Err("policy path exceeds 1024 characters or contains control characters".into());
            }
        }
        let store = match store_path_opt {
            Some(ref sp) => ServiceStore::load_from_path(sp.as_ref())?,
            None => ServiceStore::new(),
        };
        let policy = match policy_path_opt {
            Some(ref pp) => Some(ServiceSecurityPolicy::from_file(pp.as_ref())?),
            None => None,
        };
        Ok(Self::generate(&store, policy.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::service::*;

    fn sample_spec(name: &str) -> ServiceSpec {
        let mut env = BTreeMap::new();
        env.insert("AIOS_ENV".into(), "test".into());

        ServiceSpec {
            name: name.into(),
            description: format!("Service {}", name),
            exec_start: "/usr/bin/aios-test --daemon".into(),
            exec_stop: None,
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
    fn test_observability_empty_store() {
        let store = ServiceStore::empty();
        let report = ServiceObservabilityReport::generate(&store, None);

        assert_eq!(report.total_services, 0);
        assert!(report.state_breakdown.is_empty());
        assert!(report.startup_mode_breakdown.is_empty());
        assert!(report.service_type_breakdown.is_empty());
        assert!(report.restart_policy_breakdown.is_empty());
        assert_eq!(report.healthy_count, 0);
        assert_eq!(report.unhealthy_count, 0);
        assert_eq!(report.total_restarts, 0);
        assert!(report.failed_services.is_empty());
        assert_eq!(report.dependency_distribution.get("0"), Some(&0));
        assert_eq!(report.policy_compliant_count, 0);
        assert_eq!(report.policy_violations_count, 0);
        assert!(report.prohibited_services_found.is_empty());
    }

    #[test]
    fn test_observability_default_store() {
        let store = ServiceStore::new();
        let report = ServiceObservabilityReport::generate(&store, None);

        // SO1: Inventory completeness
        assert!(report.total_services > 0);
        let sum_states: usize = report.state_breakdown.values().sum();
        let sum_modes: usize = report.startup_mode_breakdown.values().sum();
        let sum_types: usize = report.service_type_breakdown.values().sum();
        let sum_policies: usize = report.restart_policy_breakdown.values().sum();

        assert_eq!(sum_states, report.total_services);
        assert_eq!(sum_modes, report.total_services);
        assert_eq!(sum_types, report.total_services);
        assert_eq!(sum_policies, report.total_services);

        // SO4: Dependency completeness
        let sum_deps: usize = report.dependency_distribution.values().sum();
        assert_eq!(sum_deps, report.total_services);

        // SO6: JSON serialization
        assert!(report.to_json_pretty().is_ok());
    }

    #[test]
    fn test_observability_health_and_policy() {
        let mut store = ServiceStore::empty();
        let mut s1 = sample_spec("telnet.service");
        s1.dependencies.push(ServiceDependency {
            name: "other.service".into(),
            dependency_type: ServiceDependencyType::Wants,
            optional: true,
        });
        store.register_service(s1).unwrap();

        let report = ServiceObservabilityReport::generate(&store, None);
        assert_eq!(report.total_services, 1);
        assert_eq!(report.policy_compliant_count, 0);
        assert!(report.policy_violations_count > 0);
        assert_eq!(report.prohibited_services_found, vec!["telnet.service".to_string()]);
        assert_eq!(report.dependency_distribution.get("1-2"), Some(&1));
    }
}
