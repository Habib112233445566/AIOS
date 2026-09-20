//! Hardware Detection core discovery service (HS1..HS5).
//!
//! Introspects Linux kernel sysfs and procfs structures to assemble validated
//! HardwareInventory manifests. Supports custom root paths for hermetic mock testing.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::hardware::{DeviceBus, DeviceClass, HardwareDevice, HardwareInventory};

/// Maximum directory entries inspected per prober to prevent unbounded traversal.
pub const MAX_PROBE_ENTRIES: usize = 1024;

/// Options for configuring a hardware scan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareScanOptions {
    /// Restrict scanning to specific device classes.
    pub classes: Option<Vec<DeviceClass>>,
    /// Whether to collect extended device attributes.
    pub include_attributes: bool,
}

impl Default for HardwareScanOptions {
    fn default() -> Self {
        HardwareScanOptions {
            classes: None,
            include_attributes: true,
        }
    }
}

/// Core service for discovering and inventorying host hardware devices.
pub struct HardwareService {
    sysfs_root: PathBuf,
    procfs_root: PathBuf,
    cached_inventory: RwLock<Option<HardwareInventory>>,
}

impl HardwareService {
    /// Creates a HardwareService targeting the host system's root filesystem.
    pub fn new() -> Self {
        Self::with_roots("/sys", "/proc")
    }

    /// Creates a HardwareService targeting custom root paths (e.g. for mock testing).
    pub fn with_roots(sysfs_root: impl Into<PathBuf>, procfs_root: impl Into<PathBuf>) -> Self {
        HardwareService {
            sysfs_root: sysfs_root.into(),
            procfs_root: procfs_root.into(),
            cached_inventory: RwLock::new(None),
        }
    }

    /// Creates a HardwareService configured via HardwareConfig.
    pub fn with_config(config: &crate::hardware_config::HardwareConfig) -> Self {
        Self::with_roots(&config.sysfs_path, &config.procfs_path)
    }

    /// Returns the active sysfs root path.
    pub fn sysfs_root(&self) -> &Path {
        &self.sysfs_root
    }

    /// Returns the active procfs root path.
    pub fn procfs_root(&self) -> &Path {
        &self.procfs_root
    }

    /// Executes a hardware scan across all supported subsystems (HS1..HS5).
    pub fn scan(&self, options: &HardwareScanOptions) -> Result<HardwareInventory, String> {
        let mut discovered = Vec::new();

        // 1. Probe PCI Devices
        discovered.extend(self.probe_pci());

        // 2. Probe USB Devices
        discovered.extend(self.probe_usb());

        // 3. Probe Storage Block Devices
        discovered.extend(self.probe_block());

        // 4. Probe Network Interfaces
        discovered.extend(self.probe_net());

        // 5. Probe CPU Topology
        discovered.extend(self.probe_cpu());

        // 6. Probe System / DMI Platform
        discovered.extend(self.probe_system());

        // Deduplicate devices by ID (keep first seen)
        let mut seen_ids = std::collections::HashSet::new();
        discovered.retain(|d| seen_ids.insert(d.id.clone()));

        // Apply class filtering if requested
        if let Some(ref filter_classes) = options.classes {
            discovered.retain(|d| filter_classes.contains(&d.class));
        }

        // Apply attribute stripping if requested
        if !options.include_attributes {
            for d in &mut discovered {
                d.attributes.clear();
            }
        }

        // Deterministic sort by ID (HS3)
        discovered.sort_by(|a, b| a.id.cmp(&b.id));

        // Construct inventory
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "localhost".into());
        let mut inv = HardwareInventory::new(hostname, std::env::consts::ARCH, "unknown");

        for d in discovered {
            inv.add_device(d)?;
        }

        inv.validate_invariants()?;

        // Update cache
        if let Ok(mut guard) = self.cached_inventory.write() {
            *guard = Some(inv.clone());
        }

