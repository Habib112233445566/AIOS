//! Package Management Data Model (PM1..PM5)
//!
//! Provides core data structures and validation logic for AIOS package management across
//! supported distribution targets (Debian/APT, Alpine/APK, etc.).

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Target package format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageFormat {
    Deb,
    Apk,
    Flatpak,
    Tarball,
}

/// Lifecycle state of a package on the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageState {
    Available,
    Installed,
    Upgradable,
    PendingInstall,
    PendingRemoval,
    Broken,
}

/// Package dependency definition with optional version constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub version_constraint: Option<String>,
    pub optional: bool,
}

/// Comprehensive specification of a software package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSpec {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub format: PackageFormat,
    pub state: PackageState,
    pub description: String,
    pub installed_size_bytes: u64,
    pub sha256: Option<String>,
    pub repository_url: Option<String>,
    pub dependencies: Vec<PackageDependency>,
}

/// Action types supported in a package transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageActionType {
    Install,
    Remove,
    Upgrade,
    Purge,
}

/// Single action within a package transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageAction {
    pub action: PackageActionType,
    pub package_name: String,
    pub target_version: Option<String>,
}

/// Atomic transaction consisting of multiple package actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTransaction {
    pub id: String,
    pub created_at: String,
    pub actions: Vec<PackageAction>,
    pub dry_run: bool,
    pub total_size_delta_bytes: i64,
}

/// Query filter for searching packages.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageQuery {
    pub name_pattern: Option<String>,
    pub format: Option<PackageFormat>,
    pub state: Option<PackageState>,
    pub limit: Option<usize>,
}

/// Validates package name syntax against upstream Debian and Alpine standards (PM1).
pub fn validate_package_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("package name cannot be empty".into());
    }
    if name.len() > 128 {
        return Err(format!("package name exceeds 128 characters (was {})", name.len()));
    }
    let bytes = name.as_bytes();
    let first = bytes[0];
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(format!(
            "package name must start with a lowercase alphanumeric character: '{}'",
            name
        ));
    }
    for &b in bytes {
        let is_valid = b.is_ascii_lowercase()
            || b.is_ascii_digit()
            || b == b'+'
            || b == b'-'
            || b == b'.';
        if !is_valid {
            return Err(format!(
                "package name contains invalid character '{}' in '{}'",
                b as char, name
            ));
        }
    }
    Ok(())
}

