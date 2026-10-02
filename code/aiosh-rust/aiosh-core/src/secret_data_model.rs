//! Secrets Handling Data Model (SECDATA1..SECDATA7).
//!
//! Provides the core data structures, memory zeroization, scoping,
//! lifecycle state, and validation for Phase 2 Secrets Handling.

use std::collections::HashMap;
use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SECDATA_ERR_EMPTY_ID: &str = "SECDATA_ERR_EMPTY_ID";
pub const SECDATA_ERR_INVALID_ID: &str = "SECDATA_ERR_INVALID_ID";
pub const SECDATA_ERR_EMPTY_NAME: &str = "SECDATA_ERR_EMPTY_NAME";
pub const SECDATA_ERR_NAME_TOO_LONG: &str = "SECDATA_ERR_NAME_TOO_LONG";
pub const SECDATA_ERR_PAYLOAD_TOO_LARGE: &str = "SECDATA_ERR_PAYLOAD_TOO_LARGE";
pub const SECDATA_ERR_INVALID_STATE: &str = "SECDATA_ERR_INVALID_STATE";
pub const SECDATA_ERR_STATE_TRANSITION: &str = "SECDATA_ERR_STATE_TRANSITION";
pub const SECDATA_ERR_INVALID_SCOPE: &str = "SECDATA_ERR_INVALID_SCOPE";
pub const SECDATA_ERR_LABEL_BOUNDS: &str = "SECDATA_ERR_LABEL_BOUNDS";

pub const MAX_SECRET_ID_LEN: usize = 64;
pub const MAX_SECRET_NAME_LEN: usize = 128;
pub const MAX_SECRET_PAYLOAD_SIZE: usize = 64 * 1024; // 64 KiB
pub const MAX_SECRET_LABELS_COUNT: usize = 32;
pub const MAX_SECRET_LABEL_KEY_LEN: usize = 64;
pub const MAX_SECRET_LABEL_VAL_LEN: usize = 256;

/// Classification of secret material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    ApiKey,
    OAuthToken,
    DatabaseCredential,
    PrivateKey,
    TlsCertificate,
    SymmetricKey,
    Generic,
}

impl SecretKind {
    pub fn parse_kind(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "api_key" | "apikey" => Some(SecretKind::ApiKey),
            "oauth_token" | "oauthtoken" | "token" => Some(SecretKind::OAuthToken),
            "database_credential" | "database" | "db" => Some(SecretKind::DatabaseCredential),
            "private_key" | "privatekey" | "key" => Some(SecretKind::PrivateKey),
            "tls_certificate" | "tls" | "cert" => Some(SecretKind::TlsCertificate),
            "symmetric_key" | "symmetrickey" => Some(SecretKind::SymmetricKey),
            "generic" => Some(SecretKind::Generic),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SecretKind::ApiKey => "api_key",
            SecretKind::OAuthToken => "oauth_token",
            SecretKind::DatabaseCredential => "database_credential",
            SecretKind::PrivateKey => "private_key",
            SecretKind::TlsCertificate => "tls_certificate",
            SecretKind::SymmetricKey => "symmetric_key",
            SecretKind::Generic => "generic",
        }
    }
}

impl fmt::Display for SecretKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Authorization and isolation scope for secrets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "target")]
pub enum SecretScope {
    Global,
    Environment(String),
    Actor(String),
    Session(String),
}

impl SecretScope {
    /// Validates and parses scope from type and optional target string.
    pub fn parse_scope(scope_type: &str, target: Option<&str>) -> Result<Self, String> {
        match scope_type.to_ascii_lowercase().as_str() {
            "global" => Ok(SecretScope::Global),
            "environment" | "env" => {
                let t = target.ok_or_else(|| format!("{}: target required for environment scope", SECDATA_ERR_INVALID_SCOPE))?;
                if t.trim().is_empty() || t.len() > 64 {
                    return Err(format!("{}: environment target invalid", SECDATA_ERR_INVALID_SCOPE));
                }
                Ok(SecretScope::Environment(t.to_string()))
            }
            "actor" => {
                let t = target.ok_or_else(|| format!("{}: target required for actor scope", SECDATA_ERR_INVALID_SCOPE))?;
                if t.trim().is_empty() || t.len() > 64 {
                    return Err(format!("{}: actor target invalid", SECDATA_ERR_INVALID_SCOPE));
                }
                Ok(SecretScope::Actor(t.to_string()))
            }
            "session" => {
                let t = target.ok_or_else(|| format!("{}: target required for session scope", SECDATA_ERR_INVALID_SCOPE))?;
                if t.trim().is_empty() || t.len() > 64 {
                    return Err(format!("{}: session target invalid", SECDATA_ERR_INVALID_SCOPE));
                }
                Ok(SecretScope::Session(t.to_string()))
            }
            _ => Err(format!("{}: unknown scope type '{}'", SECDATA_ERR_INVALID_SCOPE, scope_type)),
        }
    }