        Ok(inv)
    }

    /// Scans using defaults specified in HardwareConfig.
    pub fn scan_with_config(&self, config: &crate::hardware_config::HardwareConfig) -> Result<HardwareInventory, String> {
        let options = HardwareScanOptions {
            classes: config.enabled_classes.clone(),
            include_attributes: config.include_attributes,
        };
        self.scan(&options)
    }

    /// Scans host devices and applies security policy (filters prohibited devices, masks sensitive attributes).
    pub fn scan_with_policy(
        &self,
        options: &HardwareScanOptions,
        policy: &crate::hardware_policy::HardwareSecurityPolicy,
    ) -> Result<(HardwareInventory, crate::hardware_policy::HardwarePolicyReport), String> {
        let mut inventory = self.scan(options)?;
        let report = policy.apply_and_sanitize(&mut inventory);
        Ok((inventory, report))
    }

    /// Discovers host hardware and generates a comprehensive observability report (HO1..HO6).
    pub fn generate_observability_report(
        &self,
        options: &HardwareScanOptions,
        policy_opt: Option<&crate::hardware_policy::HardwareSecurityPolicy>,
    ) -> Result<crate::hardware_observability::HardwareObservabilityReport, String> {
        let inventory = self.scan(options)?;
        Ok(crate::hardware_observability::HardwareObservabilityReport::generate(&inventory, policy_opt))
    }

    /// Retrieves the cached hardware inventory if present.
    pub fn get_cached_inventory(&self) -> Option<HardwareInventory> {
        self.cached_inventory.read().ok().and_then(|guard| guard.clone())
    }

    /// Clears the cached inventory.
    pub fn invalidate_cache(&self) {
        if let Ok(mut guard) = self.cached_inventory.write() {
            *guard = None;
        }
    }

    // ----------------------------------------------------------------------
    // Sub-probers
    // ----------------------------------------------------------------------

    fn probe_pci(&self) -> Vec<HardwareDevice> {
        let pci_dir = self.sysfs_root.join("bus/pci/devices");
        let mut devices = Vec::new();

        let entries = match fs::read_dir(&pci_dir) {
            Ok(e) => e,
            Err(_) => return devices,
        };

        for (count, entry_res) in entries.enumerate() {
            if count >= MAX_PROBE_ENTRIES {
                break;
            }
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let raw_name = entry.file_name().to_string_lossy().to_string();
            let dev_name = if raw_name.contains('_') && !raw_name.contains(':') {
                raw_name.replacen('_', ":", 2)
            } else {
                raw_name
            };
            let dev_path = entry.path();

            let vendor_id = read_trimmed_file(&dev_path.join("vendor")).and_then(|v| normalize_hex_id(&v));
            let device_id = read_trimmed_file(&dev_path.join("device")).and_then(|v| normalize_hex_id(&v));
            let class_code = read_trimmed_file(&dev_path.join("class")).unwrap_or_default();
            let driver = resolve_driver_name(&dev_path.join("driver"));

            // Determine class from PCI class code
            let class = if class_code.starts_with("0x03") {
                DeviceClass::Gpu
            } else if class_code.starts_with("0x01") {
                DeviceClass::Block
            } else if class_code.starts_with("0x02") {
                DeviceClass::Network
            } else {
                DeviceClass::Pci
            };

            let name = format!("PCI Device {}", dev_name);
            let id = format!("pci:{}", dev_name);

            let mut dev = HardwareDevice::new(id, name, class, DeviceBus::Pci);
            dev.vendor_id = vendor_id;
            dev.device_id = device_id;
            dev.driver = driver;
            dev.sysfs_path = Some(dev_path.to_string_lossy().to_string());
            if !class_code.is_empty() {
                dev.attributes.insert("pci_class".into(), class_code);
            }

            if dev.validate().is_ok() {
                devices.push(dev);
            }
        }

        devices
    }

    fn probe_usb(&self) -> Vec<HardwareDevice> {
        let usb_dir = self.sysfs_root.join("bus/usb/devices");
        let mut devices = Vec::new();

        let entries = match fs::read_dir(&usb_dir) {
            Ok(e) => e,
            Err(_) => return devices,
        };

        for (count, entry_res) in entries.enumerate() {
            if count >= MAX_PROBE_ENTRIES {
                break;
            }
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let dev_name = entry.file_name().to_string_lossy().to_string();
            let dev_path = entry.path();

            let vendor_id = read_trimmed_file(&dev_path.join("idVendor")).and_then(|v| normalize_hex_id(&v));
            let device_id = read_trimmed_file(&dev_path.join("idProduct")).and_then(|v| normalize_hex_id(&v));
            let manufacturer = read_trimmed_file(&dev_path.join("manufacturer"));
            let product = read_trimmed_file(&dev_path.join("product"));
            let speed = read_trimmed_file(&dev_path.join("speed"));

            // Only register device if vendor_id or product is present (filters pure interface dirs)
            if vendor_id.is_none() && product.is_none() {
                continue;
            }

            let name = product.clone().unwrap_or_else(|| format!("USB Device {}", dev_name));
            let id = format!("usb:{}", dev_name);

            let mut dev = HardwareDevice::new(id, name, DeviceClass::Usb, DeviceBus::Usb);
            dev.vendor_id = vendor_id;
            dev.device_id = device_id;
            dev.vendor_name = manufacturer;
            dev.device_name = product;
            dev.sysfs_path = Some(dev_path.to_string_lossy().to_string());
            if let Some(s) = speed {
                dev.attributes.insert("speed_mbps".into(), s);
            }

            if dev.validate().is_ok() {
                devices.push(dev);
            }
        }

        devices
    }

    fn probe_block(&self) -> Vec<HardwareDevice> {
        let block_dir = self.sysfs_root.join("class/block");
        let mut devices = Vec::new();

        let entries = match fs::read_dir(&block_dir) {
            Ok(e) => e,
            Err(_) => return devices,
        };

        for (count, entry_res) in entries.enumerate() {
            if count >= MAX_PROBE_ENTRIES {
                break;
            }
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let blk_name = entry.file_name().to_string_lossy().to_string();
            let blk_path = entry.path();

            let size = read_trimmed_file(&blk_path.join("size"));
            let rotational = read_trimmed_file(&blk_path.join("queue/rotational"));
            let removable = read_trimmed_file(&blk_path.join("removable"));
            let model = read_trimmed_file(&blk_path.join("device/model"));

            let name = model.unwrap_or_else(|| format!("Block Device {}", blk_name));
            let id = format!("block:{}", blk_name);
            let dev_node = format!("/dev/{}", blk_name);

            let mut dev = HardwareDevice::new(id, name, DeviceClass::Block, DeviceBus::Scsi);
            dev.sysfs_path = Some(blk_path.to_string_lossy().to_string());
            dev.dev_path = Some(dev_node);
            if let Some(s) = size {
                dev.attributes.insert("size_sectors".into(), s);
            }
            if let Some(r) = rotational {
                dev.attributes.insert("rotational".into(), r);
            }
            if let Some(rm) = removable {
                dev.attributes.insert("removable".into(), rm);
            }

            if dev.validate().is_ok() {
                devices.push(dev);
            }
        }

        devices
    }

    fn probe_net(&self) -> Vec<HardwareDevice> {
        let net_dir = self.sysfs_root.join("class/net");
        let mut devices = Vec::new();

        let entries = match fs::read_dir(&net_dir) {
            Ok(e) => e,
            Err(_) => return devices,
        };

        for (count, entry_res) in entries.enumerate() {
            if count >= MAX_PROBE_ENTRIES {
                break;
            }
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let if_name = entry.file_name().to_string_lossy().to_string();
            let if_path = entry.path();

            let address = read_trimmed_file(&if_path.join("address"));
            let operstate = read_trimmed_file(&if_path.join("operstate"));
            let speed = read_trimmed_file(&if_path.join("speed"));

            let id = format!("net:{}", if_name);
            let name = format!("Network Interface {}", if_name);

            let mut dev = HardwareDevice::new(id, name, DeviceClass::Network, DeviceBus::Platform);
            dev.sysfs_path = Some(if_path.to_string_lossy().to_string());
            if let Some(a) = address {
                dev.attributes.insert("mac_address".into(), a);
            }
            if let Some(o) = operstate {
                dev.attributes.insert("operstate".into(), o);
            }
            if let Some(s) = speed {
                dev.attributes.insert("speed_mbps".into(), s);
            }

            if dev.validate().is_ok() {
                devices.push(dev);
            }
        }

        devices
    }

    fn probe_cpu(&self) -> Vec<HardwareDevice> {
        let mut devices = Vec::new();
        let cpu_dir = self.sysfs_root.join("devices/system/cpu");

        let mut core_count = 0;
        if let Ok(entries) = fs::read_dir(&cpu_dir) {
            for entry_res in entries.flatten() {
                let name = entry_res.file_name().to_string_lossy().to_string();
                if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                    core_count += 1;
                }
            }
        }

        // Try reading model from procfs/cpuinfo if available
        let cpuinfo_path = self.procfs_root.join("cpuinfo");
        let mut model_name = None;
        if let Ok(content) = fs::read_to_string(&cpuinfo_path) {
            for line in content.lines() {
                if line.starts_with("model name") {
                    if let Some((_, val)) = line.split_once(':') {
                        model_name = Some(val.trim().to_string());
                        break;
                    }
                }
            }
        }

        if core_count > 0 || model_name.is_some() {
            let name = model_name.unwrap_or_else(|| format!("Generic CPU ({} cores)", core_count));
            let mut dev = HardwareDevice::new("cpu:0", name, DeviceClass::Cpu, DeviceBus::System);
            if core_count > 0 {
                dev.attributes.insert("cores".into(), core_count.to_string());
            }
            dev.sysfs_path = Some(cpu_dir.to_string_lossy().to_string());
            if dev.validate().is_ok() {
                devices.push(dev);
            }
        }

        devices
    }

    fn probe_system(&self) -> Vec<HardwareDevice> {
        let mut devices = Vec::new();
        let dmi_dir = if self.sysfs_root.join("class/dmi/id").exists() {
            self.sysfs_root.join("class/dmi/id")
        } else {
            self.sysfs_root.join("devices/virtual/dmi/id")
        };

        let sys_vendor = read_trimmed_file(&dmi_dir.join("sys_vendor"));
        let product_name = read_trimmed_file(&dmi_dir.join("product_name"));
        let bios_version = read_trimmed_file(&dmi_dir.join("bios_version"));
        let chassis_type = read_trimmed_file(&dmi_dir.join("chassis_type"));

        if sys_vendor.is_some() || product_name.is_some() {
            let name = product_name.clone().unwrap_or_else(|| "Host System Board".into());
            let mut dev = HardwareDevice::new("system:dmi", name, DeviceClass::System, DeviceBus::System);
            dev.vendor_name = sys_vendor;
            dev.device_name = product_name;
            dev.sysfs_path = Some(dmi_dir.to_string_lossy().to_string());
            if let Some(b) = bios_version {
                dev.attributes.insert("bios_version".into(), b);
            }
            if let Some(c) = chassis_type {
                dev.attributes.insert("chassis_type".into(), c);
            }

            if dev.validate().is_ok() {
                devices.push(dev);
            }
        }

        devices
    }
}

