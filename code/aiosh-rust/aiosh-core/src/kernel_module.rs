//! Kernel Module Management Data Model (KM1..KM5)
//!
//! Governs Linux kernel module introspection, parameters, modprobe.d configuration
//! directives, and CIS security baseline hardening for AIOS.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Runtime lifecycle state of a Linux kernel module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleState {
    Live,
    Loading,
    Unloading,
    Unloaded,
}

impl std::str::FromStr for ModuleState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "live" => Ok(ModuleState::Live),
            "loading" => Ok(ModuleState::Loading),
            "unloading" => Ok(ModuleState::Unloading),
            "unloaded" => Ok(ModuleState::Unloaded),
            other => Err(format!("unknown module state '{}'", other)),
        }
    }
}

/// A kernel module parameter descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleParameter {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub readonly: bool,
}

/// Comprehensive information for a Linux kernel module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub size_bytes: u64,
    pub ref_count: u32,
    #[serde(default)]
    pub used_by: Vec<String>,
    pub state: ModuleState,
    #[serde(default)]
    pub parameters: Vec<ModuleParameter>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub srcversion: Option<String>,
    #[serde(default)]
    pub vermagic: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub taint_flags: Option<String>,
}

impl ModuleInfo {
    /// Parses a single line from `/proc/modules`.
    /// Format: `<name> <size> <refcnt> <used_by,...> <state> <address>`
    /// Example: `overlay 151552 1 - Live 0x0000000000000000`
    pub fn parse_proc_modules_line(line: &str) -> Result<Self, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(format!(
                "invalid /proc/modules line (expected at least 5 columns, got {}): '{}'",
                parts.len(),
                line
            ));
        }

        let name = parts[0].to_string();
        validate_module_name(&name)?;

        let size_bytes = parts[1]
            .parse::<u64>()
            .map_err(|e| format!("invalid size in /proc/modules: {}", e))?;

        let ref_count = parts[2]
            .parse::<u32>()
            .map_err(|e| format!("invalid refcount in /proc/modules: {}", e))?;

        let used_by = if parts[3] == "-" || parts[3].is_empty() {
            Vec::new()
        } else {
            parts[3]
                .trim_end_matches(',')
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };

        let state = parts[4]
            .parse::<ModuleState>()
            .map_err(|e| format!("invalid state in /proc/modules: {}", e))?;

        Ok(ModuleInfo {
            name,
            size_bytes,
            ref_count,
            used_by,
            state,
            parameters: Vec::new(),
            description: None,
            license: None,
            version: None,
            srcversion: None,
            vermagic: None,
            signature: None,
            taint_flags: None,
        })
    }
}

/// Standard modprobe.d(5) configuration directive rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModprobeRule {
    Blacklist {
        module: String,
    },
    Alias {
        alias: String,
        module: String,
    },
    Options {
        module: String,
        options: Vec<String>,
    },
    Install {
        module: String,
        command: String,
    },
    Remove {
        module: String,
        command: String,
    },
    Softdep {
        module: String,
        #[serde(default)]
        pre: Vec<String>,
        #[serde(default)]
        post: Vec<String>,
    },
}

/// Declarative kernel module configuration profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelModuleConfig {
    pub id: String,
    pub description: String,
    pub rules: Vec<ModprobeRule>,
    #[serde(default)]
    pub autoload_modules: Vec<String>,
    pub created_at: String,
}

/// Canonical preset profiles for kernel module configurations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelModulePreset {
    pub name: String,
    pub description: String,
    pub config: KernelModuleConfig,
}

/// Validates kernel module identifier syntax (KM1).
pub fn validate_module_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("module name cannot be empty".into());
    }
    if name.len() > 64 {
        return Err(format!("module name exceeds 64 characters (was {})", name.len()));
    }
    for c in name.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return Err(format!(
                "module name contains invalid character '{}' (only ASCII alphanumeric and '_' allowed)",
                c
            ));
        }
    }
    Ok(())
}

/// Validates module parameter key and value safety (KM2).
pub fn validate_parameter(key: &str, value: &str) -> Result<(), String> {
    validate_module_name(key)?;
    if value.len() > 1024 {
        return Err(format!("parameter value exceeds 1024 bytes (was {})", value.len()));
    }
    for c in value.chars() {
        if c.is_ascii_control() || c == '\n' || c == '\r' || c == ';' || c == '&' || c == '|' || c == '`' || c == '$' {
            return Err(format!(
                "parameter value contains illegal or shell metacharacter '{}'",
                c
            ));
        }
    }
    Ok(())
}

