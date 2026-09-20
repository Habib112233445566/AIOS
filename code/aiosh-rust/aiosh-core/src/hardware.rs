//! Hardware Detection data models, device inventory, and invariant validation (HD1..HD5).
//!
//! Provides strongly-typed representation of host hardware devices (CPU, Memory, Block,
//! Network, GPU, PCI, USB, System), interconnect buses, and complete inventory manifests.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// Functional classification of a hardware device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceClass {
    Cpu,
    Memory,
    Block,
    Network,
    Gpu,
    Pci,
    Usb,
    System,
    Other,
}

impl DeviceClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceClass::Cpu => "cpu",
            DeviceClass::Memory => "memory",
            DeviceClass::Block => "block",
            DeviceClass::Network => "network",
            DeviceClass::Gpu => "gpu",
            DeviceClass::Pci => "pci",
            DeviceClass::Usb => "usb",
            DeviceClass::System => "system",
            DeviceClass::Other => "other",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "cpu" | "processor" => DeviceClass::Cpu,
            "memory" | "ram" | "mem" => DeviceClass::Memory,
            "block" | "disk" | "storage" => DeviceClass::Block,
            "network" | "net" | "nic" | "wifi" | "ethernet" => DeviceClass::Network,
            "gpu" | "display" | "vga" | "graphics" => DeviceClass::Gpu,
            "pci" | "pcie" => DeviceClass::Pci,
            "usb" => DeviceClass::Usb,
            "system" | "dmi" | "smbios" | "bios" => DeviceClass::System,
            _ => DeviceClass::Other,
        }
    }
}

/// Interconnect bus type through which a device communicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceBus {
    Pci,
    Usb,
    Platform,
    Scsi,
    Virtio,
    System,
    Unknown,
}

impl DeviceBus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceBus::Pci => "pci",
            DeviceBus::Usb => "usb",
            DeviceBus::Platform => "platform",
            DeviceBus::Scsi => "scsi",
            DeviceBus::Virtio => "virtio",
            DeviceBus::System => "system",
            DeviceBus::Unknown => "unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "pci" | "pcie" => DeviceBus::Pci,
            "usb" => DeviceBus::Usb,
            "platform" => DeviceBus::Platform,
            "scsi" | "sata" | "sas" => DeviceBus::Scsi,
            "virtio" => DeviceBus::Virtio,
            "system" | "dmi" | "isa" => DeviceBus::System,
            _ => DeviceBus::Unknown,
        }
    }
}

/// An individual hardware device detected on the host system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareDevice {
    pub id: String,
    pub name: String,
    pub class: DeviceClass,
    pub bus: DeviceBus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysfs_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_path: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

impl HardwareDevice {
    pub fn new(id: impl Into<String>, name: impl Into<String>, class: DeviceClass, bus: DeviceBus) -> Self {
        HardwareDevice {
            id: id.into(),
            name: name.into(),
            class,
            bus,
            vendor_id: None,
            device_id: None,
            vendor_name: None,
            device_name: None,
            driver: None,
            sysfs_path: None,
            dev_path: None,
            attributes: BTreeMap::new(),
        }
    }

    pub fn with_vendor(mut self, vendor_id: impl Into<String>, vendor_name: Option<String>) -> Self {
        self.vendor_id = Some(vendor_id.into());
        self.vendor_name = vendor_name;
        self
    }

    pub fn with_device(mut self, device_id: impl Into<String>, device_name: Option<String>) -> Self {
        self.device_id = Some(device_id.into());
        self.device_name = device_name;
        self
    }

    pub fn with_driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = Some(driver.into());
        self
    }

    pub fn with_paths(mut self, sysfs_path: Option<String>, dev_path: Option<String>) -> Self {
        self.sysfs_path = sysfs_path;
        self.dev_path = dev_path;
        self
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Validates internal consistency of an individual device (HD1, HD2, HD4).
    pub fn validate(&self) -> Result<(), String> {
        validate_device_id(&self.id)?;
        if self.name.trim().is_empty() {
            return Err("device name cannot be empty".into());
        }
        if let Some(ref vid) = self.vendor_id {
            validate_hex_id(vid, "vendor_id")?;
        }
        if let Some(ref did) = self.device_id {
            validate_hex_id(did, "device_id")?;
        }
        if let Some(ref p) = self.sysfs_path {
            validate_path(p, "sysfs_path")?;
        }
        if let Some(ref p) = self.dev_path {
            validate_path(p, "dev_path")?;
        }
        for (k, v) in &self.attributes {
            if k.trim().is_empty() {
                return Err("attribute key cannot be empty".into());
            }
            if k.chars().any(|c| c.is_control()) || v.chars().any(|c| c.is_control()) {
                return Err(format!("attribute '{}' contains control characters", k));
            }
        }
        Ok(())
    }
}