    /// Checks if this scope matches or grants access to the requested scope.
    pub fn allows(&self, requested: &SecretScope) -> bool {
        match (self, requested) {
            (SecretScope::Global, _) => true,
            (SecretScope::Environment(a), SecretScope::Environment(b)) => a == b,
            (SecretScope::Actor(a), SecretScope::Actor(b)) => a == b,
            (SecretScope::Session(a), SecretScope::Session(b)) => a == b,
            _ => false,
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            SecretScope::Global => "global".to_string(),
            SecretScope::Environment(e) => format!("environment:{}", e),
            SecretScope::Actor(a) => format!("actor:{}", a),
            SecretScope::Session(s) => format!("session:{}", s),
        }
    }
}

impl fmt::Display for SecretScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Operational lifecycle state of a secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretState {
    Active,
    Rotated,
    Revoked,
    Expired,
}

impl SecretState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecretState::Active => "active",
            SecretState::Rotated => "rotated",
            SecretState::Revoked => "revoked",
            SecretState::Expired => "expired",
        }
    }

    pub fn is_accessible(&self) -> bool {
        matches!(self, SecretState::Active | SecretState::Rotated)
    }

    /// Verifies valid lifecycle transition.
    pub fn transition_to(&self, new_state: SecretState) -> Result<SecretState, String> {
        match (self, new_state) {
            (SecretState::Active, SecretState::Rotated) => Ok(SecretState::Rotated),
            (SecretState::Active, SecretState::Revoked) => Ok(SecretState::Revoked),
            (SecretState::Active, SecretState::Expired) => Ok(SecretState::Expired),
            (SecretState::Rotated, SecretState::Revoked) => Ok(SecretState::Revoked),
            (SecretState::Rotated, SecretState::Expired) => Ok(SecretState::Expired),
            (SecretState::Revoked, _) => Err(format!(
                "{}: cannot transition from terminal state 'revoked' to '{}'",
                SECDATA_ERR_STATE_TRANSITION,
                new_state.as_str()
            )),
            (SecretState::Expired, _) => Err(format!(
                "{}: cannot transition from terminal state 'expired' to '{}'",
                SECDATA_ERR_STATE_TRANSITION,
                new_state.as_str()
            )),
            (s, n) if *s == n => Ok(n),
            (s, n) => Err(format!(
                "{}: illegal state transition from '{}' to '{}'",
                SECDATA_ERR_STATE_TRANSITION,
                s.as_str(),
                n.as_str()
            )),
        }
    }
}

impl fmt::Display for SecretState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Publicly auditable and safe-to-serialize metadata describing a secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: SecretKind,
    pub scope: SecretScope,
    pub state: SecretState,
    pub version: u32,
    pub fingerprint: String, // hex-encoded SHA-256
    pub created_at: String,  // RFC3339
    pub updated_at: String,  // RFC3339
    pub expires_at: Option<String>,
    pub labels: HashMap<String, String>,
}

impl SecretMetadata {
    pub fn new(id: &str, name: &str, kind: SecretKind, scope: SecretScope, fingerprint: &str) -> Result<Self, String> {
        let now = Utc::now().to_rfc3339();
        let meta = Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            kind,
            scope,
            state: SecretState::Active,
            version: 1,
            fingerprint: fingerprint.to_string(),
            created_at: now.clone(),
            updated_at: now,
            expires_at: None,
            labels: HashMap::new(),
        };
        meta.validate()?;
        Ok(meta)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err(format!("{}: id cannot be empty", SECDATA_ERR_EMPTY_ID));
        }
        if self.id.len() > MAX_SECRET_ID_LEN {
            return Err(format!("{}: id exceeds {} chars", SECDATA_ERR_INVALID_ID, MAX_SECRET_ID_LEN));
        }
        if !self.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(format!("{}: id contains invalid characters", SECDATA_ERR_INVALID_ID));
        }
        if self.name.trim().is_empty() {
            return Err(format!("{}: name cannot be empty", SECDATA_ERR_EMPTY_NAME));
        }
        if self.name.len() > MAX_SECRET_NAME_LEN {
            return Err(format!("{}: name exceeds {} chars", SECDATA_ERR_NAME_TOO_LONG, MAX_SECRET_NAME_LEN));
        }
        if self.name.chars().any(|c| c.is_control()) {
            return Err(format!("{}: name contains control characters", SECDATA_ERR_NAME_TOO_LONG));
        }
        if self.labels.len() > MAX_SECRET_LABELS_COUNT {
            return Err(format!(
                "{}: labels count {} exceeds limit {}",
                SECDATA_ERR_LABEL_BOUNDS,
                self.labels.len(),
                MAX_SECRET_LABELS_COUNT
            ));
        }
        for (k, v) in &self.labels {
            if k.trim().is_empty() || k.len() > MAX_SECRET_LABEL_KEY_LEN || k.chars().any(|c| c.is_control()) {
                return Err(format!("{}: label key invalid or exceeds length limit", SECDATA_ERR_LABEL_BOUNDS));
            }
            if v.len() > MAX_SECRET_LABEL_VAL_LEN || v.chars().any(|c| c.is_control()) {
                return Err(format!("{}: label value exceeds length limit or contains control characters", SECDATA_ERR_LABEL_BOUNDS));
            }
        }
        Ok(())
    }

    /// Checks if secret is expired according to current UTC time.
    pub fn is_expired(&self) -> bool {
        if let Some(exp_str) = &self.expires_at {
            if let Ok(exp_dt) = DateTime::parse_from_rfc3339(exp_str) {
                return Utc::now() >= exp_dt.with_timezone(&Utc);
            }
        }
        false
    }
}