/// Validates a complete declarative kernel module configuration (KM3, KM4).
pub fn validate_config(config: &KernelModuleConfig) -> Result<(), String> {
    if config.id.trim().is_empty() {
        return Err("config id cannot be empty".into());
    }

    let mut blacklisted = BTreeSet::new();
    let mut disabled = BTreeSet::new();

    for rule in &config.rules {
        match rule {
            ModprobeRule::Blacklist { module } => {
                validate_module_name(module)?;
                blacklisted.insert(module.as_str());
            }
            ModprobeRule::Alias { alias, module } => {
                if alias.is_empty() || alias.len() > 64 {
                    return Err("alias name must be 1..64 chars".into());
                }
                validate_module_name(module)?;
            }
            ModprobeRule::Options { module, options } => {
                validate_module_name(module)?;
                for opt in options {
                    if let Some((k, v)) = opt.split_once('=') {
                        validate_parameter(k, v)?;
                    } else {
                        validate_module_name(opt)?;
                    }
                }
            }
            ModprobeRule::Install { module, command } => {
                validate_module_name(module)?;
                if command.trim().is_empty() {
                    return Err("install command cannot be empty".into());
                }
                if command.contains("/bin/true") || command.contains("/bin/false") {
                    disabled.insert(module.as_str());
                }
            }
            ModprobeRule::Remove { module, command } => {
                validate_module_name(module)?;
                if command.trim().is_empty() {
                    return Err("remove command cannot be empty".into());
                }
            }
            ModprobeRule::Softdep { module, pre, post } => {
                validate_module_name(module)?;
                for m in pre.iter().chain(post.iter()) {
                    validate_module_name(m)?;
                }
            }
        }
    }

    // KM3: Autoload modules cannot be blacklisted or disabled
    for m in &config.autoload_modules {
        validate_module_name(m)?;
        if blacklisted.contains(m.as_str()) {
            return Err(format!("module '{}' cannot be both autoloaded and blacklisted", m));
        }
        if disabled.contains(m.as_str()) {
            return Err(format!("module '{}' cannot be both autoloaded and disabled with install command", m));
        }
    }

    Ok(())
}

impl KernelModuleConfig {
    /// Generates standard Linux modprobe.d(5) configuration content (KM5).
    pub fn to_modprobe_conf(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# AIOS Kernel Module Configuration: {}\n", self.id));
        out.push_str(&format!("# {}\n\n", self.description));

        for rule in &self.rules {
            match rule {
                ModprobeRule::Blacklist { module } => {
                    out.push_str(&format!("blacklist {}\n", module));
                }
                ModprobeRule::Alias { alias, module } => {
                    out.push_str(&format!("alias {} {}\n", alias, module));
                }
                ModprobeRule::Options { module, options } => {
                    out.push_str(&format!("options {} {}\n", module, options.join(" ")));
                }
                ModprobeRule::Install { module, command } => {
                    out.push_str(&format!("install {} {}\n", module, command));
                }
                ModprobeRule::Remove { module, command } => {
                    out.push_str(&format!("remove {} {}\n", module, command));
                }
                ModprobeRule::Softdep { module, pre, post } => {
                    let mut s = format!("softdep {}", module);
                    if !pre.is_empty() {
                        s.push_str(&format!(" pre: {}", pre.join(" ")));
                    }
                    if !post.is_empty() {
                        s.push_str(&format!(" post: {}", post.join(" ")));
                    }
                    s.push('\n');
                    out.push_str(&s);
                }
            }
        }
        out
    }

    /// Generates standard Linux /etc/modules-load.d/ content for autoloaded modules.
    pub fn to_modules_load_conf(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# AIOS Autoload Modules: {}\n\n", self.id));
        for m in &self.autoload_modules {
            out.push_str(m);
            out.push('\n');
        }
        out
    }
}