impl Default for HardwareService {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------
// Utility Helpers
// ----------------------------------------------------------------------

pub(crate) fn read_trimmed_file(path: &Path) -> Option<String> {
    use std::io::Read;
    let file = fs::File::open(path).ok()?;
    let mut buffer = Vec::new();
    let mut take = file.take(1024);
    take.read_to_end(&mut buffer).ok()?;
    let s = String::from_utf8_lossy(&buffer);
    // Sanitize: filter out non-printable ASCII control characters except \t, \n, \r
    let sanitized: String = s
        .chars()
        .filter(|&c| !c.is_ascii_control() || c == '\t' || c == '\n' || c == '\r')
        .collect();
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn normalize_hex_id(val: &str) -> Option<String> {
    let stripped = val.strip_prefix("0x").unwrap_or(val).trim();
    if stripped.len() == 4 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(stripped.to_ascii_lowercase())
    } else {
        None
    }
}

pub(crate) fn resolve_driver_name(driver_path: &Path) -> Option<String> {
    let raw = if let Ok(target) = fs::read_link(driver_path) {
        target.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
    } else if driver_path.is_file() {
        read_trimmed_file(driver_path)
    } else if driver_path.exists() {
        driver_path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
    } else {
        None
    }?;

    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return None;
    }
    // Hardening: validate driver name format (alphanumeric, underscore, dash, dot)
    if trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.') {
        Some(trimmed.to_string())
    } else {
        None
    }
}

