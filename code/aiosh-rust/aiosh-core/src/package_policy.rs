//! Security policy enforcement for AIOS Package Management Subsystem (PP1..PP6).
//!
//! Provides validation of package specifications, transaction plans, and store
//! contents against configurable security criteria and organizational invariants.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::package::{PackageFormat, PackageSpec, PackageTransaction};
use crate::package_service::PackageStore;

/// Maximum allowable size for a policy configuration file (64 KiB).
pub const MAX_POLICY_FILE_BYTES: u64 = 65_536;

/// Enforcement mode for package management security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackagePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

impl Default for PackagePolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Security policy defining mandatory security criteria for software packages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageSecurityPolicy {
    pub mode: PackagePolicyMode,
    pub prohibited_packages: Vec<String>,
    pub allowed_architectures: Vec<String>,
    pub allowed_formats: Vec<PackageFormat>,
    pub require_checksum: bool,
    pub require_https_or_file_repo: bool,
    pub max_package_size_bytes: u64,
    pub max_dependencies_per_package: usize,
    pub allowed_repositories: Vec<String>,
}

impl Default for PackageSecurityPolicy {
    fn default() -> Self {
        Self {
            mode: PackagePolicyMode::Enforcing,
            prohibited_packages: vec![
                "telnet".into(),
                "rsh-client".into(),
                "rsh-server".into(),
                "rlogin".into(),
                "rexec".into(),
                "nis".into(),
                "yp-tools".into(),
            ],
            allowed_architectures: vec![
                "x86_64".into(),
                "amd64".into(),
                "aarch64".into(),
                "riscv64".into(),
            ],
            allowed_formats: vec![
                PackageFormat::Deb,
                PackageFormat::Apk,
                PackageFormat::Flatpak,
                PackageFormat::Tarball,
            ],
            require_checksum: true,
            require_https_or_file_repo: true,
            max_package_size_bytes: 10 * 1024 * 1024 * 1024, // 10 GiB ceiling per package
            max_dependencies_per_package: 256,
            allowed_repositories: vec![
                "https://deb.debian.org/debian".into(),
                "https://security.debian.org/debian-security".into(),
                "https://dl-cdn.alpinelinux.org/alpine".into(),
            ],
        }
    }
}

/// A specific security violation found during package policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackagePolicyViolation {
    pub rule_id: String,
    pub package_name: String,
    pub description: String,
    pub fatal: bool,
}

/// Complete report of policy evaluation against a package, transaction, or store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackagePolicyVerdict {
    pub package_name: String,
    pub allowed: bool,
    pub mode: PackagePolicyMode,
    pub violations: Vec<PackagePolicyViolation>,
    pub evaluated_at: String,
}

impl PackageSecurityPolicy {
    /// Validates policy parameters and bounds (PP1).
    pub fn validate(&self) -> Result<(), String> {
        if self.allowed_architectures.is_empty() {
            return Err("invariant PP1 violated: allowed_architectures cannot be empty".into());
        }
        if self.allowed_architectures.len() > 64 {
            return Err("invariant PP1 violated: allowed_architectures exceeds limit of 64".into());
        }
        for arch in &self.allowed_architectures {
            if arch.is_empty() || arch.len() > 32 || arch.chars().any(|c| c.is_control()) {
                return Err(format!("invariant PP1 violated: invalid architecture '{}'", arch));
            }
        }

        if self.allowed_formats.is_empty() {
            return Err("invariant PP1 violated: allowed_formats cannot be empty".into());
        }

        if self.prohibited_packages.len() > 1024 {
            return Err("invariant PP1 violated: prohibited_packages exceeds limit of 1024".into());
        }
        for pkg in &self.prohibited_packages {
            if pkg.is_empty() || pkg.len() > 128 || pkg.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(format!("invariant PP1 violated: invalid prohibited package name '{}'", pkg));
            }
        }

        if self.max_package_size_bytes < 10 * 1024 || self.max_package_size_bytes > 100 * 1024 * 1024 * 1024 {
            return Err(format!(
                "invariant PP1 violated: max_package_size_bytes must be between 10 KiB and 100 GiB (got {})",
                self.max_package_size_bytes
            ));
        }

        if self.max_dependencies_per_package < 1 || self.max_dependencies_per_package > 1024 {
            return Err(format!(
                "invariant PP1 violated: max_dependencies_per_package must be between 1 and 1024 (got {})",
                self.max_dependencies_per_package
            ));
        }

        if self.allowed_repositories.len() > 256 {
            return Err("invariant PP1 violated: allowed_repositories exceeds limit of 256".into());
        }
        for repo in &self.allowed_repositories {
            if repo.len() > 1024 || repo.chars().any(|c| c.is_control()) {
                return Err(format!("invariant PP4 violated: invalid repository URL '{}'", repo));
            }
            if !repo.starts_with("https://") && !repo.starts_with("file://") {
                return Err(format!(
                    "invariant PP4 violated: allowed_repositories entry '{}' must start with https:// or file://",
                    repo
                ));
            }
        }

