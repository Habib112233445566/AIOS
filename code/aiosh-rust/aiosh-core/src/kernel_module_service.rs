//! Kernel Module Core Service (KS1..KS5)
//!
//! Provides the runtime `KernelModuleService` coordinator, store management,
//! modprobe directive generation, live module introspection, and atomic persistence.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::kernel_module::{
    cis_hardened_preset, container_isolation_preset, pentest_wireless_preset,
    validate_config, validate_module_name, validate_parameter, KernelModuleConfig,
    KernelModulePreset, ModprobeRule, ModuleInfo,
};

/// Maximum document size for kernel module store JSON (10 MiB, KS5).
pub const MAX_MODULE_DOC_BYTES: u64 = 10 * 1024 * 1024;

/// In-memory and persistent store for kernel module configurations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelModuleStore {
    pub config: KernelModuleConfig,
}

impl KernelModuleStore {
    /// Creates a new empty store.
    pub fn new(id: &str, description: &str) -> Self {
        KernelModuleStore {
            config: KernelModuleConfig {
                id: id.to_string(),
                description: description.to_string(),
                rules: Vec::new(),
                autoload_modules: Vec::new(),
                created_at: "2026-09-19T00:00:00Z".to_string(),
            },
        }
    }

    /// Initializes a store from a preset profile.
    pub fn from_preset(preset: &KernelModulePreset) -> Self {
        KernelModuleStore {
            config: preset.config.clone(),
        }
    }

    /// Validates the store invariants (KM1..KM5, KS2).
    pub fn validate(&self) -> Result<(), String> {
        validate_config(&self.config)
    }

    /// Adds a blacklist rule idempotently (KS2, KS4).
    pub fn add_blacklist(&mut self, module: &str) -> Result<(), String> {
        validate_module_name(module)?;
        if self.config.autoload_modules.iter().any(|m| m == module) {
            return Err(format!("module '{}' is autoloaded; cannot blacklist", module));
        }
        // Deduplicate
        if !self.config.rules.iter().any(|r| match r {
            ModprobeRule::Blacklist { module: m } => m == module,
            _ => false,
        }) {
            self.config.rules.push(ModprobeRule::Blacklist {
                module: module.to_string(),
            });
        }
        self.validate()
    }

    /// Removes a blacklist rule.
    pub fn remove_blacklist(&mut self, module: &str) -> bool {
        let before = self.config.rules.len();
        self.config.rules.retain(|r| match r {
            ModprobeRule::Blacklist { module: m } => m != module,
            _ => true,
        });
        self.config.rules.len() < before
    }

    /// Adds or updates options for a module (KS4).
    pub fn add_options(&mut self, module: &str, options: Vec<String>) -> Result<(), String> {
        validate_module_name(module)?;
        for opt in &options {
            if let Some((k, v)) = opt.split_once('=') {
                validate_parameter(k, v)?;
            } else {
                validate_module_name(opt)?;
            }
        }

        // Update in-place or append
        if let Some(rule) = self.config.rules.iter_mut().find(|r| match r {
            ModprobeRule::Options { module: m, .. } => m == module,
            _ => false,
        }) {
            *rule = ModprobeRule::Options {
                module: module.to_string(),
                options,
            };
        } else {
            self.config.rules.push(ModprobeRule::Options {
                module: module.to_string(),
                options,
            });
        }
        self.validate()
    }

    /// Adds an autoload module (KS2, KS4).
    pub fn add_autoload(&mut self, module: &str) -> Result<(), String> {
        validate_module_name(module)?;
        // Check blacklist conflict
        let blacklisted = self.config.rules.iter().any(|r| match r {
            ModprobeRule::Blacklist { module: m } => m == module,
            ModprobeRule::Install { module: m, command } => {
                m == module && (command.contains("/bin/true") || command.contains("/bin/false"))
            }
            _ => false,
        });
        if blacklisted {
            return Err(format!("module '{}' is blacklisted or disabled; cannot autoload", module));
        }

        if !self.config.autoload_modules.iter().any(|m| m == module) {
            self.config.autoload_modules.push(module.to_string());
        }
        self.validate()
    }

