//! Network Bootstrap Security Policy Subsystem (NPOL1..NPOL6).
//!
//! Provides validation and sanitization of network states against
//! security policies, promiscuous mode detection, interface/DNS allowlists,
//! and resource quota enforcement.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::network::{InterfaceType, NetworkState};

/// Enforcement mode for network security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

impl Default for NetworkPolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Security policy defining mandatory criteria and validation rules for host networking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkSecurityPolicy {
    pub mode: NetworkPolicyMode,
    pub disallowed_interface_types: Vec<InterfaceType>,
    pub prohibited_interface_names: Vec<String>,
    pub allowed_interface_names: Option<Vec<String>>,
    pub allow_promiscuous: bool,
    pub require_mac_for_ethernet: bool,
    pub disallowed_dns_servers: Vec<String>,
    pub allowed_dns_servers: Option<Vec<String>>,
    pub max_interfaces_allowed: usize,
    pub max_routes_allowed: usize,
    pub max_dns_servers_allowed: usize,
    pub redact_sensitive_addresses: bool,
}

impl Default for NetworkSecurityPolicy {
    fn default() -> Self {
        Self {
            mode: NetworkPolicyMode::Enforcing,
            disallowed_interface_types: vec![InterfaceType::Other],
            prohibited_interface_names: Vec::new(),
            allowed_interface_names: None,
            allow_promiscuous: false,
            require_mac_for_ethernet: true,
            disallowed_dns_servers: Vec::new(),
            allowed_dns_servers: None,
            max_interfaces_allowed: 1024,
            max_routes_allowed: 4096,
            max_dns_servers_allowed: 32,
            redact_sensitive_addresses: false,
        }
    }
}

/// A specific security policy violation recorded against a network component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkPolicyViolation {
    pub rule_id: String,
    pub target: String,
    pub description: String,
    pub fatal: bool,
}

/// Report summarizing policy evaluation against a host network state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkPolicyReport {
    pub verdict: String, // "allow", "deny", "audit"
    pub mode: NetworkPolicyMode,
    pub violations: Vec<NetworkPolicyViolation>,
    pub interfaces_evaluated: usize,
    pub routes_evaluated: usize,
    pub dns_servers_evaluated: usize,
    pub redacted: bool,
}

/// Maximum allowed policy file size (1 MB) to prevent OOM / DoS.
pub const MAX_POLICY_FILE_BYTES: u64 = 1_048_576;

/// Validates policy file path hygiene (no traversal, no control chars, max length 1024).
pub fn validate_policy_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_str().ok_or_else(|| "NPOL6 violation: policy path must be valid UTF-8".to_string())?;
    if path_str.trim().is_empty() {
        return Err("NPOL6 violation: policy path cannot be empty".into());
    }
    if path_str.len() > 1024 {
        return Err("NPOL6 violation: policy path exceeds maximum length of 1024 characters".into());
    }
    if path_str.chars().any(|c| c.is_control() || c == '\0') {
        return Err("NPOL6 violation: policy path cannot contain control characters".into());
    }
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("NPOL6 violation: policy path traversal ('..') is not permitted".into());
    }
    Ok(())
}

impl NetworkSecurityPolicy {
    /// Validates policy configuration invariants (NPOL4, NPOL6).
    pub fn validate(&self) -> Result<(), String> {
        if self.max_interfaces_allowed == 0 || self.max_interfaces_allowed > 10_000 {
            return Err(format!(
                "NPOL4 violation: max_interfaces_allowed must be between 1 and 10,000 (got {})",
                self.max_interfaces_allowed
            ));
        }
        if self.max_routes_allowed == 0 || self.max_routes_allowed > 50_000 {
            return Err(format!(
                "NPOL4 violation: max_routes_allowed must be between 1 and 50,000 (got {})",
                self.max_routes_allowed
            ));
        }
        if self.max_dns_servers_allowed == 0 || self.max_dns_servers_allowed > 64 {
            return Err(format!(
                "NPOL4 violation: max_dns_servers_allowed must be between 1 and 64 (got {})",
                self.max_dns_servers_allowed
            ));
        }
        if self.prohibited_interface_names.len() > 1_000 {
            return Err(format!(
                "NPOL1 violation: prohibited_interface_names count {} exceeds limit of 1,000",
                self.prohibited_interface_names.len()
            ));
        }
        for name in &self.prohibited_interface_names {
            if name.trim().is_empty() {
                return Err("NPOL1 violation: prohibited_interface_names cannot contain empty strings".into());
            }
            if name.len() > 15 {
                return Err(format!("NPOL1 violation: prohibited interface name '{}' exceeds 15 characters", name));
            }
        }
        if let Some(ref names) = self.allowed_interface_names {
            if names.len() > 1_000 {
                return Err(format!(
                    "NPOL1 violation: allowed_interface_names count {} exceeds limit of 1,000",
                    names.len()
                ));
            }
            for name in names {
                if name.trim().is_empty() {
                    return Err("NPOL1 violation: allowed_interface_names cannot contain empty strings".into());
                }
                if name.len() > 15 {
                    return Err(format!("NPOL1 violation: allowed interface name '{}' exceeds 15 characters", name));
                }
            }
        }
        Ok(())
    }

