//! Package Management Core Service (CS1..CS5)
//!
//! Provides the core store registry, query engine, transaction planner, and
//! persistence engine for managing software packages on AIOS.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::package::{
    validate_package_spec, validate_package_transaction, PackageAction, PackageActionType,
    PackageDependency, PackageFormat, PackageQuery, PackageSpec, PackageState,
    PackageTransaction,
};

/// Summary report produced upon executing a package transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionReport {
    pub transaction_id: String,
    pub packages_installed: Vec<String>,
    pub packages_removed: Vec<String>,
    pub packages_upgraded: Vec<String>,
    pub total_size_delta_bytes: i64,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: String,
}

/// In-memory package repository and transaction planner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageStore {
    pub packages: HashMap<String, PackageSpec>,
}

impl Default for PackageStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageStore {
    /// Initializes a store seeded with canonical reference packages for Debian and Alpine.
    pub fn new() -> Self {
        let mut store = Self::empty();

        let debian_packages = vec![
            PackageSpec {
                name: "libc6".into(),
                version: "2.36-9+deb12u7".into(),
                architecture: "amd64".into(),
                format: PackageFormat::Deb,
                state: PackageState::Installed,
                description: "GNU C Library: Shared libraries".into(),
                installed_size_bytes: 12_582_912,
                sha256: Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into()),
                repository_url: Some("https://deb.debian.org/debian".into()),
                dependencies: vec![],
            },
            PackageSpec {
                name: "coreutils".into(),
                version: "9.1-1".into(),
                architecture: "amd64".into(),
                format: PackageFormat::Deb,
                state: PackageState::Installed,
                description: "GNU core utilities".into(),
                installed_size_bytes: 16_777_216,
                sha256: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into()),
                repository_url: Some("https://deb.debian.org/debian".into()),
                dependencies: vec![PackageDependency {
                    name: "libc6".into(),
                    version_constraint: Some(">= 2.36".into()),
                    optional: false,
                }],
            },
            PackageSpec {
                name: "bash".into(),
                version: "5.2.15-2+b2".into(),
                architecture: "amd64".into(),
                format: PackageFormat::Deb,
                state: PackageState::Installed,
                description: "GNU Bourne Again SHell".into(),
                installed_size_bytes: 6_291_456,
                sha256: Some("1111111111111111111111111111111111111111111111111111111111111111".into()),
                repository_url: Some("https://deb.debian.org/debian".into()),
                dependencies: vec![PackageDependency {
                    name: "libc6".into(),
                    version_constraint: Some(">= 2.36".into()),
                    optional: false,
                }],
            },
            PackageSpec {
                name: "libssl3".into(),
                version: "3.0.13-1~deb12u1".into(),
                architecture: "amd64".into(),
                format: PackageFormat::Deb,
                state: PackageState::Available,
                description: "Secure Sockets Layer toolkit - shared libraries".into(),
                installed_size_bytes: 5_242_880,
                sha256: Some("3333333333333333333333333333333333333333333333333333333333333333".into()),
                repository_url: Some("https://deb.debian.org/debian".into()),
                dependencies: vec![PackageDependency {
                    name: "libc6".into(),
                    version_constraint: Some(">= 2.36".into()),
                    optional: false,
                }],
            },
            PackageSpec {
                name: "curl".into(),
                version: "7.88.1-10+deb12u5".into(),
                architecture: "amd64".into(),
                format: PackageFormat::Deb,
                state: PackageState::Available,
                description: "command line tool for transferring data with URL syntax".into(),
                installed_size_bytes: 4_194_304,
                sha256: Some("2222222222222222222222222222222222222222222222222222222222222222".into()),
                repository_url: Some("https://deb.debian.org/debian".into()),
                dependencies: vec![
                    PackageDependency {
                        name: "libc6".into(),
                        version_constraint: Some(">= 2.36".into()),
                        optional: false,
                    },
                    PackageDependency {
                        name: "libssl3".into(),
                        version_constraint: Some(">= 3.0.0".into()),
                        optional: false,
                    },
                ],
            },
        ];

        let alpine_packages = vec![
            PackageSpec {
                name: "musl".into(),
                version: "1.2.4-r2".into(),
                architecture: "x86_64".into(),
                format: PackageFormat::Apk,
                state: PackageState::Installed,
                description: "the musl c library (libc) implementation".into(),
                installed_size_bytes: 629_145,
                sha256: Some("5555555555555555555555555555555555555555555555555555555555555555".into()),
                repository_url: Some("https://dl-cdn.alpinelinux.org/alpine/v3.19/main".into()),
                dependencies: vec![],
            },
            PackageSpec {
                name: "busybox".into(),
                version: "1.36.1-r15".into(),
                architecture: "x86_64".into(),
                format: PackageFormat::Apk,
                state: PackageState::Installed,
                description: "Size optimized toolbox of many common UNIX utilities".into(),
                installed_size_bytes: 1_048_576,
                sha256: Some("4444444444444444444444444444444444444444444444444444444444444444".into()),
                repository_url: Some("https://dl-cdn.alpinelinux.org/alpine/v3.19/main".into()),
                dependencies: vec![PackageDependency {
                    name: "musl".into(),
                    version_constraint: Some(">= 1.2.4".into()),
                    optional: false,
                }],
            },
            PackageSpec {
                name: "apk-tools".into(),
                version: "2.14.0-r5".into(),
                architecture: "x86_64".into(),
                format: PackageFormat::Apk,
                state: PackageState::Installed,
                description: "Alpine Package Keeper - package management tools".into(),
                installed_size_bytes: 524_288,
                sha256: Some("6666666666666666666666666666666666666666666666666666666666666666".into()),
                repository_url: Some("https://dl-cdn.alpinelinux.org/alpine/v3.19/main".into()),
                dependencies: vec![PackageDependency {
                    name: "musl".into(),
                    version_constraint: Some(">= 1.2.4".into()),
                    optional: false,
                }],
            },
        ];

        for spec in debian_packages.into_iter().chain(alpine_packages) {
            let _ = store.register_package(spec);
        }

        store
    }

    /// Initializes an empty store.
    pub fn empty() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Returns all registered packages sorted by package name.
    pub fn list_packages(&self) -> Vec<&PackageSpec> {
        let mut list: Vec<&PackageSpec> = self.packages.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// Retrieves a package by name if it exists in the store.
    pub fn get_package(&self, name: &str) -> Option<&PackageSpec> {
        self.packages.get(name)
    }

    /// Registers a new package specification into the store (CS1).
    pub fn register_package(&mut self, spec: PackageSpec) -> Result<(), String> {
        validate_package_spec(&spec).map_err(|errs| errs.join("; "))?;
        if self.packages.contains_key(&spec.name) {
            return Err(format!(
                "invariant CS1 violated: package '{}' is already registered",
                spec.name
            ));
        }
        self.packages.insert(spec.name.clone(), spec);
        Ok(())
    }

    /// Unregisters an existing package specification by name.
    pub fn unregister_package(&mut self, name: &str) -> Result<PackageSpec, String> {
        self.packages
            .remove(name)
            .ok_or_else(|| format!("package '{}' not found in store", name))
    }

    /// Queries packages matching filter criteria.
    pub fn query(&self, query: &PackageQuery) -> Vec<&PackageSpec> {
        let mut results: Vec<&PackageSpec> = self
            .list_packages()
            .into_iter()
            .filter(|pkg| {
                if let Some(ref pattern) = query.name_pattern {
                    let pat = pattern.to_lowercase();
                    if !pkg.name.to_lowercase().contains(&pat) {
                        return false;
                    }
                }
                if let Some(ref fmt) = query.format {
                    if pkg.format != *fmt {
                        return false;
                    }
                }
                if let Some(ref st) = query.state {
                    if pkg.state != *st {
                        return false;
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

    /// Plans and validates a transaction given a set of package actions (CS2, CS3, CS4).
    pub fn plan_transaction(
        &self,
        actions: Vec<PackageAction>,
        dry_run: bool,
    ) -> Result<PackageTransaction, String> {
        if actions.is_empty() {
            return Err("transaction actions list cannot be empty".into());
        }
        if actions.len() > 256 {
            return Err(format!(
                "transaction actions list exceeds 256 entries (was {})",
                actions.len()
            ));
        }

        // Verify all target packages exist in store
        for act in &actions {
            if !self.packages.contains_key(&act.package_name) {
                return Err(format!(
                    "target package '{}' not found in store",
                    act.package_name
                ));
            }
        }

        // Verify dependency closure (CS3)
        for act in &actions {
            if matches!(act.action, PackageActionType::Install | PackageActionType::Upgrade) {
                let pkg = self.packages.get(&act.package_name).unwrap();
                for dep in &pkg.dependencies {
                    if dep.optional {
                        continue;
                    }
                    let dep_installed = self
                        .packages
                        .get(&dep.name)
                        .map_or(false, |p| p.state == PackageState::Installed);
                    let dep_in_actions = actions.iter().any(|a| {
                        a.package_name == dep.name
                            && matches!(a.action, PackageActionType::Install | PackageActionType::Upgrade)
                    });
                    if !dep_installed && !dep_in_actions {
                        return Err(format!(
                            "invariant CS3 violated: unmet dependency: package '{}' requires '{}'",
                            pkg.name, dep.name
                        ));
                    }
                }
            }
        }

        // Calculate size delta (CS4)
        let mut total_size_delta_bytes: i64 = 0;
        for act in &actions {
            let pkg = self.packages.get(&act.package_name).unwrap();
            match act.action {
                PackageActionType::Install => {
                    if pkg.state != PackageState::Installed {
                        total_size_delta_bytes += pkg.installed_size_bytes as i64;
                    }
                }
                PackageActionType::Remove | PackageActionType::Purge => {
                    if pkg.state == PackageState::Installed {
                        total_size_delta_bytes -= pkg.installed_size_bytes as i64;
                    }
                }
                PackageActionType::Upgrade => {
                    // Upgrading to existing in-store spec yields zero delta
                }
            }
        }

        // Deterministic transaction ID (CS2)
        let mut hasher = Sha256::new();
        for act in &actions {
            hasher.update(act.package_name.as_bytes());
            hasher.update(format!("{:?}", act.action).as_bytes());
            if let Some(ref ver) = act.target_version {
                hasher.update(ver.as_bytes());
            }
        }
        hasher.update(&total_size_delta_bytes.to_le_bytes());
        let hash_hex = format!("{:x}", hasher.finalize());
        let tx_id = format!("tx-{}", &hash_hex[..16]);

        let tx = PackageTransaction {
            id: tx_id,
            created_at: "2026-01-01T00:00:00Z".into(),
            actions,
            dry_run,
            total_size_delta_bytes,
        };

        validate_package_transaction(&tx).map_err(|errs| errs.join("; "))?;
        Ok(tx)
    }

    /// Executes or dry-runs a validated package transaction against the store (CS2..CS4).
    pub fn execute_transaction(
        &mut self,
        tx: &PackageTransaction,
    ) -> Result<TransactionReport, String> {
        validate_package_transaction(tx).map_err(|errs| errs.join("; "))?;

        // Verify target packages exist in store
        for act in &tx.actions {
            if !self.packages.contains_key(&act.package_name) {
                return Err(format!(
                    "target package '{}' not found in store",
                    act.package_name
                ));
            }
        }

        // Verify dependency closure (CS3)
        for act in &tx.actions {
            if matches!(act.action, PackageActionType::Install | PackageActionType::Upgrade) {
                let pkg = self.packages.get(&act.package_name).unwrap();
                for dep in &pkg.dependencies {
                    if dep.optional {
                        continue;
                    }
                    let dep_installed = self
                        .packages
                        .get(&dep.name)
                        .map_or(false, |p| p.state == PackageState::Installed);
                    let dep_in_actions = tx.actions.iter().any(|a| {
                        a.package_name == dep.name
                            && matches!(a.action, PackageActionType::Install | PackageActionType::Upgrade)
                    });
                    if !dep_installed && !dep_in_actions {
                        return Err(format!(
                            "invariant CS3 violated: unmet dependency: package '{}' requires '{}'",
                            pkg.name, dep.name
                        ));
                    }
                }
            }
        }

        // Verify size delta arithmetic (CS4)
        let mut calculated_delta: i64 = 0;
        for act in &tx.actions {
            let pkg = self.packages.get(&act.package_name).unwrap();
            match act.action {
                PackageActionType::Install => {
                    if pkg.state != PackageState::Installed {
                        calculated_delta += pkg.installed_size_bytes as i64;
                    }
                }
                PackageActionType::Remove | PackageActionType::Purge => {
                    if pkg.state == PackageState::Installed {
                        calculated_delta -= pkg.installed_size_bytes as i64;
                    }
                }
                PackageActionType::Upgrade => {}
            }
        }

        if tx.total_size_delta_bytes != calculated_delta {
            return Err(format!(
                "invariant CS4 violated: transaction delta ({}) != calculated delta ({})",
                tx.total_size_delta_bytes, calculated_delta
            ));
        }

        let mut packages_installed = Vec::new();
        let mut packages_removed = Vec::new();
        let mut packages_upgraded = Vec::new();

        for act in &tx.actions {
            match act.action {
                PackageActionType::Install => {
                    packages_installed.push(act.package_name.clone());
                    if !tx.dry_run {
                        if let Some(pkg) = self.packages.get_mut(&act.package_name) {
                            pkg.state = PackageState::Installed;
                        }
                    }
                }
                PackageActionType::Remove | PackageActionType::Purge => {
                    packages_removed.push(act.package_name.clone());
                    if !tx.dry_run {
                        if let Some(pkg) = self.packages.get_mut(&act.package_name) {
                            pkg.state = PackageState::Available;
                        }
                    }
                }
                PackageActionType::Upgrade => {
                    packages_upgraded.push(act.package_name.clone());
                    if !tx.dry_run {
                        if let Some(pkg) = self.packages.get_mut(&act.package_name) {
                            pkg.state = PackageState::Installed;
                        }
                    }
                }
            }
        }

        Ok(TransactionReport {
            transaction_id: tx.id.clone(),
            packages_installed,
            packages_removed,
            packages_upgraded,
            total_size_delta_bytes: tx.total_size_delta_bytes,
            success: true,
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Atomically persists store contents to disk (CS5).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.packages)
            .map_err(|e| format!("failed to serialize package store: {}", e))?;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp_path = path.with_extension("tmp");
        if let Err(e) = std::fs::write(&tmp_path, content.as_bytes()) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("failed to write temp package store: {}", e));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o644));
        }

        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("failed to persist package store to '{}': {}", path.display(), e));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644));
        }

        Ok(())
    }

    /// Loads and validates a store from disk bounded to 10 MiB (CS5).
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)
            .map_err(|e| format!("failed to open package store at '{}': {}", path.display(), e))?;

        let meta = file
            .metadata()
            .map_err(|e| format!("failed to read metadata for '{}': {}", path.display(), e))?;
        if meta.len() > 10 * 1024 * 1024 {
            return Err(format!(
                "package store at '{}' exceeds maximum allowed size of 10 MiB (was {} bytes)",
                path.display(),
                meta.len()
            ));
        }

        let mut bytes = Vec::new();
        file.by_ref()
            .take(10 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("failed to read package store from '{}': {}", path.display(), e))?;

        if bytes.len() > 10 * 1024 * 1024 {
            return Err("package store content exceeded 10 MiB during stream read".into());
        }

        let packages: HashMap<String, PackageSpec> = serde_json::from_slice(&bytes)
            .map_err(|e| format!("failed to parse package store at '{}': {}", path.display(), e))?;

        if packages.len() > 10_000 {
            return Err(format!(
                "package store at '{}' contains {} packages, exceeding maximum allowed limit of 10,000",
                path.display(),
                packages.len()
            ));
        }

        for (key, spec) in &packages {
            if key != &spec.name {
                return Err(format!(
                    "invariant CS1 violated: store key '{}' does not match package name '{}'",
                    key, spec.name
                ));
            }
            validate_package_spec(spec)
                .map_err(|errs| format!("invalid package '{}': {}", spec.name, errs.join("; ")))?;
        }

        Ok(Self { packages })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_store_new_seeded() {
        let store = PackageStore::new();
        assert_eq!(store.packages.len(), 8);
        assert!(store.get_package("libc6").is_some());
        assert!(store.get_package("busybox").is_some());
        assert_eq!(store.list_packages().len(), 8);
    }

    #[test]
    fn test_package_store_cs1_uniqueness() {
        let mut store = PackageStore::empty();
        let spec = PackageSpec {
            name: "musl".into(),
            version: "1.2.4-r2".into(),
            architecture: "x86_64".into(),
            format: PackageFormat::Apk,
            state: PackageState::Installed,
            description: "musl libc".into(),
            installed_size_bytes: 629_145,
            sha256: Some("5555555555555555555555555555555555555555555555555555555555555555".into()),
            repository_url: Some("https://dl-cdn.alpinelinux.org/alpine/v3.19/main".into()),
            dependencies: vec![],
        };
        assert!(store.register_package(spec.clone()).is_ok());
        let err = store.register_package(spec).unwrap_err();
        assert!(err.contains("CS1"));
    }

    #[test]
    fn test_package_store_query() {
        let store = PackageStore::new();
        let deb_query = PackageQuery {
            name_pattern: None,
            format: Some(PackageFormat::Deb),
            state: None,
            limit: None,
        };
        let deb_pkgs = store.query(&deb_query);
        assert_eq!(deb_pkgs.len(), 5);

        let query_installed = PackageQuery {
            name_pattern: Some("busy".into()),
            format: None,
            state: Some(PackageState::Installed),
            limit: Some(10),
        };
        let pkgs = store.query(&query_installed);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].name, "busybox");
    }

    #[test]
    fn test_package_store_plan_and_execute_transaction() {
        let mut store = PackageStore::new();
        let actions = vec![
            PackageAction {
                action: PackageActionType::Install,
                package_name: "libssl3".into(),
                target_version: None,
            },
            PackageAction {
                action: PackageActionType::Install,
                package_name: "curl".into(),
                target_version: None,
            },
        ];

        // CS2 Determinism
        let plan1 = store.plan_transaction(actions.clone(), false).unwrap();
        let plan2 = store.plan_transaction(actions.clone(), false).unwrap();
        assert_eq!(plan1, plan2);

        // CS4 Delta arithmetic
        let expected_delta = 5_242_880 + 4_194_304;
        assert_eq!(plan1.total_size_delta_bytes, expected_delta);

        // Dry run
        let mut dry_tx = plan1.clone();
        dry_tx.dry_run = true;
        let dry_rep = store.execute_transaction(&dry_tx).unwrap();
        assert!(dry_rep.success);
        assert_eq!(store.get_package("curl").unwrap().state, PackageState::Available);

        // Actual execution
        let rep = store.execute_transaction(&plan1).unwrap();
        assert!(rep.success);
        assert_eq!(store.get_package("curl").unwrap().state, PackageState::Installed);
        assert_eq!(store.get_package("libssl3").unwrap().state, PackageState::Installed);
    }

    #[test]
    fn test_package_store_cs3_missing_dependency() {
        let store = PackageStore::new();
        let actions = vec![PackageAction {
            action: PackageActionType::Install,
            package_name: "curl".into(),
            target_version: None,
        }];

        // Fails because libssl3 is not installed and not in actions batch
        let err = store.plan_transaction(actions, false).unwrap_err();
        assert!(err.contains("CS3"));
        assert!(err.contains("libssl3"));
    }

    #[test]
    fn test_package_store_cs5_persistence_roundtrip() {
        let store = PackageStore::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let store_file = temp_dir.path().join("packages.json");

        assert!(store.save_to_path(&store_file).is_ok());
        let loaded = PackageStore::load_from_path(&store_file).unwrap();
        assert_eq!(loaded.packages.len(), store.packages.len());
        assert_eq!(
            loaded.get_package("coreutils").unwrap().version,
            store.get_package("coreutils").unwrap().version
        );
    }

    #[test]
    fn test_package_store_hardening_bounds_and_cleanup() {
        let store = PackageStore::new();

        // 1. Actions boundary checks
        let empty_actions: Vec<PackageAction> = vec![];
        let err_empty = store.plan_transaction(empty_actions, false).unwrap_err();
        assert!(err_empty.contains("cannot be empty"));

        let huge_actions: Vec<PackageAction> = (0..257)
            .map(|i| PackageAction {
                action: PackageActionType::Install,
                package_name: format!("pkg-{}", i),
                target_version: None,
            })
            .collect();
        let err_huge = store.plan_transaction(huge_actions, false).unwrap_err();
        assert!(err_huge.contains("exceeds 256"));

        // 2. Temp file cleanup on persistence failure
        let temp_dir = tempfile::tempdir().unwrap();
        // Point to a file whose parent path is an unwritable/unusable file path rather than a directory
        let dummy_file = temp_dir.path().join("dummy_file");
        std::fs::write(&dummy_file, b"not a dir").unwrap();
        let impossible_target = dummy_file.join("sub").join("packages.json");
        let err_save = store.save_to_path(&impossible_target);
        assert!(err_save.is_err());
        assert!(!impossible_target.with_extension("tmp").exists());
    }
}