/// Parses modprobe.d syntax into ModprobeRule items (KM5).
pub fn parse_modprobe_conf(content: &str) -> Result<Vec<ModprobeRule>, String> {
    let mut rules = Vec::new();
    for (line_no, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "blacklist" => {
                if parts.len() < 2 {
                    return Err(format!("line {}: blacklist directive requires module name", line_no + 1));
                }
                rules.push(ModprobeRule::Blacklist { module: parts[1].to_string() });
            }
            "alias" => {
                if parts.len() < 3 {
                    return Err(format!("line {}: alias directive requires alias and module", line_no + 1));
                }
                rules.push(ModprobeRule::Alias {
                    alias: parts[1].to_string(),
                    module: parts[2].to_string(),
                });
            }
            "options" => {
                if parts.len() < 3 {
                    return Err(format!("line {}: options directive requires module and options", line_no + 1));
                }
                rules.push(ModprobeRule::Options {
                    module: parts[1].to_string(),
                    options: parts[2..].iter().map(|s| s.to_string()).collect(),
                });
            }
            "install" => {
                if parts.len() < 3 {
                    return Err(format!("line {}: install directive requires module and command", line_no + 1));
                }
                rules.push(ModprobeRule::Install {
                    module: parts[1].to_string(),
                    command: parts[2..].join(" "),
                });
            }
            "remove" => {
                if parts.len() < 3 {
                    return Err(format!("line {}: remove directive requires module and command", line_no + 1));
                }
                rules.push(ModprobeRule::Remove {
                    module: parts[1].to_string(),
                    command: parts[2..].join(" "),
                });
            }
            "softdep" => {
                if parts.len() < 2 {
                    return Err(format!("line {}: softdep directive requires module name", line_no + 1));
                }
                let module = parts[1].to_string();
                let mut pre = Vec::new();
                let mut post = Vec::new();
                let mut target = &mut pre;
                for &part in &parts[2..] {
                    if part == "pre:" {
                        target = &mut pre;
                    } else if part == "post:" {
                        target = &mut post;
                    } else {
                        target.push(part.to_string());
                    }
                }
                rules.push(ModprobeRule::Softdep { module, pre, post });
            }
            other => {
                return Err(format!("line {}: unknown directive '{}'", line_no + 1, other));
            }
        }
    }
    Ok(rules)
}

/// Canonical CIS Hardened preset (KM4).
pub fn cis_hardened_preset() -> KernelModulePreset {
    let legacy_fs = ["cramfs", "freevxfs", "jffs2", "hfs", "hfsplus", "udf"];
    let obsolete_proto = ["dccp", "sctp", "rds", "tipc"];

    let mut rules = Vec::new();
    for fs in legacy_fs {
        rules.push(ModprobeRule::Install {
            module: fs.to_string(),
            command: "/bin/true".to_string(),
        });
        rules.push(ModprobeRule::Blacklist {
            module: fs.to_string(),
        });
    }
    for proto in obsolete_proto {
        rules.push(ModprobeRule::Install {
            module: proto.to_string(),
            command: "/bin/true".to_string(),
        });
        rules.push(ModprobeRule::Blacklist {
            module: proto.to_string(),
        });
    }

    KernelModulePreset {
        name: "cis_hardened_baseline".to_string(),
        description: "CIS benchmark hardening profile disabling legacy filesystems and protocols".to_string(),
        config: KernelModuleConfig {
            id: "cis-hardened-v1".to_string(),
            description: "CIS hardened modprobe configuration".to_string(),
            rules,
            autoload_modules: vec![],
            created_at: "2026-09-19T00:00:00Z".to_string(),
        },
    }
}

/// Canonical Wireless Penetration Testing preset.
pub fn pentest_wireless_preset() -> KernelModulePreset {
    KernelModulePreset {
        name: "pentest_wireless_baseline".to_string(),
        description: "Wireless penetration testing baseline drivers and monitor mode modules".to_string(),
        config: KernelModuleConfig {
            id: "pentest-wireless-v1".to_string(),
            description: "Wireless penetration testing drivers".to_string(),
            rules: vec![
                ModprobeRule::Options {
                    module: "ath9k_htc".to_string(),
                    options: vec!["nohwcrypt=1".to_string()],
                },
                ModprobeRule::Options {
                    module: "rtl8812au".to_string(),
                    options: vec!["rtw_vht_enable=1".to_string()],
                },
            ],
            autoload_modules: vec![
                "cfg80211".to_string(),
                "mac80211".to_string(),
            ],
            created_at: "2026-09-19T00:00:00Z".to_string(),
        },
    }
}