    /// Evaluates a host network state snapshot against the security policy (NPOL1..NPOL5).
    pub fn evaluate(&self, state: &NetworkState) -> NetworkPolicyReport {
        let mut violations = Vec::new();

        // NPOL4: Capacity limit evaluation
        if state.interfaces.len() > self.max_interfaces_allowed {
            violations.push(NetworkPolicyViolation {
                rule_id: "RULE_IFACE_MAX_CAP".into(),
                target: "global:interfaces".into(),
                description: format!(
                    "Interface count ({}) exceeds allowed maximum ({})",
                    state.interfaces.len(),
                    self.max_interfaces_allowed
                ),
                fatal: true,
            });
        }
        if state.routes.len() > self.max_routes_allowed {
            violations.push(NetworkPolicyViolation {
                rule_id: "RULE_ROUTE_MAX_CAP".into(),
                target: "global:routes".into(),
                description: format!(
                    "Route count ({}) exceeds allowed maximum ({})",
                    state.routes.len(),
                    self.max_routes_allowed
                ),
                fatal: true,
            });
        }
        if state.dns.nameservers.len() > self.max_dns_servers_allowed {
            violations.push(NetworkPolicyViolation {
                rule_id: "RULE_DNS_MAX_CAP".into(),
                target: "global:dns".into(),
                description: format!(
                    "DNS nameserver count ({}) exceeds allowed maximum ({})",
                    state.dns.nameservers.len(),
                    self.max_dns_servers_allowed
                ),
                fatal: true,
            });
        }

        // Available interface names set for route cross-validation
        let mut iface_names = BTreeSet::new();

        // NPOL1: Interface evaluation
        for iface in &state.interfaces {
            iface_names.insert(iface.name.clone());

            // Check disallowed types
            if self.disallowed_interface_types.contains(&iface.iftype) {
                violations.push(NetworkPolicyViolation {
                    rule_id: "RULE_IFACE_DISALLOWED_TYPE".into(),
                    target: iface.name.clone(),
                    description: format!(
                        "Interface '{}' has disallowed type '{:?}'",
                        iface.name, iface.iftype
                    ),
                    fatal: true,
                });
            }

            // Check prohibited names
            if self.prohibited_interface_names.iter().any(|p| p == &iface.name) {
                violations.push(NetworkPolicyViolation {
                    rule_id: "RULE_IFACE_PROHIBITED_NAME".into(),
                    target: iface.name.clone(),
                    description: format!("Interface '{}' is explicitly prohibited by policy", iface.name),
                    fatal: true,
                });
            }

            // Check allowed whitelist if configured
            if let Some(ref whitelist) = self.allowed_interface_names {
                if !whitelist.iter().any(|w| w == &iface.name) {
                    violations.push(NetworkPolicyViolation {
                        rule_id: "RULE_IFACE_NOT_WHITELISTED".into(),
                        target: iface.name.clone(),
                        description: format!("Interface '{}' is not in the allowed interfaces whitelist", iface.name),
                        fatal: true,
                    });
                }
            }

            // Check promiscuous mode
            if !self.allow_promiscuous && iface.flags.iter().any(|f| f == "PROMISC") {
                violations.push(NetworkPolicyViolation {
                    rule_id: "RULE_IFACE_PROMISCUOUS".into(),
                    target: iface.name.clone(),
                    description: format!("Interface '{}' is in promiscuous mode (PROMISC flag set)", iface.name),
                    fatal: true,
                });
            }

            // Check required MAC on Ethernet
            if self.require_mac_for_ethernet
                && iface.iftype == InterfaceType::Ethernet
                && iface.mac_address.is_none()
            {
                violations.push(NetworkPolicyViolation {
                    rule_id: "RULE_IFACE_MISSING_MAC".into(),
                    target: iface.name.clone(),
                    description: format!("Ethernet interface '{}' is missing required MAC address", iface.name),
                    fatal: true,
                });
            }
        }

        // NPOL2: Route evaluation
        for route in &state.routes {
            if let Some(ref iface) = route.interface {
                if !iface_names.contains(iface) {
                    violations.push(NetworkPolicyViolation {
                        rule_id: "RULE_ROUTE_ORPHAN_IFACE".into(),
                        target: format!("route:{}", route.destination),
                        description: format!(
                            "Route to '{}' references non-existent interface '{}'",
                            route.destination, iface
                        ),
                        fatal: true,
                    });
                }
            }
        }

        // NPOL3: DNS evaluation
        for ns in &state.dns.nameservers {
            if self.disallowed_dns_servers.iter().any(|d| d == ns) {
                violations.push(NetworkPolicyViolation {
                    rule_id: "RULE_DNS_DISALLOWED_SERVER".into(),
                    target: format!("dns:{}", ns),
                    description: format!("DNS server '{}' is in the disallowed nameservers list", ns),
                    fatal: true,
                });
            }
            if let Some(ref whitelist) = self.allowed_dns_servers {
                if !whitelist.iter().any(|w| w == ns) {
                    violations.push(NetworkPolicyViolation {
                        rule_id: "RULE_DNS_NOT_WHITELISTED".into(),
                        target: format!("dns:{}", ns),
                        description: format!("DNS server '{}' is not in the allowed nameservers whitelist", ns),
                        fatal: true,
                    });
                }
            }
        }

        // Deterministic sorting of violations by rule_id then target (NPOL4)
        violations.sort_by(|a, b| a.rule_id.cmp(&b.rule_id).then_with(|| a.target.cmp(&b.target)));

        // Verdict determination
        let has_fatal = violations.iter().any(|v| v.fatal);
        let verdict = match self.mode {
            NetworkPolicyMode::Enforcing => {
                if has_fatal {
                    "deny"
                } else {
                    "allow"
                }
            }
            NetworkPolicyMode::Audit => {
                if has_fatal {
                    "audit"
                } else {
                    "allow"
                }
            }
            NetworkPolicyMode::Permissive => "allow",
        };

        NetworkPolicyReport {
            verdict: verdict.into(),
            mode: self.mode,
            violations,
            interfaces_evaluated: state.interfaces.len(),
            routes_evaluated: state.routes.len(),
            dns_servers_evaluated: state.dns.nameservers.len(),
            redacted: self.redact_sensitive_addresses,
        }
    }

