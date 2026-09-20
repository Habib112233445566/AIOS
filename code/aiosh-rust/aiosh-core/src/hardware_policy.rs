//! Hardware Detection Security Policy Subsystem (HSEC1..HSEC5).
//!
//! Provides validation and sanitization of host hardware inventories against
//! security policies, sensitive attribute redaction, and device allow/denylists.

use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};

use crate::hardware::{DeviceBus, DeviceClass, HardwareInventory};

/// Enforcement mode for hardware security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwarePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

impl Default for HardwarePolicyMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// Security policy defining mandatory criteria and sanitization rules for hardware inventories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareSecurityPolicy {
    pub mode: HardwarePolicyMode,
    pub disallowed_classes: Vec<DeviceClass>,
    pub disallowed_buses: Vec<DeviceBus>,
    pub prohibited_device_ids: Vec<String>,
    pub allowed_vendor_ids: Option<Vec<String>>,
    pub redact_sensitive_attributes: bool,
    pub max_devices_allowed: usize,
}

impl Default for HardwareSecurityPolicy {
    fn default() -> Self {
        Self {
            mode: HardwarePolicyMode::Enforcing,
            disallowed_classes: vec![DeviceClass::Other],
            disallowed_buses: vec![DeviceBus::Unknown],
            prohibited_device_ids: Vec::new(),
            allowed_vendor_ids: None,
            redact_sensitive_attributes: true,
            max_devices_allowed: 10_000,
        }
    }
}

/// A specific security policy violation recorded against a hardware device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwarePolicyViolation {
    pub rule_id: String,
    pub device_id: String,
    pub description: String,
    pub fatal: bool,
}

/// Report summarizing policy evaluation against an inventory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwarePolicyReport {
    pub verdict: String, // "allow", "deny", "audit"
    pub mode: HardwarePolicyMode,
    pub violations: Vec<HardwarePolicyViolation>,
    pub devices_evaluated: usize,
    pub devices_redacted: usize,
}

