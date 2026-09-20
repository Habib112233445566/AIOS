//! Integration test for Hardware Detection Core Service (T-01716)
//!
//! Validates:
//! - Public API re-exports from crate root: `aiosh_core::{HardwareService, HardwareScanOptions, HardwareInventory, validate_hardware_inventory, DeviceClass}`
//! - Integration invariants HS1..HS5:
//!   - HS1: Graceful fallback on missing roots.
//!   - HS2: Accurate class identification across subsystems.
//!   - HS3: Deterministic device ID sorting.
//!   - HS4: Hex normalization and path sanitization.
//!   - HS5: Inventory validity against HD1..HD5.
//! - End-to-end JSON serialization and deserialization roundtrip.

use aiosh_core::{
    validate_hardware_inventory, DeviceClass, HardwareInventory, HardwareScanOptions,
    HardwareService,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_hardware_service_crate_root_integration() {
    let tmp = TempDir::new().expect("create temp dir");
    let root = tmp.path();

    let sysfs = root.join("sys");
    let procfs = root.join("proc");

    // Setup mock sysfs
    let pci_dir = sysfs.join("bus/pci/devices/0000_01_00.0");
    fs::create_dir_all(&pci_dir).unwrap();
    fs::write(pci_dir.join("vendor"), "0x10de\n").unwrap();
    fs::write(pci_dir.join("device"), "0x2206\n").unwrap();
    fs::write(pci_dir.join("class"), "0x030000\n").unwrap();
    fs::write(pci_dir.join("driver"), "nvidia\n").unwrap();

    let usb_dir = sysfs.join("bus/usb/devices/1-2");
    fs::create_dir_all(&usb_dir).unwrap();
    fs::write(usb_dir.join("idVendor"), "046d\n").unwrap();
    fs::write(usb_dir.join("idProduct"), "c52b\n").unwrap();
    fs::write(usb_dir.join("product"), "Wireless Receiver\n").unwrap();

    let block_dir = sysfs.join("class/block/nvme0n1");
    fs::create_dir_all(&block_dir).unwrap();
    fs::write(block_dir.join("size"), "1000000\n").unwrap();
    fs::create_dir_all(block_dir.join("queue")).unwrap();
    fs::write(block_dir.join("queue/rotational"), "0\n").unwrap();
    fs::write(block_dir.join("removable"), "0\n").unwrap();

    let net_dir = sysfs.join("class/net/eth0");
    fs::create_dir_all(&net_dir).unwrap();
    fs::write(net_dir.join("address"), "52:54:00:12:34:56\n").unwrap();
    fs::write(net_dir.join("operstate"), "up\n").unwrap();

    let dmi_dir = sysfs.join("class/dmi/id");
    fs::create_dir_all(&dmi_dir).unwrap();
    fs::write(dmi_dir.join("sys_vendor"), "Supermicro\n").unwrap();
    fs::write(dmi_dir.join("product_name"), "SYS-E300-9D\n").unwrap();

    let cpu_dir = procfs.join("cpuinfo");
    fs::create_dir_all(&procfs).unwrap();
    fs::write(cpu_dir, "model name : AMD EPYC 7002\nprocessor : 0\nprocessor : 1\n").unwrap();

    // 1. Instantiate via crate root HardwareService
    let service = HardwareService::with_roots(&sysfs, &procfs);
    let options = HardwareScanOptions::default();
    let inventory = service.scan(&options).expect("scan must succeed");

    // 2. Validate HS5: validate_hardware_inventory
    assert!(validate_hardware_inventory(&inventory).is_ok());

    // 3. Validate HS2: All device classes present
    assert!(inventory.devices.iter().any(|d| d.class == DeviceClass::Gpu));
    assert!(inventory.devices.iter().any(|d| d.class == DeviceClass::Usb));
    assert!(inventory.devices.iter().any(|d| d.class == DeviceClass::Block));
    assert!(inventory.devices.iter().any(|d| d.class == DeviceClass::Network));
    assert!(inventory.devices.iter().any(|d| d.class == DeviceClass::System));
    assert!(inventory.devices.iter().any(|d| d.class == DeviceClass::Cpu));

    // 4. Validate HS3: Deterministic ordering by ID
    for window in inventory.devices.windows(2) {
        assert!(window[0].id <= window[1].id, "devices must be ordered by id");
    }

    // 5. Validate HS4: Hex normalization (lowercase, 4-digit, no 0x)
    let gpu = inventory.devices.iter().find(|d| d.class == DeviceClass::Gpu).unwrap();
    assert_eq!(gpu.vendor_id.as_deref(), Some("10de"));
    assert_eq!(gpu.device_id.as_deref(), Some("2206"));

    // 6. Validate JSON serialization roundtrip
    let json_repr = inventory.to_json().expect("serialize to json");
    let recovered = HardwareInventory::from_json(&json_repr).expect("deserialize from json");
    assert_eq!(recovered.devices.len(), inventory.devices.len());
    assert_eq!(recovered.summary, inventory.summary);
    assert!(validate_hardware_inventory(&recovered).is_ok());
}

#[test]
fn test_hardware_service_hs1_missing_roots_fallback() {
    let tmp = TempDir::new().expect("create temp dir");
    let non_existent = tmp.path().join("does_not_exist");

    let service = HardwareService::with_roots(&non_existent, &non_existent);
    let inventory = service.scan(&HardwareScanOptions::default()).expect("must not fail");

    assert_eq!(inventory.devices.len(), 0);
    assert_eq!(inventory.summary.len(), 0);
    assert!(validate_hardware_inventory(&inventory).is_ok());
}

#[test]
fn test_hardware_service_hs2_class_isolation() {
    let tmp = TempDir::new().expect("create temp dir");
    let root = tmp.path();

    let sysfs = root.join("sys");
    let procfs = root.join("proc");

    let net_dir = sysfs.join("class/net/wlan0");
    fs::create_dir_all(&net_dir).unwrap();
    fs::write(net_dir.join("address"), "aa:bb:cc:dd:ee:ff\n").unwrap();

    let service = HardwareService::with_roots(&sysfs, &procfs);
    let options = HardwareScanOptions {
        classes: Some(vec![DeviceClass::Network]),
        include_attributes: true,
    };
    let inventory = service.scan(&options).expect("scan must succeed");

    assert_eq!(inventory.devices.len(), 1);
    assert_eq!(inventory.devices[0].class, DeviceClass::Network);
    assert_eq!(inventory.devices[0].id, "net:wlan0");
    assert_eq!(inventory.summary.get("network"), Some(&1));
    assert_eq!(inventory.summary.len(), 1);
    assert!(validate_hardware_inventory(&inventory).is_ok());
}