    /// Removes an autoload module.
    pub fn remove_autoload(&mut self, module: &str) -> bool {
        let before = self.config.autoload_modules.len();
        self.config.autoload_modules.retain(|m| m != module);
        self.config.autoload_modules.len() < before
    }

    /// Saves store to disk atomically with size bounding (KS3, KS5).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize kernel module store: {}", e))?;

        if data.len() as u64 > MAX_MODULE_DOC_BYTES {
            return Err(format!(
                "store size ({} bytes) exceeds maximum limit ({} bytes)",
                data.len(),
                MAX_MODULE_DOC_BYTES
            ));
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let tmp_path = parent.join(format!(".tmp.{}.{}", std::process::id(), filename));

        let mut f = fs::File::create(&tmp_path)
            .map_err(|e| format!("failed to create temporary file {:?}: {}", tmp_path, e))?;

        if let Err(e) = f.write_all(data.as_bytes()) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("failed to write temporary file: {}", e));
        }

        if let Err(e) = f.sync_all() {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("failed to sync temporary file: {}", e));
        }
        drop(f);

        if let Err(e) = fs::rename(&tmp_path, path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("failed to atomically replace {:?}: {}", path, e));
        }

        Ok(())
    }

    /// Loads store from disk with size bounding (KS5).
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let meta = fs::metadata(path)
            .map_err(|e| format!("failed to read store metadata {:?}: {}", path, e))?;

        if meta.len() > MAX_MODULE_DOC_BYTES {
            return Err(format!(
                "store file {:?} ({} bytes) exceeds maximum limit ({} bytes)",
                path, meta.len(), MAX_MODULE_DOC_BYTES
            ));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("failed to read store file {:?}: {}", path, e))?;

        let store: KernelModuleStore = serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse kernel module store: {}", e))?;

        store.validate()?;
        Ok(store)
    }

    /// Exports standard modprobe.d configuration string.
    pub fn export_modprobe_conf(&self) -> String {
        self.config.to_modprobe_conf()
    }

    /// Exports standard modules-load.d configuration string.
    pub fn export_modules_load_conf(&self) -> String {
        self.config.to_modules_load_conf()
    }
}

/// Runtime coordinator service for kernel module inspection and management.
#[derive(Debug)]
pub struct KernelModuleService {
    pub store: KernelModuleStore,
    pub proc_modules_path: PathBuf,
}

impl KernelModuleService {
    /// Creates a new service with the given store.
    pub fn new(store: KernelModuleStore) -> Self {
        KernelModuleService {
            store,
            proc_modules_path: PathBuf::from("/proc/modules"),
        }
    }

    /// Overrides proc modules path for testing.
    pub fn with_proc_modules_path(mut self, path: PathBuf) -> Self {
        self.proc_modules_path = path;
        self
    }

