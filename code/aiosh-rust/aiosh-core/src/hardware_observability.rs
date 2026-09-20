//! Hardware Detection Observability Subsystem (HO1..HO6).
//!
//! Provides structured telemetry reports, driver binding ratios, device class
//! breakdowns, and policy compliance summaries for discovered hardware inventories.

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

use crate::hardware::{DeviceBus, DeviceClass, HardwareInventory};
use crate::hardware_policy::HardwareSecurityPolicy;

/// Canonical string representation for device classes in telemetry.
pub fn device_class_to_str(class: DeviceClass) -> &'static str {
    class.as_str()
}

/// Canonical string representation for device buses in telemetry.
pub fn device_bus_to_str(bus: DeviceBus) -> &'static str {
    bus.as_str()
}

/// Comprehensive hardware observability and telemetry report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareObservabilityReport {
    pub total_devices: usize,
    pub class_breakdown: BTreeMap<String, usize>,
    pub bus_breakdown: BTreeMap<String, usize>,
    pub driver_binding_count: usize,
    pub unbound_device_count: usize,
    pub driver_binding_rate: f64,
    pub total_attributes_count: usize,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_devices_found: Vec<String>,
    pub redacted_devices_count: usize,
    pub hostname: String,
    pub architecture: String,
    pub kernel_version: String,
    pub generated_at: String,
}

/// Maximum number of prohibited device IDs included in a single observability report to prevent payload inflation.
pub const MAX_PROHIBITED_DEVICES_REPORTED: usize = 1_000;

/// Sanitizes a string for safe inclusion in telemetry outputs (removes control characters, trims, limits to 256 chars).
pub fn sanitize_telemetry_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect::<String>()
        .trim()
        .to_string()
}

impl HardwareObservabilityReport {
    /// Generates an observability report from the provided inventory and optional security policy (HO1..HO6).
    pub fn generate(
        inventory: &HardwareInventory,
        policy_opt: Option<&HardwareSecurityPolicy>,
    ) -> Self {
        let total_devices = inventory.devices.len();
        let mut class_breakdown = BTreeMap::new();
        let mut bus_breakdown = BTreeMap::new();
        let mut driver_binding_count = 0;
        let mut unbound_device_count = 0;
        let mut total_attributes_count = 0;

        for dev in &inventory.devices {
            let class_str = dev.class.as_str();
            *class_breakdown.entry(class_str.to_string()).or_insert(0) += 1;

            let bus_str = dev.bus.as_str();
            *bus_breakdown.entry(bus_str.to_string()).or_insert(0) += 1;

            if dev.driver.is_some() {
                driver_binding_count += 1;
            } else {
                unbound_device_count += 1;
            }

            total_attributes_count += dev.attributes.len();
        }

        let driver_binding_rate = if total_devices == 0 {
            0.0
        } else {
            let rate = driver_binding_count as f64 / total_devices as f64;
            (rate * 10000.0).round() / 10000.0
        };

        let mut policy_compliant_count = total_devices;
        let mut policy_violations_count = 0;
        let mut prohibited_set = BTreeSet::new();
        let mut redacted_devices_count = 0;

        if let Some(policy) = policy_opt {
            let report = policy.evaluate(inventory);
            policy_violations_count = report.violations.len();
            redacted_devices_count = report.devices_redacted;

            let violating_devices: BTreeSet<String> = report
                .violations
                .iter()
                .map(|v| v.device_id.clone())
                .collect();

            policy_compliant_count = total_devices.saturating_sub(violating_devices.len());

            for v in &report.violations {
                if v.rule_id == "HPOL-ID" {
                    prohibited_set.insert(v.device_id.clone());
                }
            }
        }

        let prohibited_devices_found: Vec<String> = prohibited_set
            .into_iter()
            .take(MAX_PROHIBITED_DEVICES_REPORTED)
            .collect();

        let generated_at = chrono::Utc::now().to_rfc3339();

        Self {
            total_devices,
            class_breakdown,
            bus_breakdown,
            driver_binding_count,
            unbound_device_count,
            driver_binding_rate,
            total_attributes_count,
            policy_compliant_count,
            policy_violations_count,
            prohibited_devices_found,
            redacted_devices_count,
            hostname: sanitize_telemetry_text(&inventory.hostname),
            architecture: sanitize_telemetry_text(&inventory.architecture),
            kernel_version: sanitize_telemetry_text(&inventory.kernel_version),
            generated_at,
        }
    }
}
