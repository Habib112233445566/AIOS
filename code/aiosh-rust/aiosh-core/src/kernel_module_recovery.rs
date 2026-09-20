//! Health check, validation, and corruption recovery for Kernel Module Management (KR1..KR6).
//!
//! Provides automated non-destructive self-healing, timestamped quarantine of damaged
//! stores, and deep validation reports.

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::kernel_module::{
    validate_module_name, validate_parameter, ModprobeRule,
};
use crate::kernel_module_service::KernelModuleStore;
use chrono::Utc;

/// Validation report detailing the integrity of a kernel module store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelModuleValidationReport {
    pub store_path: String,
    pub total_rules: usize,
    pub valid_rules: usize,
    pub invalid_rules: usize,
    pub total_autoload: usize,
    pub valid_autoload: usize,
    pub invalid_autoload: usize,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub recovered: bool,
    pub backup_path: Option<String>,
    pub evaluated_at: String,
}

impl KernelModuleValidationReport {
    /// Validates report internal consistency (KR1..KR3).
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.valid_rules + self.invalid_rules != self.total_rules {
            return Err(format!(
                "KR1 violated: valid_rules ({}) + invalid_rules ({}) != total_rules ({})",
                self.valid_rules, self.invalid_rules, self.total_rules
            ));
        }

        if self.valid_autoload + self.invalid_autoload != self.total_autoload {
            return Err(format!(
                "KR2 violated: valid_autoload ({}) + invalid_autoload ({}) != total_autoload ({})",
                self.valid_autoload, self.invalid_autoload, self.total_autoload
            ));
        }

        let expected_healthy = self.errors.is_empty()
            && self.invalid_rules == 0
            && self.invalid_autoload == 0;
        if self.healthy != expected_healthy {
            return Err(format!(
                "KR3 violated: healthy ({}) != expected_healthy ({})",
                self.healthy, expected_healthy
            ));
        }

        Ok(())
    }
}

/// Validates an in-memory KernelModuleStore against syntax, safety, and conflict invariants.
pub fn validate_kernel_module_store(store: &KernelModuleStore, store_path: &Path) -> KernelModuleValidationReport {
    let mut errors = Vec::new();
    let mut valid_rules = 0;
    let mut invalid_rules = 0;
    let mut valid_autoload = 0;
    let mut invalid_autoload = 0;

    let config = &store.config;

    if config.id.trim().is_empty() {
        errors.push("store id cannot be empty".into());
    }

    // 1. Validate rules
    for (i, rule) in config.rules.iter().enumerate() {
        let mut rule_errs = Vec::new();
        match rule {
            ModprobeRule::Blacklist { module } => {
                if let Err(e) = validate_module_name(module) {
                    rule_errs.push(format!("rule #{}: invalid blacklist module name: {}", i, e));
                }
            }
            ModprobeRule::Alias { alias, module } => {
                if alias.trim().is_empty() {
                    rule_errs.push(format!("rule #{}: alias cannot be empty", i));
                }
                if let Err(e) = validate_module_name(module) {
                    rule_errs.push(format!("rule #{}: invalid alias target module name: {}", i, e));
                }
            }
            ModprobeRule::Options { module, options } => {
                if let Err(e) = validate_module_name(module) {
                    rule_errs.push(format!("rule #{}: invalid options module name: {}", i, e));
                }
                for opt in options {
                    if let Some((k, v)) = opt.split_once('=') {
                        if let Err(e) = validate_parameter(k, v) {
                            rule_errs.push(format!("rule #{}: invalid parameter {}: {}", i, opt, e));
                        }
                    } else if let Err(e) = validate_module_name(opt) {
                        rule_errs.push(format!("rule #{}: invalid option flag {}: {}", i, opt, e));
                    }
                }
            }
            ModprobeRule::Install { module, command } => {
                if let Err(e) = validate_module_name(module) {
                    rule_errs.push(format!("rule #{}: invalid install module name: {}", i, e));
                }
                if command.trim().is_empty() {
                    rule_errs.push(format!("rule #{}: install command cannot be empty", i));
                }
            }
            ModprobeRule::Remove { module, command } => {
                if let Err(e) = validate_module_name(module) {
                    rule_errs.push(format!("rule #{}: invalid remove module name: {}", i, e));
                }
                if command.trim().is_empty() {
                    rule_errs.push(format!("rule #{}: remove command cannot be empty", i));
                }
            }
            ModprobeRule::Softdep { module, pre, post } => {
                if let Err(e) = validate_module_name(module) {
                    rule_errs.push(format!("rule #{}: invalid softdep module name: {}", i, e));
                }
                for p in pre {
                    if let Err(e) = validate_module_name(p) {
                        rule_errs.push(format!("rule #{}: invalid softdep pre-module name {}: {}", i, p, e));
                    }
                }
                for p in post {
                    if let Err(e) = validate_module_name(p) {
                        rule_errs.push(format!("rule #{}: invalid softdep post-module name {}: {}", i, p, e));
                    }
                }
            }
        }

        if rule_errs.is_empty() {
            valid_rules += 1;
        } else {
            invalid_rules += 1;
            errors.extend(rule_errs);
        }
    }

    // 2. Validate autoload modules and conflicts (KM3 / KR4)
    for (i, autol) in config.autoload_modules.iter().enumerate() {
        let mut autol_errs = Vec::new();
        if let Err(e) = validate_module_name(autol) {
            autol_errs.push(format!("autoload #{}: invalid module name: {}", i, e));
        }

        let is_blacklisted = config.rules.iter().any(|r| match r {
            ModprobeRule::Blacklist { module } => module.eq_ignore_ascii_case(autol),
            ModprobeRule::Install { module, command } => {
                module.eq_ignore_ascii_case(autol) && (command.contains("/bin/true") || command.contains("/bin/false"))
            }
            _ => false,
        });

        if is_blacklisted {
            autol_errs.push(format!(
                "autoload #{}: module '{}' is both autoloaded and blacklisted/disabled (KM3 conflict)",
                i, autol
            ));
        }

        if autol_errs.is_empty() {
            valid_autoload += 1;
        } else {
            invalid_autoload += 1;
            errors.extend(autol_errs);
        }
    }

    let total_rules = config.rules.len();
    let total_autoload = config.autoload_modules.len();
    let healthy = errors.is_empty() && invalid_rules == 0 && invalid_autoload == 0;

    KernelModuleValidationReport {
        store_path: store_path.to_string_lossy().to_string(),
        total_rules,
        valid_rules,
        invalid_rules,
        total_autoload,
        valid_autoload,
        invalid_autoload,
        errors,
        healthy,
        recovered: false,
        backup_path: None,
        evaluated_at: Utc::now().to_rfc3339(),
    }
}

