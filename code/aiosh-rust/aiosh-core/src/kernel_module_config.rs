//! Kernel Module Management Configuration Subsystem
//!
//! Handles subsystem configuration, modprobe.d and modules-load.d import/export,
//! and validation invariants CFG-KM1..CFG-KM5.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::kernel_module::{
    validate_module_name, validate_parameter, ModprobeRule,
};
use crate::kernel_module_service::{KernelModuleStore, MAX_MODULE_DOC_BYTES};

/// Subsystem configuration for Kernel Module Management.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelModuleManagementConfig {
    /// Default path to the canonical store JSON.
    pub default_store_path: PathBuf,
    /// Target path for exported modprobe.d configuration directives.
    pub modprobe_d_path: PathBuf,
    /// Target path for exported modules-load.d configuration directives.
    pub modules_load_d_path: PathBuf,
    /// Path to live kernel module introspection file (typically /proc/modules).
    pub proc_modules_path: PathBuf,
    /// Maximum allowed store document size in bytes.
    pub max_doc_bytes: u64,
    /// Whether pre-commit conflict prevention between blacklist and autoload is strictly enforced.
    pub strict_conflict_prevention: bool,
    /// Whether non-standard install commands outside /bin/true and /bin/false are permitted.
    pub allow_custom_commands: bool,
}

impl Default for KernelModuleManagementConfig {
    fn default() -> Self {
        Self {
            default_store_path: PathBuf::from(".aios/kernel_modules.json"),
            modprobe_d_path: PathBuf::from("/etc/modprobe.d/aios.conf"),
            modules_load_d_path: PathBuf::from("/etc/modules-load.d/aios.conf"),
            proc_modules_path: PathBuf::from("/proc/modules"),
            max_doc_bytes: MAX_MODULE_DOC_BYTES,
            strict_conflict_prevention: true,
            allow_custom_commands: false,
        }
    }
}

impl KernelModuleManagementConfig {
    /// Validates the subsystem configuration against CFG-KM1..CFG-KM5 invariants.
    pub fn validate(&self) -> Result<(), String> {
        if self.default_store_path.as_os_str().is_empty() {
            return Err("CFG-KM1 violation: default_store_path cannot be empty".into());
        }
        if let Some(s) = self.default_store_path.to_str() {
            if s.chars().any(|c| c.is_control() || c == '\0') {
                return Err("CFG-KM2 violation: default_store_path cannot contain control characters".into());
            }
            if s.len() > 1024 {
                return Err("CFG-KM2 violation: default_store_path exceeds 1024 bytes".into());
            }
        }
        if self.modprobe_d_path.as_os_str().is_empty() {
            return Err("CFG-KM1 violation: modprobe_d_path cannot be empty".into());
        }
        if self.modules_load_d_path.as_os_str().is_empty() {
            return Err("CFG-KM1 violation: modules_load_d_path cannot be empty".into());
        }
        if self.max_doc_bytes == 0 || self.max_doc_bytes > MAX_MODULE_DOC_BYTES {
            return Err(format!(
                "CFG-KM4 violation: max_doc_bytes must be between 1 and {}",
                MAX_MODULE_DOC_BYTES
            ));
        }
        Ok(())
    }
}

/// Parses a single line from a modprobe.d configuration file.
pub fn parse_modprobe_conf_line(line: &str) -> Result<Option<ModprobeRule>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return Ok(None);
    }

    match tokens[0] {
        "blacklist" => {
            if tokens.len() != 2 {
                return Err(format!("'blacklist' requires exactly 1 module argument: '{}'", line));
            }
            let module = tokens[1];
            validate_module_name(module)?;
            Ok(Some(ModprobeRule::Blacklist {
                module: module.to_string(),
            }))
        }
        "options" => {
            if tokens.len() < 3 {
                return Err(format!("'options' requires module and at least one parameter: '{}'", line));
            }
            let module = tokens[1];
            validate_module_name(module)?;
            let mut options = Vec::new();
            for &opt in &tokens[2..] {
                if let Some((k, v)) = opt.split_once('=') {
                    validate_parameter(k, v)?;
                } else {
                    validate_module_name(opt)?;
                }
                options.push(opt.to_string());
            }
            Ok(Some(ModprobeRule::Options {
                module: module.to_string(),
                options,
            }))
        }
        "install" => {
            if tokens.len() < 3 {
                return Err(format!("'install' requires module and command: '{}'", line));
            }
            let module = tokens[1];
            validate_module_name(module)?;
            let command = tokens[2..].join(" ");
            Ok(Some(ModprobeRule::Install {
                module: module.to_string(),
                command,
            }))
        }
        "alias" => {
            if tokens.len() != 3 {
                return Err(format!("'alias' requires alias name and module name: '{}'", line));
            }
            let alias = tokens[1];
            let module = tokens[2];
            validate_module_name(alias)?;
            validate_module_name(module)?;
            Ok(Some(ModprobeRule::Alias {
                alias: alias.to_string(),
                module: module.to_string(),
            }))
        }
        "softdep" => {
            if tokens.len() < 3 {
                return Err(format!("'softdep' requires module and dependencies: '{}'", line));
            }
            let module = tokens[1];
            validate_module_name(module)?;
            let mut pre = Vec::new();
            let mut post = Vec::new();
            let mut current = &mut pre;
            for &tok in &tokens[2..] {
                if tok == "pre:" {
                    current = &mut pre;
                } else if tok == "post:" {
                    current = &mut post;
                } else {
                    validate_module_name(tok)?;
                    current.push(tok.to_string());
                }
            }
            Ok(Some(ModprobeRule::Softdep {
                module: module.to_string(),
                pre,
                post,
            }))
        }
        "remove" => {
            if tokens.len() < 3 {
                return Err(format!("'remove' requires module and command: '{}'", line));
            }
            let module = tokens[1];
            validate_module_name(module)?;
            let command = tokens[2..].join(" ");
            Ok(Some(ModprobeRule::Remove {
                module: module.to_string(),
                command,
            }))
        }
        other => Err(format!("unrecognized modprobe directive '{}'", other)),
    }
}

