//! Capability Security Policy Subsystem (CAPSEC1..CAPSEC6) for AIOS Security Kernel.
//!
//! Enforces policy rules, attenuation depth bounds, prohibited paths, disallowed rights,
//! and temporal constraints on capability issuance and derivation.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::capability::{
    Capability, CapabilityConstraints, CapabilityRight, CapabilityScope,
};

/// Enforcement mode for capability security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityPolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

impl Default for CapabilityPolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// A specific security policy violation recorded during evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityPolicyViolation {
    pub rule_id: String,
    pub description: String,
    pub fatal: bool,
}

/// The result of evaluating an action against the capability security policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityPolicyVerdict {
    pub allowed: bool,
    pub mode: CapabilityPolicyMode,
    pub violations: Vec<CapabilityPolicyViolation>,
    pub evaluated_at: String,
}

impl CapabilityPolicyVerdict {
    pub fn pass(mode: CapabilityPolicyMode) -> Self {
        Self {
            allowed: true,
            mode,
            violations: Vec::new(),
            evaluated_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn reject(mode: CapabilityPolicyMode, violations: Vec<CapabilityPolicyViolation>) -> Self {
        let allowed = mode != CapabilityPolicyMode::Enforcing;
        Self {
            allowed,
            mode,
            violations,
            evaluated_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Security policy defining mandatory security criteria for capability issuance and attenuation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilitySecurityPolicy {
    pub mode: CapabilityPolicyMode,
    pub max_attenuation_depth: usize,
    pub disallowed_rights_by_subject_prefix: HashMap<String, Vec<CapabilityRight>>,
    pub prohibited_path_prefixes: Vec<String>,
    pub prohibited_network_hosts: Vec<String>,
    pub prohibited_tools: Vec<String>,
    pub require_temporal_bounds: bool,
    pub max_validity_duration_seconds: Option<i64>,
    pub max_invocations_ceiling: Option<u64>,
    pub max_bytes_ceiling: Option<u64>,
}

impl Default for CapabilitySecurityPolicy {
    fn default() -> Self {
        let mut disallowed = HashMap::new();
        disallowed.insert(
            "untrusted".to_string(),
            vec![CapabilityRight::Admin, CapabilityRight::Delegate, CapabilityRight::Delete],
        );
        disallowed.insert(
            "guest".to_string(),
            vec![CapabilityRight::Admin, CapabilityRight::Delegate, CapabilityRight::Write, CapabilityRight::Delete],
        );

        Self {
            mode: CapabilityPolicyMode::Enforcing,
            max_attenuation_depth: 64,
            disallowed_rights_by_subject_prefix: disallowed,
            prohibited_path_prefixes: vec![
                "/etc".to_string(),
                "/proc".to_string(),
                "/sys".to_string(),
                "/dev".to_string(),
                "/root".to_string(),
                "/var/run".to_string(),
                "C:\\Windows".to_string(),
                "C:\\Program Files".to_string(),
            ],
            prohibited_network_hosts: vec![
                "169.254.169.254".to_string(),
                "metadata.google.internal".to_string(),
            ],
            prohibited_tools: vec![
                "raw_syscall".to_string(),
                "kernel_module_load".to_string(),
                "reboot".to_string(),
            ],
            require_temporal_bounds: false,
            max_validity_duration_seconds: Some(86400 * 30), // 30 days
            max_invocations_ceiling: Some(1_000_000),
            max_bytes_ceiling: Some(10 * 1024 * 1024 * 1024), // 10 GB
        }
    }
}

impl CapabilitySecurityPolicy {
    /// Validates policy configuration invariants (1 <= depth <= 128, non-empty paths, etc.).
    pub fn validate(&self) -> Result<(), String> {
        if self.max_attenuation_depth == 0 || self.max_attenuation_depth > 128 {
            return Err("max_attenuation_depth must be between 1 and 128".into());
        }
        for prefix in &self.prohibited_path_prefixes {
            if prefix.is_empty() {
                return Err("prohibited_path_prefixes cannot contain empty strings".into());
            }
            if prefix.contains("..") {
                return Err("prohibited_path_prefixes cannot contain path traversal ('..')".into());
            }
        }
        if let Some(dur) = self.max_validity_duration_seconds {
            if dur <= 0 {
                return Err("max_validity_duration_seconds must be positive".into());
            }
        }
        Ok(())
    }

    /// Evaluates issuance of a root capability against the security policy.
    pub fn evaluate_issuance(
        &self,
        _issuer: &str,
        subject: &str,
        scope: &CapabilityScope,
        rights: &[CapabilityRight],
        constraints: &CapabilityConstraints,
    ) -> CapabilityPolicyVerdict {
        if self.mode == CapabilityPolicyMode::Permissive {
            return CapabilityPolicyVerdict::pass(self.mode);
        }

        let mut violations = Vec::new();

        // 1. Prohibited path prefixes & path hygiene
        if let CapabilityScope::Filesystem { path, .. } = scope {
            if path.chars().any(|c| c.is_control()) {
                violations.push(CapabilityPolicyViolation {
                    rule_id: "CAPSEC_MALFORMED_PATH".into(),
                    description: format!("path contains control characters"),
                    fatal: true,
                });
            }

            let has_traversal = path.split(['/', '\\']).any(|comp| comp == "..");
            if has_traversal {
                violations.push(CapabilityPolicyViolation {
                    rule_id: "CAPSEC_PATH_TRAVERSAL".into(),
                    description: format!("path '{}' contains path traversal ('..')", path),
                    fatal: true,
                });
            }

            let normalized_path = normalize_path(path);
            for prefix in &self.prohibited_path_prefixes {
                let normalized_prefix = normalize_path(prefix);
                if normalized_path == normalized_prefix
                    || normalized_path.starts_with(&format!("{}/", normalized_prefix))
                {
                    violations.push(CapabilityPolicyViolation {
                        rule_id: "CAPSEC_PROHIBITED_PATH".into(),
                        description: format!("path '{}' matches prohibited prefix '{}'", path, prefix),
                        fatal: true,
                    });
                    break;
                }
            }
        }

        // 2. Prohibited network hosts with sanitization
        if let CapabilityScope::Network { host, .. } = scope {
            let sanitized_host = sanitize_host(host);
            if self.prohibited_network_hosts.iter().any(|h| {
                let sanitized_rule = sanitize_host(h);
                sanitized_rule.eq_ignore_ascii_case(&sanitized_host)
            }) {
                violations.push(CapabilityPolicyViolation {
                    rule_id: "CAPSEC_PROHIBITED_HOST".into(),
                    description: format!("network host '{}' is prohibited by policy", host),
                    fatal: true,
                });
            }
        }

        // 3. Prohibited tools
        if let CapabilityScope::Tool { tool_name, .. } = scope {
            if self.prohibited_tools.iter().any(|t| t == tool_name) {
                violations.push(CapabilityPolicyViolation {
                    rule_id: "CAPSEC_PROHIBITED_TOOL".into(),
                    description: format!("tool '{}' is prohibited by policy", tool_name),
                    fatal: true,
                });
            }
        }

        // 4. Disallowed rights by subject prefix
        for (prefix, disallowed) in &self.disallowed_rights_by_subject_prefix {
            if subject.starts_with(prefix) {
                for r in rights {
                    if disallowed.contains(r) {
                        violations.push(CapabilityPolicyViolation {
                            rule_id: "CAPSEC_DISALLOWED_RIGHT".into(),
                            description: format!("subject '{}' is disallowed from receiving right '{}'", subject, r),
                            fatal: true,
                        });
                    }
                }
            }
        }

        // 5. Temporal constraints
        if self.require_temporal_bounds && constraints.expires_at.is_none() {
            violations.push(CapabilityPolicyViolation {
                rule_id: "CAPSEC_MISSING_TEMPORAL_BOUND".into(),
                description: "policy requires an explicit expires_at temporal bound".into(),
                fatal: true,
            });
        }
        if let (Some(expires_str), Some(max_dur)) = (&constraints.expires_at, self.max_validity_duration_seconds) {
            if let Ok(exp) = DateTime::parse_from_rfc3339(expires_str) {
                let now = Utc::now();
                let dur = exp.with_timezone(&Utc) - now;
                if dur.num_seconds() > max_dur {
                    violations.push(CapabilityPolicyViolation {
                        rule_id: "CAPSEC_EXCESSIVE_TEMPORAL_BOUND".into(),
                        description: format!("validity duration ({}s) exceeds maximum allowed duration ({}s)", dur.num_seconds(), max_dur),
                        fatal: true,
                    });
                }
            } else {
                violations.push(CapabilityPolicyViolation {
                    rule_id: "CAPSEC_INVALID_TEMPORAL_BOUND".into(),
                    description: format!("expires_at '{}' is not a valid RFC3339 timestamp", expires_str),
                    fatal: true,
                });
            }
        }

        // 6. Quota ceilings
        if let (Some(max_inv), Some(ceiling)) = (constraints.max_invocations, self.max_invocations_ceiling) {
            if max_inv > ceiling {
                violations.push(CapabilityPolicyViolation {
                    rule_id: "CAPSEC_INVOCATIONS_CEILING_EXCEEDED".into(),
                    description: format!("max_invocations ({}) exceeds policy ceiling ({})", max_inv, ceiling),
                    fatal: true,
                });
            }
        }
        if let (Some(quota_bytes), Some(ceiling)) = (constraints.quota_bytes, self.max_bytes_ceiling) {
            if quota_bytes > ceiling {
                violations.push(CapabilityPolicyViolation {
                    rule_id: "CAPSEC_BYTES_CEILING_EXCEEDED".into(),
                    description: format!("quota_bytes ({}) exceeds policy ceiling ({})", quota_bytes, ceiling),
                    fatal: true,
                });
            }
        }

        if violations.is_empty() {
            CapabilityPolicyVerdict::pass(self.mode)
        } else {
            CapabilityPolicyVerdict::reject(self.mode, violations)
        }
    }

    /// Evaluates attenuation of a parent capability to a child subject.
    pub fn evaluate_attenuation(
        &self,
        parent: &Capability,
        new_subject: &str,
        scope: &CapabilityScope,
        rights: &[CapabilityRight],
        constraints: &CapabilityConstraints,
        current_depth: usize,
    ) -> CapabilityPolicyVerdict {
        if self.mode == CapabilityPolicyMode::Permissive {
            return CapabilityPolicyVerdict::pass(self.mode);
        }

        let mut verdict = self.evaluate_issuance(&parent.issuer, new_subject, scope, rights, constraints);

        // Check depth limit
        if current_depth > self.max_attenuation_depth {
            verdict.violations.push(CapabilityPolicyViolation {
                rule_id: "CAPSEC_DEPTH_EXCEEDED".into(),
                description: format!("attenuation depth {} exceeds maximum allowed depth {}", current_depth, self.max_attenuation_depth),
                fatal: true,
            });
            if self.mode == CapabilityPolicyMode::Enforcing {
                verdict.allowed = false;
            }
        }

        verdict
    }
}

/// Normalizes path separators and collapses redundant slashes.
pub fn normalize_path(path: &str) -> String {
    let unified = path.replace('\\', "/");
    let mut result = String::with_capacity(unified.len());
    let mut prev_slash = false;
    for c in unified.chars() {
        if c == '/' {
            if !prev_slash {
                result.push(c);
                prev_slash = true;
            }
        } else {
            result.push(c);
            prev_slash = false;
        }
    }
    // Remove trailing slash if length > 1
    if result.len() > 1 && result.ends_with('/') {
        result.pop();
    }
    result
}

/// Sanitizes a host string by stripping brackets, trailing dots, and accidental port numbers.
pub fn sanitize_host(host: &str) -> String {
    let mut trimmed = host.trim();
    if let Some(colon_pos) = trimmed.rfind(':') {
        let after_colon = &trimmed[colon_pos + 1..];
        if !after_colon.is_empty() && after_colon.chars().all(|c| c.is_ascii_digit()) {
            trimmed = &trimmed[..colon_pos];
        }
    }
    let unbracketed = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    unbracketed.trim_end_matches('.').to_string()
}