/// Checks an existing store file on disk in read-only mode.
pub fn check_store_file(path: &Path) -> Result<KernelModuleValidationReport, String> {
    if !path.exists() {
        return Ok(KernelModuleValidationReport {
            store_path: path.to_string_lossy().to_string(),
            total_rules: 0,
            valid_rules: 0,
            invalid_rules: 0,
            total_autoload: 0,
            valid_autoload: 0,
            invalid_autoload: 0,
            errors: vec![format!("store file {:?} does not exist", path)],
            healthy: false,
            recovered: false,
            backup_path: None,
            evaluated_at: Utc::now().to_rfc3339(),
        });
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return Ok(KernelModuleValidationReport {
                store_path: path.to_string_lossy().to_string(),
                total_rules: 0,
                valid_rules: 0,
                invalid_rules: 0,
                total_autoload: 0,
                valid_autoload: 0,
                invalid_autoload: 0,
                errors: vec![format!("store read failure: {}", e)],
                healthy: false,
                recovered: false,
                backup_path: None,
                evaluated_at: Utc::now().to_rfc3339(),
            });
        }
    };

    match serde_json::from_str::<KernelModuleStore>(&content) {
        Ok(store) => Ok(validate_kernel_module_store(&store, path)),
        Err(e) => Ok(KernelModuleValidationReport {
            store_path: path.to_string_lossy().to_string(),
            total_rules: 0,
            valid_rules: 0,
            invalid_rules: 0,
            total_autoload: 0,
            valid_autoload: 0,
            invalid_autoload: 0,
            errors: vec![format!("store parse failure: {}", e)],
            healthy: false,
            recovered: false,
            backup_path: None,
            evaluated_at: Utc::now().to_rfc3339(),
        }),
    }
}