/// Parses a single line from a modules-load.d configuration file.
pub fn parse_modules_load_conf_line(line: &str) -> Result<Option<String>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
        return Ok(None);
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return Ok(None);
    }

    let module = tokens[0];
    validate_module_name(module)?;
    Ok(Some(module.to_string()))
}

/// Imports modprobe.d configuration text into a vector of ModprobeRules.
pub fn import_modprobe_conf(content: &str) -> Result<Vec<ModprobeRule>, String> {
    let mut rules = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        match parse_modprobe_conf_line(line) {
            Ok(Some(rule)) => rules.push(rule),
            Ok(None) => {}
            Err(e) => return Err(format!("line {}: {}", line_num + 1, e)),
        }
    }
    Ok(rules)
}

/// Imports modules-load.d configuration text into a vector of module names.
pub fn import_modules_load_conf(content: &str) -> Result<Vec<String>, String> {
    let mut modules = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        match parse_modules_load_conf_line(line) {
            Ok(Some(module)) => {
                if !modules.contains(&module) {
                    modules.push(module);
                }
            }
            Ok(None) => {}
            Err(e) => return Err(format!("line {}: {}", line_num + 1, e)),
        }
    }
    Ok(modules)
}

/// Ingests a modprobe.d file into a KernelModuleStore.
pub fn import_modprobe_file_to_store(store: &mut KernelModuleStore, path: &Path) -> Result<usize, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read modprobe file {:?}: {}", path, e))?;
    let rules = import_modprobe_conf(&content)?;
    let count = rules.len();
    for rule in rules {
        match rule {
            ModprobeRule::Blacklist { module } => {
                store.add_blacklist(&module)?;
            }
            ModprobeRule::Options { module, options } => {
                store.add_options(&module, options)?;
            }
            other => {
                store.config.rules.push(other);
            }
        }
    }
    store.validate()?;
    Ok(count)
}

/// Ingests a modules-load.d file into a KernelModuleStore.
pub fn import_modules_load_file_to_store(store: &mut KernelModuleStore, path: &Path) -> Result<usize, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read modules-load file {:?}: {}", path, e))?;
    let modules = import_modules_load_conf(&content)?;
    let count = modules.len();
    for module in modules {
        store.add_autoload(&module)?;
    }
    store.validate()?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_km_config_validation() {
        let mut config = KernelModuleManagementConfig::default();
        assert!(config.validate().is_ok());

        config.default_store_path = PathBuf::from("");
        assert!(config.validate().is_err());

        config.default_store_path = PathBuf::from("bad\x07path");
        assert!(config.validate().is_err());

        config.default_store_path = PathBuf::from("a".repeat(1025));
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cfg_km_parse_modprobe() {
        let sample = r#"
# Security blacklist
blacklist cramfs
blacklist freevxfs

# Options
options i915 enable_guc=3 enable_fbc=1

# Install
install dccp /bin/true

# Alias
alias net_pf_33 dccp
"#;
        let rules = import_modprobe_conf(sample).expect("should parse");
        assert_eq!(rules.len(), 5);
        assert_eq!(rules[0], ModprobeRule::Blacklist { module: "cramfs".into() });
        assert_eq!(rules[1], ModprobeRule::Blacklist { module: "freevxfs".into() });
        assert_eq!(
            rules[2],
            ModprobeRule::Options {
                module: "i915".into(),
                options: vec!["enable_guc=3".into(), "enable_fbc=1".into()]
            }
        );
        assert_eq!(
            rules[3],
            ModprobeRule::Install {
                module: "dccp".into(),
                command: "/bin/true".into()
            }
        );
        assert_eq!(
            rules[4],
            ModprobeRule::Alias {
                alias: "net_pf_33".into(),
                module: "dccp".into()
            }
        );
    }

    #[test]
    fn test_cfg_km_parse_modules_load() {
        let sample = r#"
# Modules loaded at boot
overlay
br_netfilter
; alternate comment
wireguard
"#;
        let modules = import_modules_load_conf(sample).expect("should parse");
        assert_eq!(modules, vec!["overlay", "br_netfilter", "wireguard"]);
    }

    #[test]
    fn test_cfg_km_import_to_store_with_conflicts() {
        let mut store = KernelModuleStore::new("test-store", "test");
        store.add_autoload("overlay").expect("autoload should succeed");

        let modprobe_text = "blacklist overlay\n";
        let rules = import_modprobe_conf(modprobe_text).expect("parse should succeed");
        assert_eq!(rules.len(), 1);

        // Attempting to add blacklisted overlay to store with autoload overlay must fail (CFG-KM3)
        assert!(store.add_blacklist("overlay").is_err());
    }
}
