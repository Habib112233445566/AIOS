//! Security policy enforcement for AIOS Kernel Module Management Subsystem (SP-KM1..SP-KM6).
//!
//! Provides validation of modprobe directives, kernel module options, and autoload
//! configurations against mandatory security criteria, CIS hardening baselines, and
//! system stability invariants.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::kernel_module::{
    validate_module_name, validate_parameter, ModprobeRule,
};
use crate::kernel_module_service::KernelModuleStore;

/// Maximum allowable size for a kernel module policy configuration file (64 KiB).
pub const MAX_POLICY_FILE_BYTES: u64 = 65_536;

/// Enforcement mode for kernel module security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelModulePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

impl Default for KernelModulePolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Security policy defining mandatory security criteria for kernel modules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelModuleSecurityPolicy {
    pub mode: KernelModulePolicyMode,
    pub prohibited_modules: Vec<String>,
    pub protected_modules: Vec<String>,
    pub allowed_install_commands: Vec<String>,
    pub disallowed_parameter_keys: Vec<String>,
    pub disallowed_parameter_patterns: Vec<String>,
    pub max_parameter_value_len: usize,
    pub max_rules_per_module: usize,
    pub enforce_strict_naming: bool,
}

impl Default for KernelModuleSecurityPolicy {
    fn default() -> Self {
        Self {
            mode: KernelModulePolicyMode::Enforcing,
            prohibited_modules: vec![
                "cramfs".into(),
                "freevxfs".into(),
                "jffs2".into(),
                "hfs".into(),
                "hfsplus".into(),
                "squashfs".into(),
                "udf".into(),
                "dccp".into(),
                "sctp".into(),
                "rds".into(),
                "tipc".into(),
                "firewire_core".into(),
                "thunderbolt".into(),
            ],
            protected_modules: vec![
                "ext4".into(),
                "xfs".into(),
                "btrfs".into(),
                "overlay".into(),
                "crypto".into(),
                "dm_mod".into(),
                "dm_crypt".into(),
                "vfat".into(),
            ],
            allowed_install_commands: vec![
                "/bin/true".into(),
                "/bin/false".into(),
                "/usr/bin/true".into(),
                "/usr/bin/false".into(),
            ],
            disallowed_parameter_keys: vec![
                "init".into(),
                "rdinit".into(),
                "panic".into(),
            ],
            disallowed_parameter_patterns: vec![
                ";".into(),
                "&".into(),
                "|".into(),
                "`".into(),
                "$".into(),
                "\n".into(),
                "\r".into(),
                "/bin/sh".into(),
                "/bin/bash".into(),
            ],
            max_parameter_value_len: 1024,
            max_rules_per_module: 64,
            enforce_strict_naming: true,
        }
    }
}

/// A specific security violation found during kernel module policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelModulePolicyViolation {
    pub rule_id: String,
    pub module_name: String,
    pub description: String,
    pub fatal: bool,
}

/// Complete report of policy evaluation against a module rule, autoload entry, or store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelModulePolicyVerdict {
    pub module_name: String,
    pub allowed: bool,
    pub mode: KernelModulePolicyMode,
    pub violations: Vec<KernelModulePolicyViolation>,
    pub evaluated_at: String,
}