/// Recovers a damaged or invalid store file non-destructively (KR4..KR6).
pub fn recover_store_file(path: &Path) -> Result<KernelModuleValidationReport, String> {
    if !path.exists() {
        let store = KernelModuleStore::new("default", "recovered default kernel module store");
        store.save_to_path(path)?;
        let mut rep = validate_kernel_module_store(&store, path);
        rep.recovered = true;
        return Ok(rep);
    }

    // Try reading content
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_e) => {
            // Quarantine unreadable file and recreate default
            let backup_path = create_timestamped_backup(path)?;
            let store = KernelModuleStore::new("default", "recovered default kernel module store");
            store.save_to_path(path)?;
            let mut rep = validate_kernel_module_store(&store, path);
            rep.recovered = true;
            rep.backup_path = Some(backup_path);
            return Ok(rep);
        }
    };

    // Try parsing store JSON
    let parse_res: Result<KernelModuleStore, _> = serde_json::from_str(&content);
    match parse_res {
        Ok(mut store) => {
            let initial_rep = validate_kernel_module_store(&store, path);
            if initial_rep.healthy {
                return Ok(initial_rep);
            }

            // Create non-destructive backup before repairing
            let backup_path = create_timestamped_backup(path)?;

            // Sanitize rules: keep only valid rules
            let mut sanitized_rules = Vec::new();
            for rule in store.config.rules {
                let is_valid = match &rule {
                    ModprobeRule::Blacklist { module } => validate_module_name(module).is_ok(),
                    ModprobeRule::Alias { alias, module } => !alias.trim().is_empty() && validate_module_name(module).is_ok(),
                    ModprobeRule::Options { module, options } => {
                        validate_module_name(module).is_ok() && options.iter().all(|opt| {
                            if let Some((k, v)) = opt.split_once('=') {
                                validate_parameter(k, v).is_ok()
                            } else {
                                validate_module_name(opt).is_ok()
                            }
                        })
                    }
                    ModprobeRule::Install { module, command } => validate_module_name(module).is_ok() && !command.trim().is_empty(),
                    ModprobeRule::Remove { module, command } => validate_module_name(module).is_ok() && !command.trim().is_empty(),
                    ModprobeRule::Softdep { module, pre, post } => {
                        validate_module_name(module).is_ok()
                            && pre.iter().all(|p| validate_module_name(p).is_ok())
                            && post.iter().all(|p| validate_module_name(p).is_ok())
                    }
                };
                if is_valid && !sanitized_rules.contains(&rule) {
                    sanitized_rules.push(rule);
                }
            }

            // Sanitize autoload: drop invalid and blacklisted/disabled modules
            let mut sanitized_autoload = Vec::new();
            for autol in store.config.autoload_modules {
                if validate_module_name(&autol).is_err() {
                    continue;
                }
                let is_blacklisted = sanitized_rules.iter().any(|r| match r {
                    ModprobeRule::Blacklist { module } => module.eq_ignore_ascii_case(&autol),
                    ModprobeRule::Install { module, command } => {
                        module.eq_ignore_ascii_case(&autol) && (command.contains("/bin/true") || command.contains("/bin/false"))
                    }
                    _ => false,
                });
                if !is_blacklisted && !sanitized_autoload.contains(&autol) {
                    sanitized_autoload.push(autol);
                }
            }

            store.config.rules = sanitized_rules;
            store.config.autoload_modules = sanitized_autoload;
            if store.config.id.trim().is_empty() {
                store.config.id = "repaired".into();
            }

            // Persist sanitized store atomically
            store.save_to_path(path)?;

            let mut final_rep = validate_kernel_module_store(&store, path);
            final_rep.recovered = true;
            final_rep.backup_path = Some(backup_path);
            Ok(final_rep)
        }
        Err(_) => {
            // Unparseable JSON: quarantine and recreate clean default
            let backup_path = create_timestamped_backup(path)?;
            let store = KernelModuleStore::new("default", "recovered default kernel module store");
            store.save_to_path(path)?;
            let mut rep = validate_kernel_module_store(&store, path);
            rep.recovered = true;
            rep.backup_path = Some(backup_path);
            Ok(rep)
        }
    }
}

/// Creates a timestamped quarantine backup of the given path (KR5).
fn create_timestamped_backup(path: &Path) -> Result<String, String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("store");
    let ts = Utc::now().format("%Y%m%d_%H%M%S_%6f").to_string();
    let backup_name = format!("{}.corrupt.{}.bak", file_name, ts);
    let backup_path = parent.join(&backup_name);

    fs::copy(path, &backup_path)
        .map_err(|e| format!("failed to create backup at {:?}: {}", backup_path, e))?;

    Ok(backup_path.to_string_lossy().to_string())
}