/// Full host hardware inventory aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareInventory {
    pub timestamp: String,
    pub hostname: String,
    pub architecture: String,
    pub kernel_version: String,
    pub devices: Vec<HardwareDevice>,
    pub summary: BTreeMap<String, usize>,
}

impl HardwareInventory {
    pub fn new(hostname: impl Into<String>, architecture: impl Into<String>, kernel_version: impl Into<String>) -> Self {
        HardwareInventory {
            timestamp: chrono::Utc::now().to_rfc3339(),
            hostname: hostname.into(),
            architecture: architecture.into(),
            kernel_version: kernel_version.into(),
            devices: Vec::new(),
            summary: BTreeMap::new(),
        }
    }

    pub fn add_device(&mut self, device: HardwareDevice) -> Result<(), String> {
        device.validate()?;
        if self.devices.iter().any(|d| d.id == device.id) {
            return Err(format!("duplicate device id: {}", device.id));
        }
        self.devices.push(device);
        self.update_summary();
        Ok(())
    }

    pub fn get_device(&self, id: &str) -> Option<&HardwareDevice> {
        self.devices.iter().find(|d| d.id == id)
    }

    pub fn remove_device(&mut self, id: &str) -> Option<HardwareDevice> {
        if let Some(pos) = self.devices.iter().position(|d| d.id == id) {
            let removed = self.devices.remove(pos);
            self.update_summary();
            Some(removed)
        } else {
            None
        }
    }

    pub fn devices_by_class(&self, class: DeviceClass) -> Vec<&HardwareDevice> {
        self.devices.iter().filter(|d| d.class == class).collect()
    }

    pub fn devices_by_bus(&self, bus: DeviceBus) -> Vec<&HardwareDevice> {
        self.devices.iter().filter(|d| d.bus == bus).collect()
    }

    pub fn filter_by_class(&self, class: DeviceClass) -> Vec<&HardwareDevice> {
        self.devices_by_class(class)
    }

    pub fn update_summary(&mut self) {
        let mut counts = BTreeMap::new();
        for d in &self.devices {
            *counts.entry(d.class.as_str().to_string()).or_insert(0) += 1;
        }
        self.summary = counts;
    }

    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.hostname.trim().is_empty() {
            return Err("hostname cannot be empty".into());
        }
        if self.architecture.trim().is_empty() {
            return Err("architecture cannot be empty".into());
        }

        // HD1: Unique IDs
        let mut seen = std::collections::HashSet::new();
        for d in &self.devices {
            d.validate()?;
            if !seen.insert(&d.id) {
                return Err(format!("HD1 violated: duplicate device id '{}'", d.id));
            }
        }

        // HD3: Summary Parity
        let mut expected = BTreeMap::new();
        for d in &self.devices {
            *expected.entry(d.class.as_str().to_string()).or_insert(0) += 1;
        }
        if self.summary != expected {
            return Err(format!(
                "HD3 violated: summary {:?} does not match counted devices {:?}",
                self.summary, expected
            ));
        }

        Ok(())
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("json serialize failure: {}", e))
    }

    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let inv: HardwareInventory = serde_json::from_str(json_str)
            .map_err(|e| format!("json deserialize failure: {}", e))?;
        inv.validate_invariants()?;
        Ok(inv)
    }
}

pub fn validate_device_id(id: &str) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err("device id cannot be empty".into());
    }
    if id.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(format!("device id '{}' contains whitespace or control characters", id));
    }
    Ok(())
}

pub fn validate_hex_id(id: &str, field_name: &str) -> Result<(), String> {
    if id.len() != 4 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "{} must be a 4-digit hexadecimal string (e.g. '8086'), got '{}'",
            field_name, id
        ));
    }
    Ok(())
}

pub fn validate_path(path: &str, field_name: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err(format!("{} cannot be empty", field_name));
    }
    if path.chars().any(|c| c.is_control()) {
        return Err(format!("{} contains control characters", field_name));
    }
    if path.contains("..") {
        return Err(format!("{} contains traversal sequence '..'", field_name));
    }
    Ok(())
}
