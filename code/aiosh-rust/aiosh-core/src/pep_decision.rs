//! PEP Decision Engine Data Model (PEPDEC1..PEPDEC6) for AIOS Security Kernel.
//!
//! Provides the core request, decision effect, obligation, and combining algorithm
//! data structures for complete mediation and policy enforcement.

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Maximum length for subject identifiers (256 chars).
pub const MAX_PEP_SUBJECT_LEN: usize = 256;

/// Maximum length for resource URIs (1024 chars).
pub const MAX_PEP_RESOURCE_LEN: usize = 1024;

/// Maximum length for action names (64 chars).
pub const MAX_PEP_ACTION_LEN: usize = 64;

/// Maximum length for reason descriptions (512 chars).
pub const MAX_PEP_REASON_LEN: usize = 512;

/// Maximum number of obligations permitted in a single decision.
pub const MAX_PEP_OBLIGATIONS: usize = 32;

pub const PEP_ERR_INVALID_SUBJECT: &str = "PEP_ERR_INVALID_SUBJECT";
pub const PEP_ERR_INVALID_RESOURCE: &str = "PEP_ERR_INVALID_RESOURCE";
pub const PEP_ERR_INVALID_ACTION: &str = "PEP_ERR_INVALID_ACTION";
pub const PEP_ERR_EVALUATION: &str = "PEP_ERR_EVALUATION";

/// Errors originating from PEP decision processing and request validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PepDecisionError {
    InvalidSubject(String),
    InvalidResource(String),
    InvalidAction(String),
    InvalidEnvironment(String),
    EvaluationError(String),
}

impl std::fmt::Display for PepDecisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSubject(msg) => write!(f, "{}: {}", PEP_ERR_INVALID_SUBJECT, msg),
            Self::InvalidResource(msg) => write!(f, "{}: {}", PEP_ERR_INVALID_RESOURCE, msg),
            Self::InvalidAction(msg) => write!(f, "{}: {}", PEP_ERR_INVALID_ACTION, msg),
            Self::InvalidEnvironment(msg) => write!(f, "{}: {}", PEP_ERR_EVALUATION, msg),
            Self::EvaluationError(msg) => write!(f, "{}: {}", PEP_ERR_EVALUATION, msg),
        }
    }
}

impl std::error::Error for PepDecisionError {}

/// Environmental context bounding the policy request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PepEnvironmentContext {
    pub timestamp: String,
    pub session_id: Option<String>,
    pub client_ip: Option<String>,
    pub grant_id: Option<String>,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

/// Core authorization request evaluated by the PEP Decision Engine (PEPDEC2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepRequest {
    pub id: String,
    pub subject: String,
    pub resource: String,
    pub action: String,
    pub environment: PepEnvironmentContext,
}

/// Formal verdict of a policy decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepDecisionEffect {
    Permit,
    Deny,
    Indeterminate,
    NotApplicable,
}

/// Post-decision obligation or side-effect requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PepObligation {
    AuditLog { level: String, message: String },
    RateLimit { key: String, cost: u32 },
    RedactFields { fields: Vec<String> },
    Custom { name: String, payload: serde_json::Value },
}

/// Structured decision returned by the PEP Decision Engine (PEPDEC1, PEPDEC4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepDecision {
    pub request_id: String,
    pub effect: PepDecisionEffect,
    pub allowed: bool,
    pub matched_rule_id: Option<String>,
    pub reason: String,
    pub obligations: Vec<PepObligation>,
    pub evaluated_at: String,
    pub evaluation_duration_us: u64,
}

/// Rule combining algorithms supported by the decision engine (PEPDEC3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PepCombiningAlgorithm {
    #[default]
    DenyOverrides,
    PermitOverrides,
    FirstApplicable,
}

