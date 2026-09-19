//! Observability and telemetry reports for AIOS Kernel Module Management Subsystem (KO1..KO6).

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::kernel_module::{ModprobeRule, ModuleState};
use crate::kernel_module_service::KernelModuleService;
use crate::kernel_module_policy::KernelModuleSecurityPolicy;

/// Canonical string representation for kernel module states.
pub fn module_state_to_str(state: ModuleState) -> &'static str {
    match state {
        ModuleState::Live => "live",
        ModuleState::Loading => "loading",
        ModuleState::Unloading => "unloading",
        ModuleState::Unloaded => "unloaded",
    }
}

/// Canonical string representation for modprobe directive rule types.
pub fn rule_type_to_str(rule: &ModprobeRule) -> &'static str {
    match rule {
        ModprobeRule::Blacklist { .. } => "blacklist",
        ModprobeRule::Alias { .. } => "alias",
        ModprobeRule::Options { .. } => "options",
        ModprobeRule::Install { .. } => "install",
        ModprobeRule::Remove { .. } => "remove",
        ModprobeRule::Softdep { .. } => "softdep",
    }
}

/// Comprehensive observability report detailing loaded modules, store rules, and policy compliance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelModuleObservabilityReport {
    pub total_loaded_modules: usize,
    pub total_memory_bytes: u64,
    pub state_breakdown: BTreeMap<String, usize>,
    pub store_rules_count: usize,
    pub rule_type_breakdown: BTreeMap<String, usize>,
    pub autoload_modules_count: usize,
    pub ref_count_distribution: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_modules_configured: Vec<String>,
    pub protected_modules_configured: Vec<String>,
    pub generated_at: String,
}

impl KernelModuleObservabilityReport {
    /// Generates an observability report from the provided service and optional security policy (KO1..KO6).
    pub fn generate(
        service: &KernelModuleService,
        policy_opt: Option<&KernelModuleSecurityPolicy>,
    ) -> Self {
        let mut total_loaded_modules = 0;
        let mut total_memory_bytes: u64 = 0;
        let mut state_breakdown = BTreeMap::new();
        let mut ref_count_distribution = BTreeMap::new();

        // Initialize ref_count buckets
        ref_count_distribution.insert("0".into(), 0);
        ref_count_distribution.insert("1-2".into(), 0);
        ref_count_distribution.insert("3-5".into(), 0);
        ref_count_distribution.insert("6+".into(), 0);

        // 1. Process live loaded modules
        if let Ok(loaded) = service.list_loaded_modules() {
            total_loaded_modules = loaded.len();
            for m in &loaded {
                total_memory_bytes = total_memory_bytes.saturating_add(m.size_bytes);
                let state_str = module_state_to_str(m.state);
                *state_breakdown.entry(state_str.to_string()).or_insert(0) += 1;

                let bucket = if m.ref_count == 0 {
                    "0"
                } else if m.ref_count <= 2 {
                    "1-2"
                } else if m.ref_count <= 5 {
                    "3-5"
                } else {
                    "6+"
                };
                *ref_count_distribution.entry(bucket.to_string()).or_insert(0) += 1;
            }
        }

        // 2. Process store rules and autoload modules
        let mut rule_type_breakdown = BTreeMap::new();
        let store_rules = &service.store.config.rules;
        let store_rules_count = store_rules.len();

        for rule in store_rules {
            let rtype = rule_type_to_str(rule);
            *rule_type_breakdown.entry(rtype.to_string()).or_insert(0) += 1;
        }

        let autoload_modules = &service.store.config.autoload_modules;
        let autoload_modules_count = autoload_modules.len();

        // 3. Process policy evaluation
        let default_policy = KernelModuleSecurityPolicy::default();
        let policy = policy_opt.unwrap_or(&default_policy);

        let mut policy_compliant_count = 0;
        let mut policy_violations_count = 0;
        let mut prohibited_configured_set = BTreeSet::new();
        let mut protected_configured_set = BTreeSet::new();

        for rule in store_rules {
            let verdict = policy.evaluate_rule(rule);
            if verdict.allowed {
                policy_compliant_count += 1;
            }
            policy_violations_count += verdict.violations.len();

            if verdict.violations.iter().any(|v| v.rule_id == "SP-KM2-PROHIBITED-MODULE") {
                prohibited_configured_set.insert(verdict.module_name.clone());
            }

            if policy.protected_modules.iter().any(|p| p.eq_ignore_ascii_case(&verdict.module_name)) {
                protected_configured_set.insert(verdict.module_name.clone());
            }
        }

        for autol in autoload_modules {
            let verdict = policy.evaluate_autoload(autol);
            if verdict.allowed {
                policy_compliant_count += 1;
            }
            policy_violations_count += verdict.violations.len();

            if verdict.violations.iter().any(|v| v.rule_id == "SP-KM2-PROHIBITED-AUTOLOAD") {
                prohibited_configured_set.insert(autol.clone());
            }

            if policy.protected_modules.iter().any(|p| p.eq_ignore_ascii_case(autol)) {
                protected_configured_set.insert(autol.clone());
            }
        }

        Self {
            total_loaded_modules,
            total_memory_bytes,
            state_breakdown,
            store_rules_count,
            rule_type_breakdown,
            autoload_modules_count,
            ref_count_distribution,
            policy_compliant_count,
            policy_violations_count,
            prohibited_modules_configured: prohibited_configured_set.into_iter().collect(),
            protected_modules_configured: protected_configured_set.into_iter().collect(),
            generated_at: "2026-09-19T00:00:00Z".into(),
        }
    }
}