    /// Evaluates policy and sanitizes state (redacts addresses if configured).
    pub fn apply_and_sanitize(&self, state: &mut NetworkState) -> NetworkPolicyReport {
        let report = self.evaluate(state);

        if self.redact_sensitive_addresses {
            for iface in &mut state.interfaces {
                if let Some(ref mac) = iface.mac_address {
                    if mac.len() >= 8 {
                        iface.mac_address = Some(format!("{}:xx:xx:xx", &mac[..8]));
                    }
                }
                for ip in &mut iface.ip_addresses {
                    if let Some((prefix, _)) = ip.address.rsplit_once('.') {
                        ip.address = format!("{}.xxx", prefix);
                    }
                }
            }
        }

        report
    }

    /// Loads policy from JSON file (NPOL6).
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        validate_policy_path(path)?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let metadata = fs::metadata(path).map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;
        if metadata.len() > MAX_POLICY_FILE_BYTES {
            return Err(format!(
                "Policy file {} size {} exceeds maximum allowed ({} bytes)",
                path.display(),
                metadata.len(),
                MAX_POLICY_FILE_BYTES
            ));
        }
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read policy from {}: {}", path.display(), e))?;
        let policy: NetworkSecurityPolicy = serde_json::from_str(&content).map_err(|e| format!("Failed to parse policy JSON: {}", e))?;
        policy.validate()?;
        Ok(policy)
    }

    /// Serializes and saves policy to JSON file atomically (NPOL6).
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        validate_policy_path(path)?;
        self.validate()?;
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory {}: {}", parent.display(), e))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize policy: {}", e))?;

        let tmp_file_name = format!(
            ".{}.tmp.{}",
            path.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "pol".into()),
            std::process::id()
        );
        let tmp_path = if parent.as_os_str().is_empty() {
            PathBuf::from(tmp_file_name)
        } else {
            parent.join(tmp_file_name)
        };

        if let Err(e) = fs::write(&tmp_path, &json) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("Failed to write policy temp file {}: {}", tmp_path.display(), e));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
        }

        if let Err(e) = fs::rename(&tmp_path, path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("Failed to atomically rename {} to {}: {}", tmp_path.display(), path.display(), e));
        }
        Ok(())
    }

    /// Loads policy taking environment variables into account.
    pub fn from_env() -> Self {
        let mut pol = if let Ok(path) = std::env::var("AIOS_NETWORK_POLICY_FILE") {
            Self::load_from_path(Path::new(&path)).unwrap_or_default()
        } else {
            Self::default()
        };

        if let Ok(mode_str) = std::env::var("AIOS_NETWORK_POLICY_MODE") {
            match mode_str.trim().to_ascii_lowercase().as_str() {
                "enforcing" => pol.mode = NetworkPolicyMode::Enforcing,
                "audit" => pol.mode = NetworkPolicyMode::Audit,
                "permissive" => pol.mode = NetworkPolicyMode::Permissive,
                _ => {}
            }
        }

        if pol.validate().is_err() {
            return Self::default();
        }

        pol
    }
}