        Ok(())
    }

    /// Evaluates a single package specification against this security policy.
    pub fn evaluate_spec(&self, spec: &PackageSpec) -> PackagePolicyVerdict {
        let mut violations = Vec::new();

        // Control character hygiene check
        if spec.name.chars().any(|c| c.is_control()) {
            violations.push(PackagePolicyViolation {
                rule_id: "PP1-MALFORMED-NAME".into(),
                package_name: spec.name.clone(),
                description: "Package name contains prohibited control characters".into(),
                fatal: true,
            });
        }

        // PP2: Prohibited packages check
        for prohibited in &self.prohibited_packages {
            if prohibited.eq_ignore_ascii_case(&spec.name) {
                violations.push(PackagePolicyViolation {
                    rule_id: "PP2-PROHIBITED-PACKAGE".into(),
                    package_name: spec.name.clone(),
                    description: format!("Package '{}' is prohibited by security policy", spec.name),
                    fatal: true,
                });
            }
        }

        // PP3: Cryptographic checksum validation
        if self.require_checksum {
            match spec.sha256 {
                Some(ref hash) => {
                    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                        violations.push(PackagePolicyViolation {
                            rule_id: "PP3-INVALID-CHECKSUM".into(),
                            package_name: spec.name.clone(),
                            description: format!("Package '{}' has invalid SHA-256 checksum format (expected 64 hex characters)", spec.name),
                            fatal: true,
                        });
                    }
                }
                None => {
                    violations.push(PackagePolicyViolation {
                        rule_id: "PP3-MISSING-CHECKSUM".into(),
                        package_name: spec.name.clone(),
                        description: format!("Package '{}' lacks mandatory SHA-256 checksum", spec.name),
                        fatal: true,
                    });
                }
            }
        }

        // PP4: Transport protocol security
        if self.require_https_or_file_repo {
            if let Some(ref url) = spec.repository_url {
                if !url.starts_with("https://") && !url.starts_with("file://") {
                    violations.push(PackagePolicyViolation {
                        rule_id: "PP4-INSECURE-TRANSPORT".into(),
                        package_name: spec.name.clone(),
                        description: format!("Package '{}' repository URL '{}' is not HTTPS or file mirror", spec.name, url),
                        fatal: true,
                    });
                } else if !self.allowed_repositories.is_empty() {
                    let matched = self.allowed_repositories.iter().any(|allowed| url.starts_with(allowed));
                    if !matched {
                        violations.push(PackagePolicyViolation {
                            rule_id: "PP4-UNAUTHORIZED-REPO".into(),
                            package_name: spec.name.clone(),
                            description: format!("Package '{}' repository URL '{}' not in allowed list", spec.name, url),
                            fatal: false,
                        });
                    }
                }
            }
        }

        // PP5: Architecture, format, size, and dependency count hygiene
        if !self.allowed_architectures.iter().any(|a| a.eq_ignore_ascii_case(&spec.architecture)) {
            violations.push(PackagePolicyViolation {
                rule_id: "PP5-DISALLOWED-ARCH".into(),
                package_name: spec.name.clone(),
                description: format!("Package '{}' architecture '{}' is not in allowed architectures", spec.name, spec.architecture),
                fatal: true,
            });
        }

        if !self.allowed_formats.contains(&spec.format) {
            violations.push(PackagePolicyViolation {
                rule_id: "PP5-DISALLOWED-FORMAT".into(),
                package_name: spec.name.clone(),
                description: format!("Package '{}' format '{:?}' is not in allowed formats", spec.name, spec.format),
                fatal: true,
            });
        }

        if spec.installed_size_bytes > self.max_package_size_bytes {
            violations.push(PackagePolicyViolation {
                rule_id: "PP5-SIZE-EXCEEDED".into(),
                package_name: spec.name.clone(),
                description: format!(
                    "Package '{}' installed size {} exceeds maximum allowed {}",
                    spec.name, spec.installed_size_bytes, self.max_package_size_bytes
                ),
                fatal: true,
            });
        }

        if spec.dependencies.len() > self.max_dependencies_per_package {
            violations.push(PackagePolicyViolation {
                rule_id: "PP5-DEP-COUNT-EXCEEDED".into(),
                package_name: spec.name.clone(),
                description: format!(
                    "Package '{}' has {} dependencies exceeding limit {}",
                    spec.name, spec.dependencies.len(), self.max_dependencies_per_package
                ),
                fatal: false,
            });
        }

        // Compute allowed status under mode (PP6)
        let allowed = match self.mode {
            PackagePolicyMode::Enforcing => !violations.iter().any(|v| v.fatal),
            PackagePolicyMode::Audit => true,
            PackagePolicyMode::Permissive => !violations.iter().any(|v| v.rule_id == "PP2-PROHIBITED-PACKAGE"),
        };

        PackagePolicyVerdict {
            package_name: spec.name.clone(),
            allowed,
            mode: self.mode,
            violations,
            evaluated_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    /// Evaluates all packages targeted in a proposed transaction before execution (PP5).
    pub fn evaluate_transaction(
        &self,
        tx: &PackageTransaction,
        store: &PackageStore,
    ) -> PackagePolicyVerdict {
        let mut combined_violations = Vec::new();
        let mut all_allowed = true;

        for action in &tx.actions {
            if let Some(spec) = store.get_package(&action.package_name) {
                let verdict = self.evaluate_spec(spec);
                if !verdict.allowed {
                    all_allowed = false;
                }
                combined_violations.extend(verdict.violations);
            } else {
                combined_violations.push(PackagePolicyViolation {
                    rule_id: "PP5-PACKAGE-NOT-FOUND".into(),
                    package_name: action.package_name.clone(),
                    description: format!("Target transaction package '{}' not found in store", action.package_name),
                    fatal: true,
                });
                if self.mode == PackagePolicyMode::Enforcing {
                    all_allowed = false;
                }
            }
        }

        PackagePolicyVerdict {
            package_name: format!("transaction-{}", tx.id),
            allowed: all_allowed,
            mode: self.mode,
            violations: combined_violations,
            evaluated_at: "2026-09-04T00:00:00Z".into(),
        }
    }

    /// Evaluates an entire package store and reports verdicts for all registered packages.
    pub fn evaluate_store(&self, store: &PackageStore) -> Vec<PackagePolicyVerdict> {
        store
            .list_packages()
            .into_iter()
            .map(|pkg| self.evaluate_spec(pkg))
            .collect()
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

    /// Helper for environment variable reading.
    pub fn from_source<F: Fn(&str) -> Option<String>>(lookup: F) -> Result<Self, String> {
        let mut policy = Self::default();

        if let Some(val) = lookup("AIOS_PACKAGE_POLICY_MODE") {
            policy.mode = match val.to_lowercase().as_str() {
                "enforcing" => PackagePolicyMode::Enforcing,
                "audit" => PackagePolicyMode::Audit,
                "permissive" => PackagePolicyMode::Permissive,
                other => return Err(format!("unknown AIOS_PACKAGE_POLICY_MODE '{}'", other)),
            };
        }

        if let Some(val) = lookup("AIOS_PACKAGE_REQUIRE_CHECKSUM") {
            policy.require_checksum = val == "1" || val.eq_ignore_ascii_case("true");
        }

        if let Some(val) = lookup("AIOS_PACKAGE_REQUIRE_HTTPS") {
            policy.require_https_or_file_repo = val == "1" || val.eq_ignore_ascii_case("true");
        }

        if let Some(val) = lookup("AIOS_PACKAGE_MAX_SIZE_BYTES") {
            policy.max_package_size_bytes = val.parse::<u64>().map_err(|e| format!("invalid AIOS_PACKAGE_MAX_SIZE_BYTES: {}", e))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::*;

    fn dummy_spec(name: &str) -> PackageSpec {
        PackageSpec {
            name: name.into(),
            version: "1.0.0".into(),
            architecture: "amd64".into(),
            format: PackageFormat::Deb,
            state: PackageState::Installed,
            description: "Dummy test package".into(),
            installed_size_bytes: 1024,
            sha256: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into()),
            repository_url: Some("https://deb.debian.org/debian".into()),
            dependencies: vec![],
        }
    }

    #[test]
    fn test_policy_defaults_and_validation() {
        let policy = PackageSecurityPolicy::default();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.mode, PackagePolicyMode::Enforcing);
    }

    #[test]
    fn test_policy_prohibited_package_rejection() {
        let policy = PackageSecurityPolicy::default();
        let telnet_pkg = dummy_spec("telnet");
        let verdict = policy.evaluate_spec(&telnet_pkg);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "PP2-PROHIBITED-PACKAGE"));
    }

    #[test]
    fn test_policy_checksum_and_transport_enforcement() {
        let policy = PackageSecurityPolicy::default();
        let mut bad_pkg = dummy_spec("valid-app");
        bad_pkg.sha256 = None;
        bad_pkg.repository_url = Some("http://insecure.example.com/repo".into());

        let verdict = policy.evaluate_spec(&bad_pkg);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "PP3-MISSING-CHECKSUM"));
        assert!(verdict.violations.iter().any(|v| v.rule_id == "PP4-INSECURE-TRANSPORT"));
    }

    #[test]
    fn test_policy_audit_mode_non_blocking() {
        let mut policy = PackageSecurityPolicy::default();
        policy.mode = PackagePolicyMode::Audit;
        let telnet_pkg = dummy_spec("telnet");
        let verdict = policy.evaluate_spec(&telnet_pkg);
        // Under audit mode, allowed is true but violations are reported
        assert!(verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.rule_id == "PP2-PROHIBITED-PACKAGE"));
    }
}