impl PepRequest {
    /// Constructs a new validated PEP authorization request.
    pub fn new(
        subject: impl Into<String>,
        resource: impl Into<String>,
        action: impl Into<String>,
        environment: Option<PepEnvironmentContext>,
    ) -> Result<Self, PepDecisionError> {
        let subject = subject.into();
        let resource = resource.into();
        let action = action.into();
        let env = environment.unwrap_or_default();

        validate_pep_string(&subject, "subject", MAX_PEP_SUBJECT_LEN)
            .map_err(PepDecisionError::InvalidSubject)?;
        validate_pep_string(&resource, "resource", MAX_PEP_RESOURCE_LEN)
            .map_err(PepDecisionError::InvalidResource)?;
        validate_pep_string(&action, "action", MAX_PEP_ACTION_LEN)
            .map_err(PepDecisionError::InvalidAction)?;

        let now = Utc::now();
        let id = format!("pep_req_{}_{}", now.timestamp_millis(), &now.format("%f"));

        Ok(Self {
            id,
            subject,
            resource,
            action,
            environment: env,
        })
    }
}

impl PepDecision {
    /// Creates a default deny decision (PEPDEC1).
    pub fn default_deny(request_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            effect: PepDecisionEffect::Deny,
            allowed: false,
            matched_rule_id: None,
            reason: reason.into(),
            obligations: Vec::new(),
            evaluated_at: Utc::now().to_rfc3339(),
            evaluation_duration_us: 0,
        }
    }

    /// Creates an explicit permit decision (PEPDEC1).
    pub fn permit(
        request_id: impl Into<String>,
        matched_rule_id: Option<String>,
        reason: impl Into<String>,
        obligations: Vec<PepObligation>,
        duration_us: u64,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            effect: PepDecisionEffect::Permit,
            allowed: true,
            matched_rule_id,
            reason: reason.into(),
            obligations,
            evaluated_at: Utc::now().to_rfc3339(),
            evaluation_duration_us: duration_us,
        }
    }

    /// Creates an explicit deny decision.
    pub fn deny(
        request_id: impl Into<String>,
        matched_rule_id: Option<String>,
        reason: impl Into<String>,
        obligations: Vec<PepObligation>,
        duration_us: u64,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            effect: PepDecisionEffect::Deny,
            allowed: false,
            matched_rule_id,
            reason: reason.into(),
            obligations,
            evaluated_at: Utc::now().to_rfc3339(),
            evaluation_duration_us: duration_us,
        }
    }
}

/// Helper function to validate string fields against control characters and length bounds.
pub fn validate_pep_string(val: &str, field_name: &str, max_len: usize) -> Result<(), String> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return Err(format!("{} cannot be empty", field_name));
    }
    if trimmed.len() > max_len {
        return Err(format!("{} exceeds max length of {} characters", field_name, max_len));
    }
    if trimmed.chars().any(|c| c.is_control() || c == '\0') {
        return Err(format!("{} cannot contain control characters", field_name));
    }
    Ok(())
}

/// Individual policy rule evaluated by the PEP Decision Engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepPolicyRule {
    pub id: String,
    pub target_subject: Option<String>,
    pub target_resource: Option<String>,
    pub target_action: Option<String>,
    pub effect: PepDecisionEffect,
    #[serde(default)]
    pub obligations: Vec<PepObligation>,
    #[serde(default)]
    pub description: String,
}

impl PepPolicyRule {
    /// Checks if this rule matches a given PEP authorization request.
    pub fn matches(&self, req: &PepRequest) -> bool {
        // Subject matching (exact or wildcard)
        if let Some(ref s) = self.target_subject {
            if !match_pattern(s, &req.subject) {
                return false;
            }
        }

        // Resource matching (exact, prefix, or wildcard)
        if let Some(ref r) = self.target_resource {
            if !match_pattern(r, &req.resource) {
                return false;
            }
        }

        // Action matching (exact or wildcard)
        if let Some(ref a) = self.target_action {
            if !match_pattern(a, &req.action) {
                return false;
            }
        }

        true
    }
}

/// Helper to match wildcards '*' in patterns.
pub fn match_pattern(pattern: &str, candidate: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        candidate.starts_with(prefix)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        candidate.ends_with(suffix)
    } else {
        pattern.eq_ignore_ascii_case(candidate)
    }
}