impl HardwareSecurityPolicy {
    /// Validates policy configuration invariants (HSEC5).
    pub fn validate(&self) -> Result<(), String> {
        if self.max_devices_allowed == 0 || self.max_devices_allowed > 50_000 {
            return Err(format!(
                "HSEC5 violation: max_devices_allowed must be between 1 and 50,000 (got {})",
                self.max_devices_allowed
            ));
        }
        for id in &self.prohibited_device_ids {
            if id.trim().is_empty() {
                return Err("HSEC5 violation: prohibited_device_ids cannot contain empty strings".into());
            }
            if id.len() > 256 {
                return Err("HSEC5 violation: prohibited device ID exceeds 256 characters".into());
            }
        }
        if let Some(ref vids) = self.allowed_vendor_ids {
            for vid in vids {
                if vid.len() != 4 || !vid.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(format!("HSEC5 violation: invalid vendor ID '{}' in allowed_vendor_ids", vid));
                }
            }
        }
        Ok(())
    }

    /// Loads security policy from a JSON file, falling back to default if file is missing (HSEC5).
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read hardware policy from {}: {}", path.display(), e))?;
        let policy: HardwareSecurityPolicy = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse hardware policy JSON: {}", e))?;
        policy.validate()?;
        Ok(policy)
    }

    /// Saves security policy to a JSON file atomically (HSEC5).
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        self.validate()?;
        let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory {}: {}", parent.display(), e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize hardware policy: {}", e))?;

        let tmp_file_name = format!(
            ".{}.tmp.{}",
            path.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "pol".into()),
            std::process::id()
        );
        let tmp_path = if parent.as_os_str().is_empty() {
            std::path::PathBuf::from(tmp_file_name)
        } else {
            parent.join(tmp_file_name)
        };

        std::fs::write(&tmp_path, &json)
            .map_err(|e| format!("Failed to write hardware policy temp file {}: {}", tmp_path.display(), e))?;
        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("Failed to atomically rename {} to {}: {}", tmp_path.display(), path.display(), e));
        }
        Ok(())
    }

    /// Evaluates inventory against security policy without modifying it (HSEC1..HSEC5).
    pub fn evaluate(&self, inventory: &HardwareInventory) -> HardwarePolicyReport {
        let mut violations = Vec::new();
        let mut redacted_count = 0;

        // Check device count bound (HSEC5)
        if inventory.devices.len() > self.max_devices_allowed {
            violations.push(HardwarePolicyViolation {
                rule_id: "HPOL-COUNT".into(),
                device_id: "inventory".into(),
                description: format!(
                    "Device count {} exceeds policy limit of {}",
                    inventory.devices.len(),
                    self.max_devices_allowed
                ),
                fatal: true,
            });
        }

        for dev in &inventory.devices {
            // Check disallowed classes (HSEC3)
            if self.disallowed_classes.contains(&dev.class) {
                violations.push(HardwarePolicyViolation {
                    rule_id: "HPOL-CLASS".into(),
                    device_id: dev.id.clone(),
                    description: format!("Device class '{:?}' is disallowed by policy", dev.class),
                    fatal: true,
                });
            }

            // Check disallowed buses (HSEC3)
            if self.disallowed_buses.contains(&dev.bus) {
                violations.push(HardwarePolicyViolation {
                    rule_id: "HPOL-BUS".into(),
                    device_id: dev.id.clone(),
                    description: format!("Device bus '{:?}' is disallowed by policy", dev.bus),
                    fatal: true,
                });
            }

            // Check prohibited device IDs (HSEC1)
            if self.prohibited_device_ids.iter().any(|p| p == &dev.id) {
                violations.push(HardwarePolicyViolation {
                    rule_id: "HPOL-ID".into(),
                    device_id: dev.id.clone(),
                    description: format!("Device ID '{}' is explicitly prohibited by policy", dev.id),
                    fatal: true,
                });
            }

            // Check vendor allowlist if configured
            if let Some(ref allowed_vids) = self.allowed_vendor_ids {
                if let Some(ref vid) = dev.vendor_id {
                    if !allowed_vids.contains(vid) {
                        violations.push(HardwarePolicyViolation {
                            rule_id: "HPOL-VENDOR".into(),
                            device_id: dev.id.clone(),
                            description: format!("Vendor ID '{}' is not in allowed vendor list", vid),
                            fatal: false,
                        });
                    }
                }
            }

            // Count redactable attributes (HSEC2)
            if self.redact_sensitive_attributes {
                let has_sensitive = dev.attributes.keys().any(|k| is_sensitive_key(k));
                if has_sensitive {
                    redacted_count += 1;
                }
            }
        }

        // Deterministic sorting of violations by rule_id then device_id (HSEC4)
        violations.sort_by(|a, b| a.rule_id.cmp(&b.rule_id).then_with(|| a.device_id.cmp(&b.device_id)));

        let has_fatal = violations.iter().any(|v| v.fatal);
        let verdict = match self.mode {
            HardwarePolicyMode::Enforcing if has_fatal => "deny",
            HardwarePolicyMode::Audit if !violations.is_empty() => "audit",
            _ => "allow",
        };

        HardwarePolicyReport {
            verdict: verdict.into(),
            mode: self.mode,
            violations,
            devices_evaluated: inventory.devices.len(),
            devices_redacted: redacted_count,
        }
    }

    /// Evaluates policy and sanitizes inventory (filters prohibited devices and masks sensitive attributes).
    pub fn apply_and_sanitize(&self, inventory: &mut HardwareInventory) -> HardwarePolicyReport {
        let report = self.evaluate(inventory);

        if self.mode == HardwarePolicyMode::Enforcing {
            // Filter out prohibited or disallowed devices
            let prohibited_ids: BTreeSet<String> = report
                .violations
                .iter()
                .filter(|v| v.fatal)
                .map(|v| v.device_id.clone())
                .collect();

            if !prohibited_ids.is_empty() {
                inventory.devices.retain(|d| !prohibited_ids.contains(&d.id));
                inventory.update_summary();
            }
        }

        // Redact sensitive attributes if enabled (HSEC2)
        if self.redact_sensitive_attributes {
            for dev in &mut inventory.devices {
                for (k, v) in &mut dev.attributes {
                    if is_sensitive_key(k) {
                        *v = "<REDACTED>".into();
                    }
                }
            }
        }

        report
    }
}

pub(crate) fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("address")
        || lower.contains("mac")
        || lower.contains("serial")
        || lower.contains("uuid")
        || lower.contains("wwid")
}