impl KernelModuleSecurityPolicy {
    /// Validates policy parameters, limits, and disjointness (SP-KM1, SP-KM6).
    pub fn validate(&self) -> Result<(), String> {
        if self.prohibited_modules.len() > 1024 {
            return Err("invariant SP-KM1 violated: prohibited_modules exceeds limit of 1024".into());
        }
        if self.protected_modules.len() > 256 {
            return Err("invariant SP-KM1 violated: protected_modules exceeds limit of 256".into());
        }
        if self.allowed_install_commands.len() > 32 {
            return Err("invariant SP-KM1 violated: allowed_install_commands exceeds limit of 32".into());
        }
        if self.disallowed_parameter_keys.len() > 128 {
            return Err("invariant SP-KM1 violated: disallowed_parameter_keys exceeds limit of 128".into());
        }
        if self.max_parameter_value_len < 1 || self.max_parameter_value_len > 65536 {
            return Err(format!(
                "invariant SP-KM1 violated: max_parameter_value_len out of range [1, 65536] (was {})",
                self.max_parameter_value_len
            ));
        }
        if self.max_rules_per_module < 1 || self.max_rules_per_module > 1024 {
            return Err(format!(
                "invariant SP-KM1 violated: max_rules_per_module out of range [1, 1024] (was {})",
                self.max_rules_per_module
            ));
        }

        // Validate module names and disjointness
        for m in &self.prohibited_modules {
            validate_module_name(m)?;
        }
        for m in &self.protected_modules {
            validate_module_name(m)?;
            if self.prohibited_modules.iter().any(|p| p.eq_ignore_ascii_case(m)) {
                return Err(format!(
                    "invariant SP-KM1 violated: module '{}' is both prohibited and protected",
                    m
                ));
            }
        }

        // Validate allowed install commands
        for cmd in &self.allowed_install_commands {
            if cmd.is_empty() || cmd.len() > 256 || cmd.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(format!("invariant SP-KM1 violated: invalid allowed install command '{}'", cmd));
            }
            if !cmd.starts_with('/') && !cmd.starts_with("C:") && !cmd.starts_with("c:") {
                return Err(format!(
                    "invariant SP-KM1 violated: allowed install command must be absolute path: '{}'",
                    cmd
                ));
            }
            if cmd.contains("..") {
                return Err(format!(
                    "invariant SP-KM1 violated: allowed install command contains path traversal '..': '{}'",
                    cmd
                ));
            }
        }

        // Validate disallowed parameter keys
        for key in &self.disallowed_parameter_keys {
            validate_module_name(key)?;
        }