/// Validates a package specification against PM1..PM5 invariants.
pub fn validate_package_spec(spec: &PackageSpec) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // PM1: Naming syntax
    if let Err(err) = validate_package_name(&spec.name) {
        errors.push(err);
    }

    // PM2: Bounds & lengths
    if spec.version.is_empty() {
        errors.push("package version cannot be empty".into());
    } else if spec.version.len() > 64 {
        errors.push(format!("package version exceeds 64 characters (was {})", spec.version.len()));
    } else if spec.version.chars().any(|c| c.is_control()) {
        errors.push("package version contains illegal control characters".into());
    }

    if spec.architecture.is_empty() {
        errors.push("package architecture cannot be empty".into());
    } else if spec.architecture.chars().any(|c| c.is_control() || !c.is_ascii_graphic()) {
        errors.push("package architecture contains invalid characters".into());
    }

    if spec.description.len() > 4096 {
        errors.push(format!(
            "package description exceeds 4096 bytes (was {})",
            spec.description.len()
        ));
    }

    const MAX_PKG_SIZE: u64 = 100 * 1024 * 1024 * 1024; // 100 GiB
    if spec.installed_size_bytes > MAX_PKG_SIZE {
        errors.push(format!(
            "installed_size_bytes exceeds 100 GiB ceiling (was {})",
            spec.installed_size_bytes
        ));
    }

    if spec.dependencies.len() > 256 {
        errors.push(format!(
            "dependencies list exceeds 256 items (was {})",
            spec.dependencies.len()
        ));
    }

    // PM3: Dependency hygiene
    let mut seen_deps = HashSet::new();
    for dep in &spec.dependencies {
        if dep.name == spec.name {
            errors.push(format!("package cannot depend on itself: '{}'", dep.name));
        }
        if !seen_deps.insert(&dep.name) {
            errors.push(format!("duplicate dependency detected: '{}'", dep.name));
        }
        if let Err(err) = validate_package_name(&dep.name) {
            errors.push(format!("invalid dependency name: {}", err));
        }
        if let Some(ref vc) = dep.version_constraint {
            if vc.len() > 64 {
                errors.push(format!(
                    "version constraint for '{}' exceeds 64 characters",
                    dep.name
                ));
            } else if vc.chars().any(|c| c.is_control()) {
                errors.push(format!(
                    "version constraint for '{}' contains control characters",
                    dep.name
                ));
            }
        }
    }

    // PM4: Checksum and provenance
    if let Some(ref sha) = spec.sha256 {
        if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
            errors.push(format!(
                "invalid sha256 checksum format (must be 64 hex characters): '{}'",
                sha
            ));
        }
    }

    if let Some(ref url) = spec.repository_url {
        if url.is_empty() {
            errors.push("repository_url cannot be empty string".into());
        } else if url.len() > 2048 {
            errors.push(format!("repository_url exceeds 2048 characters (was {})", url.len()));
        } else if url.chars().any(|c| c.is_control()) {
            errors.push("repository_url contains illegal control characters".into());
        } else {
            let is_secure = url.starts_with("https://")
                || url.starts_with("http://127.0.0.1")
                || url.starts_with("http://localhost");
            if !is_secure {
                errors.push(format!(
                    "repository_url must use secure HTTPS protocol (was '{}')",
                    url
                ));
            }
        }
    }

    // PM5: State consistency
    if spec.state == PackageState::Installed && spec.installed_size_bytes == 0 {
        errors.push("installed package must have positive installed_size_bytes > 0".into());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates a package transaction for internal consistency and sanity.
pub fn validate_package_transaction(tx: &PackageTransaction) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if tx.id.is_empty() {
        errors.push("transaction id cannot be empty".into());
    } else if tx.id.len() > 64 {
        errors.push(format!("transaction id exceeds 64 characters (was {})", tx.id.len()));
    } else if tx.id.chars().any(|c| !c.is_ascii_graphic()) {
        errors.push("transaction id must contain only printable graphic ASCII characters".into());
    }

    if DateTime::parse_from_rfc3339(&tx.created_at).is_err() {
        errors.push(format!(
            "created_at must be valid RFC 3339 timestamp (was '{}')",
            tx.created_at
        ));
    }

    if tx.actions.is_empty() {
        errors.push("transaction actions list cannot be empty".into());
    } else if tx.actions.len() > 256 {
        errors.push(format!(
            "transaction actions list exceeds 256 entries (was {})",
            tx.actions.len()
        ));
    }

    let mut seen_packages: HashSet<String> = HashSet::new();
    for action in &tx.actions {
        if let Err(err) = validate_package_name(&action.package_name) {
            errors.push(format!("action target: {}", err));
        }

        if !seen_packages.insert(action.package_name.clone()) {
            errors.push(format!(
                "multiple conflicting actions targeted at package '{}' in single transaction",
                action.package_name
            ));
        }

        if let Some(ref ver) = action.target_version {
            if ver.is_empty() {
                errors.push(format!(
                    "target_version for package '{}' cannot be empty string",
                    action.package_name
                ));
            } else if ver.len() > 64 {
                errors.push(format!(
                    "target_version for package '{}' exceeds 64 characters",
                    action.package_name
                ));
            } else if ver.chars().any(|c| c.is_control()) {
                errors.push(format!(
                    "target_version for package '{}' contains control characters",
                    action.package_name
                ));
            }
        }
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

    #[test]
    fn test_validate_package_name() {
        assert!(validate_package_name("curl").is_ok());
        assert!(validate_package_name("libc6").is_ok());
        assert!(validate_package_name("libssl3").is_ok());
        assert!(validate_package_name("g++").is_ok());
        assert!(validate_package_name("python3.11").is_ok());

        // Negative cases
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name("Curl").is_err()); // uppercase
        assert!(validate_package_name("-bad").is_err()); // leading hyphen
        assert!(validate_package_name("+bad").is_err()); // leading plus
        assert!(validate_package_name(".bad").is_err()); // leading dot
        assert!(validate_package_name("bad name").is_err()); // space
        assert!(validate_package_name("bad/name").is_err()); // slash
        assert!(validate_package_name("bad\0name").is_err()); // null byte
        assert!(validate_package_name(&"a".repeat(129)).is_err()); // too long
    }

    #[test]
    fn test_validate_package_spec_valid() {
        let spec = PackageSpec {
            name: "curl".into(),
            version: "8.5.0-2".into(),
            architecture: "x86_64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Installed,
            description: "command line tool for transferring data with URL syntax".into(),
            installed_size_bytes: 450_000,
            sha256: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into()),
            repository_url: Some("https://deb.debian.org/debian".into()),
            dependencies: vec![PackageDependency {
                name: "libc6".into(),
                version_constraint: Some(">= 2.34".into()),
                optional: false,
            }],
        };
        assert!(validate_package_spec(&spec).is_ok());
    }

    #[test]
    fn test_validate_package_spec_pm1_to_pm5_rejections() {
        // Self-dependency
        let self_dep_spec = PackageSpec {
            name: "curl".into(),
            version: "8.5.0-2".into(),
            architecture: "x86_64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "curl".into(),
            installed_size_bytes: 1000,
            sha256: None,
            repository_url: None,
            dependencies: vec![PackageDependency {
                name: "curl".into(),
                version_constraint: None,
                optional: false,
            }],
        };
        let errs = validate_package_spec(&self_dep_spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("cannot depend on itself")));

        // Insecure repository URL
        let insecure_url_spec = PackageSpec {
            name: "curl".into(),
            version: "8.5.0-2".into(),
            architecture: "x86_64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "curl".into(),
            installed_size_bytes: 1000,
            sha256: None,
            repository_url: Some("http://insecure-repo.org".into()),
            dependencies: vec![],
        };
        let errs = validate_package_spec(&insecure_url_spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("must use secure HTTPS protocol")));

        // Installed package with 0 installed size
        let zero_size_installed = PackageSpec {
            name: "curl".into(),
            version: "8.5.0-2".into(),
            architecture: "x86_64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Installed,
            description: "curl".into(),
            installed_size_bytes: 0,
            sha256: None,
            repository_url: None,
            dependencies: vec![],
        };
        let errs = validate_package_spec(&zero_size_installed).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("installed package must have positive installed_size_bytes")));

        // Invalid sha256 checksum
        let bad_sha_spec = PackageSpec {
            name: "curl".into(),
            version: "8.5.0-2".into(),
            architecture: "x86_64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Available,
            description: "curl".into(),
            installed_size_bytes: 1000,
            sha256: Some("not-a-valid-sha256".into()),
            repository_url: None,
            dependencies: vec![],
        };
        let errs = validate_package_spec(&bad_sha_spec).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("invalid sha256 checksum format")));
    }

    #[test]
    fn test_validate_package_transaction() {
        let valid_tx = PackageTransaction {
            id: "tx-2026-09-04-001".into(),
            created_at: "2026-09-04T07:00:00Z".into(),
            actions: vec![PackageAction {
                action: PackageActionType::Install,
                package_name: "curl".into(),
                target_version: Some("8.5.0-2".into()),
            }],
            dry_run: true,
            total_size_delta_bytes: 450_000,
        };
        assert!(validate_package_transaction(&valid_tx).is_ok());

        // Conflicting duplicate target package in single transaction
        let conflicting_tx = PackageTransaction {
            id: "tx-002".into(),
            created_at: "2026-09-04T07:00:00Z".into(),
            actions: vec![
                PackageAction {
                    action: PackageActionType::Install,
                    package_name: "curl".into(),
                    target_version: None,
                },
                PackageAction {
                    action: PackageActionType::Remove,
                    package_name: "curl".into(),
                    target_version: None,
                },
            ],
            dry_run: false,
            total_size_delta_bytes: 0,
        };
        let errs = validate_package_transaction(&conflicting_tx).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("multiple conflicting actions")));

        // Invalid timestamp
        let bad_time_tx = PackageTransaction {
            id: "tx-003".into(),
            created_at: "not-a-valid-timestamp".into(),
            actions: vec![PackageAction {
                action: PackageActionType::Install,
                package_name: "curl".into(),
                target_version: None,
            }],
            dry_run: false,
            total_size_delta_bytes: 100,
        };
        let errs = validate_package_transaction(&bad_time_tx).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("valid RFC 3339 timestamp")));
    }
}