impl PepDecision {
    /// Validates core invariants PEPDEC1..PEPDEC6 on the decision record.
    pub fn validate_invariants(&self) -> Result<(), String> {
        // PEPDEC1: allowed == (effect == PepDecisionEffect::Permit)
        let expected_allowed = self.effect == PepDecisionEffect::Permit;
        if self.allowed != expected_allowed {
            return Err(format!(
                "PEPDEC1 invariant violated: allowed ({}) != expected ({}) for effect {:?}",
                self.allowed, expected_allowed, self.effect
            ));
        }

        // PEPDEC2 & PEPDEC4: Request ID must be present
        if self.request_id.trim().is_empty() {
            return Err("PEPDEC4 invariant violated: request_id cannot be empty".to_string());
        }

        // Obligation count bound
        if self.obligations.len() > MAX_PEP_OBLIGATIONS {
            return Err(format!(
                "obligation count {} exceeds maximum {}",
                self.obligations.len(),
                MAX_PEP_OBLIGATIONS
            ));
        }

        Ok(())
    }
}

/// Evaluates a set of policy rules against a request using the specified combining algorithm (PEPDEC1, PEPDEC3).
pub fn evaluate_rules(
    rules: &[PepPolicyRule],
    req: &PepRequest,
    algorithm: PepCombiningAlgorithm,
) -> PepDecision {
    let start = std::time::Instant::now();

    match algorithm {
        PepCombiningAlgorithm::DenyOverrides => {
            let mut matched_permit: Option<(&PepPolicyRule, Vec<PepObligation>)> = None;
            for rule in rules {
                if rule.matches(req) {
                    if rule.effect == PepDecisionEffect::Deny {
                        let duration = start.elapsed().as_micros() as u64;
                        return PepDecision::deny(
                            &req.id,
                            Some(rule.id.clone()),
                            format!("denied by rule '{}': {}", rule.id, rule.description),
                            rule.obligations.clone(),
                            duration,
                        );
                    } else if rule.effect == PepDecisionEffect::Permit && matched_permit.is_none() {
                        matched_permit = Some((rule, rule.obligations.clone()));
                    }
                }
            }

            let duration = start.elapsed().as_micros() as u64;
            if let Some((rule, obligations)) = matched_permit {
                PepDecision::permit(
                    &req.id,
                    Some(rule.id.clone()),
                    format!("permitted by rule '{}': {}", rule.id, rule.description),
                    obligations,
                    duration,
                )
            } else {
                PepDecision::default_deny(
                    &req.id,
                    "default deny: no applicable permit rule matched",
                )
            }
        }
        PepCombiningAlgorithm::PermitOverrides => {
            let mut matched_deny: Option<(&PepPolicyRule, Vec<PepObligation>)> = None;
            for rule in rules {
                if rule.matches(req) {
                    if rule.effect == PepDecisionEffect::Permit {
                        let duration = start.elapsed().as_micros() as u64;
                        return PepDecision::permit(
                            &req.id,
                            Some(rule.id.clone()),
                            format!("permitted by rule '{}': {}", rule.id, rule.description),
                            rule.obligations.clone(),
                            duration,
                        );
                    } else if rule.effect == PepDecisionEffect::Deny && matched_deny.is_none() {
                        matched_deny = Some((rule, rule.obligations.clone()));
                    }
                }
            }

            let duration = start.elapsed().as_micros() as u64;
            if let Some((rule, obligations)) = matched_deny {
                PepDecision::deny(
                    &req.id,
                    Some(rule.id.clone()),
                    format!("denied by rule '{}': {}", rule.id, rule.description),
                    obligations,
                    duration,
                )
            } else {
                PepDecision::default_deny(
                    &req.id,
                    "default deny: no applicable permit rule matched",
                )
            }
        }
        PepCombiningAlgorithm::FirstApplicable => {
            for rule in rules {
                if rule.matches(req) {
                    let duration = start.elapsed().as_micros() as u64;
                    return match rule.effect {
                        PepDecisionEffect::Permit => PepDecision::permit(
                            &req.id,
                            Some(rule.id.clone()),
                            format!("permitted by first applicable rule '{}': {}", rule.id, rule.description),
                            rule.obligations.clone(),
                            duration,
                        ),
                        PepDecisionEffect::Deny => PepDecision::deny(
                            &req.id,
                            Some(rule.id.clone()),
                            format!("denied by first applicable rule '{}': {}", rule.id, rule.description),
                            rule.obligations.clone(),
                            duration,
                        ),
                        _ => PepDecision::default_deny(
                            &req.id,
                            format!("indeterminate rule '{}'", rule.id),
                        ),
                    };
                }
            }

            PepDecision::default_deny(
                &req.id,
                "default deny: no matching rule found",
            )
        }
    }
}