        Ok(())
    }

    /// Evaluates a single modprobe rule against this security policy.
    pub fn evaluate_rule(&self, rule: &ModprobeRule) -> KernelModulePolicyVerdict {
        let mut violations = Vec::new();
        let (mod_name, is_blacklist, is_options, _is_install) = match rule {
            ModprobeRule::Blacklist { module } => (module.clone(), true, false, false),
            ModprobeRule::Alias { module, .. } => (module.clone(), false, false, false),
            ModprobeRule::Options { module, .. } => (module.clone(), false, true, false),
            ModprobeRule::Install { module, .. } => (module.clone(), false, false, true),
            ModprobeRule::Remove { module, .. } => (module.clone(), false, false, false),
            ModprobeRule::Softdep { module, .. } => (module.clone(), false, false, false),
        };

        // SP-KM1: Validate module name
        if let Err(e) = validate_module_name(&mod_name) {
            violations.push(KernelModulePolicyViolation {
                rule_id: "SP-KM1-INVALID-NAME".into(),
                module_name: mod_name.clone(),
                description: format!("Module name syntax invalid: {}", e),
                fatal: true,
            });
        }

        // SP-KM2: Prohibited module enforcement
        let is_prohibited = self.prohibited_modules.iter().any(|p| p.eq_ignore_ascii_case(&mod_name));
        if is_prohibited {
            // Prohibited modules should ONLY be blacklisted or disabled; setting options or alias is a violation
            if is_options {
                violations.push(KernelModulePolicyViolation {
                    rule_id: "SP-KM2-PROHIBITED-MODULE".into(),
                    module_name: mod_name.clone(),
                    description: format!(
                        "Module '{}' is prohibited by security policy; configuring options is disallowed",
                        mod_name
                    ),
                    fatal: true,
                });
            }
        }

        // SP-KM3: Protected module guard
        let is_protected = self.protected_modules.iter().any(|p| p.eq_ignore_ascii_case(&mod_name));
        if is_protected {
            if is_blacklist {
                violations.push(KernelModulePolicyViolation {
                    rule_id: "SP-KM3-PROTECTED-MODULE".into(),
                    module_name: mod_name.clone(),
                    description: format!(
                        "Module '{}' is a protected system module and cannot be blacklisted",
                        mod_name
                    ),
                    fatal: true,
                });
            }
        }

        // Rule-specific checks
        match rule {
            ModprobeRule::Install { module, command } => {
                // Check if install command is disabling a protected module
                let is_disabling = command.contains("/bin/true")
                    || command.contains("/bin/false")
                    || command.contains("/usr/bin/true")
                    || command.contains("/usr/bin/false");
                if is_protected && is_disabling {
                    violations.push(KernelModulePolicyViolation {
                        rule_id: "SP-KM3-PROTECTED-MODULE".into(),
                        module_name: module.clone(),
                        description: format!(
                            "Module '{}' is protected and cannot be disabled via install command '{}'",
                            module, command
                        ),
                        fatal: true,
                    });
                }

                // SP-KM4: Install command sanitization
                let trimmed_cmd = command.trim();
                let binary = trimmed_cmd.split_whitespace().next().unwrap_or("");
                if !self.allowed_install_commands.iter().any(|c| c == binary) {
                    violations.push(KernelModulePolicyViolation {
                        rule_id: "SP-KM4-UNAPPROVED-INSTALL-CMD".into(),
                        module_name: module.clone(),
                        description: format!(
                            "Install command binary '{}' is not in allowed install commands",
                            binary
                        ),
                        fatal: true,
                    });
                }

                // Check for injection
                if trimmed_cmd.contains(';')
                    || trimmed_cmd.contains('&')
                    || trimmed_cmd.contains('|')
                    || trimmed_cmd.contains('`')
                    || trimmed_cmd.contains('$')
                    || trimmed_cmd.contains("..")
                {
                    violations.push(KernelModulePolicyViolation {
                        rule_id: "SP-KM4-INSTALL-COMMAND-INJECTION".into(),
                        module_name: module.clone(),
                        description: format!(
                            "Install command '{}' contains prohibited shell metacharacters or path traversal",
                            command
                        ),
                        fatal: true,
                    });
                }
            }
            ModprobeRule::Options { module, options } => {
                for opt in options {
                    if let Some((key, val)) = opt.split_once('=') {
                        // SP-KM5: Check parameter key
                        if self.disallowed_parameter_keys.iter().any(|k| k.eq_ignore_ascii_case(key)) {
                            violations.push(KernelModulePolicyViolation {
                                rule_id: "SP-KM5-DISALLOWED-PARAM-KEY".into(),
                                module_name: module.clone(),
                                description: format!(
                                    "Parameter key '{}' for module '{}' is disallowed by security policy",
                                    key, module
                                ),
                                fatal: true,
                            });
                        }

                        // Check value length
                        if val.len() > self.max_parameter_value_len {
                            violations.push(KernelModulePolicyViolation {
                                rule_id: "SP-KM5-PARAM-TOO-LONG".into(),
                                module_name: module.clone(),
                                description: format!(
                                    "Parameter '{}' value length ({} bytes) exceeds maximum allowable ({} bytes)",
                                    key, val.len(), self.max_parameter_value_len
                                ),
                                fatal: true,
                            });
                        }

                        // Check dangerous patterns
                        for pat in &self.disallowed_parameter_patterns {
                            if val.contains(pat) {
                                violations.push(KernelModulePolicyViolation {
                                    rule_id: "SP-KM5-DANGEROUS-PARAM-VALUE".into(),
                                    module_name: module.clone(),
                                    description: format!(
                                        "Parameter '{}' contains prohibited pattern '{}'",
                                        key, pat
                                    ),
                                    fatal: true,
                                });
                                break;
                            }
                        }

                        // Validate syntax
                        if let Err(e) = validate_parameter(key, val) {
                            violations.push(KernelModulePolicyViolation {
                                rule_id: "SP-KM1-MALFORMED-PARAM".into(),
                                module_name: module.clone(),
                                description: format!("Parameter syntax invalid: {}", e),
                                fatal: true,
                            });
                        }
                    } else {
                        // Flag parameter without '='
                        if let Err(e) = validate_module_name(opt) {
                            violations.push(KernelModulePolicyViolation {
                                rule_id: "SP-KM1-MALFORMED-PARAM".into(),
                                module_name: module.clone(),
                                description: format!("Parameter flag syntax invalid: {}", e),
                                fatal: true,
                            });
                        }
                    }
                }
            }
            ModprobeRule::Alias { alias, module } => {
                if alias.len() > 64 || alias.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-') {
                    violations.push(KernelModulePolicyViolation {
                        rule_id: "SP-KM1-INVALID-ALIAS".into(),
                        module_name: module.clone(),
                        description: format!("Alias '{}' syntax is invalid", alias),
                        fatal: true,
                    });
                }
            }
            _ => {}
        }

        // SP-KM6: Tri-state mode evaluation
        let allowed = match self.mode {
            KernelModulePolicyMode::Enforcing => !violations.iter().any(|v| v.fatal),
            KernelModulePolicyMode::Audit => true,
            KernelModulePolicyMode::Permissive => {
                // In permissive mode, only protected module destruction or install command injection are blocked
                !violations.iter().any(|v| {
                    v.rule_id == "SP-KM3-PROTECTED-MODULE"
                        || v.rule_id == "SP-KM4-INSTALL-COMMAND-INJECTION"
                        || v.rule_id == "SP-KM4-UNAPPROVED-INSTALL-CMD"
                })
            }
        };

        KernelModulePolicyVerdict {
            module_name: mod_name,
            allowed,
            mode: self.mode,
            violations,
            evaluated_at: "2026-09-19T00:00:00Z".into(),
        }
    }

    /// Evaluates an autoload entry against this security policy.
    pub fn evaluate_autoload(&self, module: &str) -> KernelModulePolicyVerdict {
        let mut violations = Vec::new();

        // SP-KM1: Validate module name
        if let Err(e) = validate_module_name(module) {
            violations.push(KernelModulePolicyViolation {
                rule_id: "SP-KM1-INVALID-NAME".into(),
                module_name: module.to_string(),
                description: format!("Autoload module name syntax invalid: {}", e),
                fatal: true,
            });
        }

        // SP-KM2: Prohibited module in autoload
        if self.prohibited_modules.iter().any(|p| p.eq_ignore_ascii_case(module)) {
            violations.push(KernelModulePolicyViolation {
                rule_id: "SP-KM2-PROHIBITED-AUTOLOAD".into(),
                module_name: module.to_string(),
                description: format!(
                    "Module '{}' is prohibited by security policy and cannot be autoloaded",
                    module
                ),
                fatal: true,
            });
        }

        let allowed = match self.mode {
            KernelModulePolicyMode::Enforcing => !violations.iter().any(|v| v.fatal),
            KernelModulePolicyMode::Audit => true,
            KernelModulePolicyMode::Permissive => !violations.iter().any(|v| v.rule_id == "SP-KM1-INVALID-NAME"),
        };

        KernelModulePolicyVerdict {
            module_name: module.to_string(),
            allowed,
            mode: self.mode,
            violations,
            evaluated_at: "2026-09-19T00:00:00Z".into(),
        }
    }

    /// Evaluates all directives and autoload entries in a KernelModuleStore.
    pub fn evaluate_store(&self, store: &KernelModuleStore) -> Vec<KernelModulePolicyVerdict> {
        let mut verdicts = Vec::new();

        for rule in &store.config.rules {
            verdicts.push(self.evaluate_rule(rule));
        }

        for module in &store.config.autoload_modules {
            verdicts.push(self.evaluate_autoload(module));
        }

        verdicts
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
        if !metadata.is_file() {
            return Err(format!("policy path '{}' is not a regular file", path.display()));
        }

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

    /// Helper for reading policy overrides from environment or lookup function.
    pub fn from_source<F: Fn(&str) -> Option<String>>(lookup: F) -> Result<Self, String> {
        let mut policy = Self::default();

        if let Some(val) = lookup("AIOS_KERNEL_MODULE_POLICY_MODE") {
            policy.mode = match val.to_lowercase().as_str() {
                "enforcing" => KernelModulePolicyMode::Enforcing,
                "audit" => KernelModulePolicyMode::Audit,
                "permissive" => KernelModulePolicyMode::Permissive,
                other => return Err(format!("unknown AIOS_KERNEL_MODULE_POLICY_MODE value '{}'", other)),
            };
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