/// In-memory protected secret payload that zeroizes its content when dropped.
pub struct SecretValue {
    bytes: Vec<u8>,
}

impl SecretValue {
    /// Creates a new SecretValue from byte slice, validating maximum payload bounds.
    pub fn new(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_SECRET_PAYLOAD_SIZE {
            return Err(format!(
                "{}: payload size {} exceeds limit {}",
                SECDATA_ERR_PAYLOAD_TOO_LARGE,
                raw.len(),
                MAX_SECRET_PAYLOAD_SIZE
            ));
        }
        Ok(Self {
            bytes: raw.to_vec(),
        })
    }

    /// Creates a new SecretValue from string slice.
    pub fn from_str_slice(s: &str) -> Result<Self, String> {
        Self::new(s.as_bytes())
    }

    /// Returns byte slice view.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns UTF-8 string view if valid.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.bytes)
    }

    /// Intent-revealing accessor for controlled disclosure.
    pub fn expose_secret(&self) -> &[u8] {
        &self.bytes
    }

    /// Computes SHA-256 hexadecimal fingerprint of payload.
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.bytes);
        format!("{:x}", hasher.finalize())
    }

    /// Constant-time equality check to prevent timing side channels.
    pub fn constant_time_eq(&self, other: &SecretValue) -> bool {
        if self.bytes.len() != other.bytes.len() {
            return false;
        }
        let mut diff = 0u8;
        for (a, b) in self.bytes.iter().zip(other.bytes.iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }

    /// Safe masked representation without leaking sensitive entropy.
    pub fn masked_display(&self) -> String {
        if self.bytes.len() < 12 {
            "********".to_string()
        } else if let Ok(s) = self.as_str() {
            let prefix = &s[..4];
            let suffix = &s[s.len() - 4..];
            format!("{}...{}", prefix, suffix)
        } else {
            "********".to_string()
        }
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretValue({})", self.masked_display())
    }
}

/// Zeroize memory upon Drop.
impl Drop for SecretValue {
    fn drop(&mut self) {
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        for byte in self.bytes.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

/// A complete vault entry combining metadata and secret value.
pub struct SecretEntry {
    pub metadata: SecretMetadata,
    pub value: SecretValue,
}

impl SecretEntry {
    pub fn new(
        id: &str,
        name: &str,
        kind: SecretKind,
        scope: SecretScope,
        raw_val: &[u8],
    ) -> Result<Self, String> {
        let val = SecretValue::new(raw_val)?;
        let fingerprint = val.compute_fingerprint();
        let meta = SecretMetadata::new(id, name, kind, scope, &fingerprint)?;

        Ok(Self {
            metadata: meta,
            value: val,
        })
    }

    /// Rotates the secret value: increments version, recomputes fingerprint and updates timestamp.
    pub fn rotate(&mut self, new_raw_val: &[u8]) -> Result<(), String> {
        let new_val = SecretValue::new(new_raw_val)?;
        self.metadata.state = self.metadata.state.transition_to(SecretState::Rotated)?;
        self.metadata.version += 1;
        self.metadata.fingerprint = new_val.compute_fingerprint();
        self.metadata.updated_at = Utc::now().to_rfc3339();
        self.value = new_val;
        Ok(())
    }

    /// Revokes the secret: marks state as Revoked and clears secret bytes.
    pub fn revoke(&mut self) -> Result<(), String> {
        self.metadata.state = self.metadata.state.transition_to(SecretState::Revoked)?;
        self.metadata.updated_at = Utc::now().to_rfc3339();
        self.value = SecretValue::new(&[])?;
        Ok(())
    }
}