/// Canonical Container & Sandboxing Isolation preset.
pub fn container_isolation_preset() -> KernelModulePreset {
    KernelModulePreset {
        name: "container_isolation_baseline".to_string(),
        description: "Container runtime and network namespace kernel modules".to_string(),
        config: KernelModuleConfig {
            id: "container-isolation-v1".to_string(),
            description: "Container virtualization and isolation modules".to_string(),
            rules: vec![
                ModprobeRule::Options {
                    module: "overlay".to_string(),
                    options: vec!["metacopy=on".to_string()],
                },
            ],
            autoload_modules: vec![
                "overlay".to_string(),
                "tun".to_string(),
                "br_netfilter".to_string(),
                "nf_tables".to_string(),
            ],
            created_at: "2026-09-19T00:00:00Z".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_km1_validate_module_name() {
        assert!(validate_module_name("overlay").is_ok());
        assert!(validate_module_name("ath9k_htc").is_ok());
        assert!(validate_module_name("br_netfilter").is_ok());

        assert!(validate_module_name("").is_err());
        assert!(validate_module_name("overlay;rm -rf /").is_err());
        assert!(validate_module_name("../sys/module").is_err());
        assert!(validate_module_name("invalid-name-with-dash").is_err());
        assert!(validate_module_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_km2_validate_parameter() {
        assert!(validate_parameter("metacopy", "on").is_ok());
        assert!(validate_parameter("nohwcrypt", "1").is_ok());
        assert!(validate_parameter("debug", "0").is_ok());

        assert!(validate_parameter("", "val").is_err());
        assert!(validate_parameter("param", "val;whoami").is_err());
        assert!(validate_parameter("param", "val|sh").is_err());
        assert!(validate_parameter("param", "val`id`").is_err());
        assert!(validate_parameter("param", "val$HOME").is_err());
    }

    #[test]
    fn test_km3_blacklist_conflict_detection() {
        let mut config = container_isolation_preset().config;
        assert!(validate_config(&config).is_ok());

        // Add conflict: blacklist autoloaded module 'overlay'
        config.rules.push(ModprobeRule::Blacklist {
            module: "overlay".to_string(),
        });
        assert!(validate_config(&config).is_err());

        // Add conflict: disable autoloaded module with install /bin/true
        let mut config2 = container_isolation_preset().config;
        config2.rules.push(ModprobeRule::Install {
            module: "tun".to_string(),
            command: "/bin/true".to_string(),
        });
        assert!(validate_config(&config2).is_err());
    }

    #[test]
    fn test_km4_cis_hardened_preset() {
        let preset = cis_hardened_preset();
        assert_eq!(preset.name, "cis_hardened_baseline");
        assert!(validate_config(&preset.config).is_ok());

        let conf = preset.config.to_modprobe_conf();
        assert!(conf.contains("install cramfs /bin/true"));
        assert!(conf.contains("blacklist cramfs"));
        assert!(conf.contains("install dccp /bin/true"));
        assert!(conf.contains("blacklist dccp"));
    }

    #[test]
    fn test_km5_modprobe_roundtrip() {
        let preset = pentest_wireless_preset();
        let conf_str = preset.config.to_modprobe_conf();

        let parsed_rules = parse_modprobe_conf(&conf_str).expect("parse failed");
        assert_eq!(parsed_rules.len(), preset.config.rules.len());
        assert_eq!(parsed_rules, preset.config.rules);
    }

    #[test]
    fn test_proc_modules_parsing() {
        let line = "overlay 151552 1 - Live 0x0000000000000000";
        let info = ModuleInfo::parse_proc_modules_line(line).expect("parse failed");
        assert_eq!(info.name, "overlay");
        assert_eq!(info.size_bytes, 151552);
        assert_eq!(info.ref_count, 1);
        assert!(info.used_by.is_empty());
        assert_eq!(info.state, ModuleState::Live);

        let line_with_deps = "cfg80211 983040 2 ath9k_htc,mac80211, Live 0x0000000000000000";
        let info_deps = ModuleInfo::parse_proc_modules_line(line_with_deps).expect("parse failed");
        assert_eq!(info_deps.name, "cfg80211");
        assert_eq!(info_deps.ref_count, 2);
        assert_eq!(info_deps.used_by, vec!["ath9k_htc", "mac80211"]);
    }
}