    /// Lists loaded modules from procfs (KS1).
    pub fn list_loaded_modules(&self) -> Result<Vec<ModuleInfo>, String> {
        if !self.proc_modules_path.exists() {
            // Graceful fallback for non-Linux or containerized mock environments
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.proc_modules_path)
            .map_err(|e| format!("failed to read {:?}: {}", self.proc_modules_path, e))?;

        let mut modules = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let info = ModuleInfo::parse_proc_modules_line(line)?;
            modules.push(info);
        }
        Ok(modules)
    }

    /// Queries a single loaded module by name.
    pub fn get_module(&self, name: &str) -> Result<Option<ModuleInfo>, String> {
        let loaded = self.list_loaded_modules()?;
        Ok(loaded.into_iter().find(|m| m.name == name))
    }

    /// Lists available canonical presets.
    pub fn list_presets(&self) -> Vec<KernelModulePreset> {
        vec![
            cis_hardened_preset(),
            pentest_wireless_preset(),
            container_isolation_preset(),
        ]
    }

    /// Applies a canonical preset into the active store (KS4).
    pub fn apply_preset(&mut self, preset_name: &str) -> Result<(), String> {
        let presets = self.list_presets();
        let target = presets.into_iter().find(|p| p.name == preset_name)
            .ok_or_else(|| format!("preset '{}' not found", preset_name))?;

        for rule in target.config.rules {
            match rule {
                ModprobeRule::Blacklist { module } => {
                    self.store.add_blacklist(&module)?;
                }
                ModprobeRule::Options { module, options } => {
                    self.store.add_options(&module, options)?;
                }
                ModprobeRule::Install { module, command } => {
                    validate_module_name(&module)?;
                    self.store.config.rules.push(ModprobeRule::Install { module, command });
                }
                other => {
                    self.store.config.rules.push(other);
                }
            }
        }

        for m in target.config.autoload_modules {
            self.store.add_autoload(&m)?;
        }

        self.store.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ks1_graceful_procfs_fallback() {
        let store = KernelModuleStore::new("test-store", "test store");
        let service = KernelModuleService::new(store)
            .with_proc_modules_path(PathBuf::from("/nonexistent/path/proc/modules"));

        let modules = service.list_loaded_modules().expect("must succeed with fallback");
        assert!(modules.is_empty());
    }

    #[test]
    fn test_ks2_precommit_conflict_detection() {
        let mut store = KernelModuleStore::new("test-store", "test store");
        store.add_autoload("overlay").expect("autoload should succeed");

        // Attempting to blacklist an autoloaded module must fail
        assert!(store.add_blacklist("overlay").is_err());

        let mut store2 = KernelModuleStore::new("test-store", "test store");
        store2.add_blacklist("cramfs").expect("blacklist should succeed");

        // Attempting to autoload a blacklisted module must fail
        assert!(store2.add_autoload("cramfs").is_err());
    }

    #[test]
    fn test_ks3_atomic_persistence() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store_path = dir.path().join("kernel_modules.json");

        let mut store = KernelModuleStore::new("persist-store", "persist test");
        store.add_blacklist("cramfs").expect("add blacklist");
        store.add_autoload("wireguard").expect("add autoload");

        store.save_to_path(&store_path).expect("save should succeed");
        assert!(store_path.exists());

        let loaded = KernelModuleStore::load_from_path(&store_path).expect("load should succeed");
        assert_eq!(loaded, store);
    }

    #[test]
    fn test_ks4_idempotent_mutations() {
        let mut store = KernelModuleStore::new("idempotent-store", "test store");
        store.add_blacklist("dccp").expect("first add");
        store.add_blacklist("dccp").expect("second add (idempotent)");

        let count = store.config.rules.iter().filter(|r| match r {
            ModprobeRule::Blacklist { module } => module == "dccp",
            _ => false,
        }).count();
        assert_eq!(count, 1, "rule should not be duplicated");

        // Update options
        store.add_options("ath9k_htc", vec!["nohwcrypt=1".to_string()]).expect("first opt");
        store.add_options("ath9k_htc", vec!["nohwcrypt=1".to_string(), "blink=1".to_string()]).expect("update opt");

        let opt_rules: Vec<_> = store.config.rules.iter().filter(|r| match r {
            ModprobeRule::Options { module, .. } => module == "ath9k_htc",
            _ => false,
        }).collect();
        assert_eq!(opt_rules.len(), 1, "options rule should be updated in-place");
    }

    #[test]
    fn test_ks5_preset_application_and_inspection() {
        let store = KernelModuleStore::new("preset-store", "test store");
        let mut service = KernelModuleService::new(store);

        service.apply_preset("cis_hardened_baseline").expect("apply CIS preset");
        assert!(service.store.config.rules.iter().any(|r| match r {
            ModprobeRule::Blacklist { module } => module == "cramfs",
            _ => false,
        }));

        service.apply_preset("container_isolation_baseline").expect("apply container preset");
        assert!(service.store.config.autoload_modules.contains(&"overlay".to_string()));
    }

    #[test]
    fn test_mock_proc_modules_inspection() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mock_file = dir.path().join("modules");
        fs::write(
            &mock_file,
            "overlay 151552 1 - Live 0x0000000000000000\ntun 61440 2 - Live 0x0000000000000000\n",
        ).expect("write mock");

        let store = KernelModuleStore::new("test", "test");
        let service = KernelModuleService::new(store).with_proc_modules_path(mock_file);

        let modules = service.list_loaded_modules().expect("list loaded");
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].name, "overlay");
        assert_eq!(modules[1].name, "tun");

        let tun = service.get_module("tun").expect("get module");
        assert!(tun.is_some());
        assert_eq!(tun.unwrap().ref_count, 2);
    }
}
