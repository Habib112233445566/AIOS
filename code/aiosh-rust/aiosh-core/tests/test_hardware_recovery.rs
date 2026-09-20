//! Unit tests for Hardware Detection Recovery & Validation subsystem (HVAL1..HVAL6).

use std::fs;
use std::path::Path;
use tempfile::tempdir;

use aiosh_core::hardware::{
    DeviceBus, DeviceClass, HardwareDevice, HardwareInventory,
};
use aiosh_core::hardware_recovery::{
    check_inventory_file, recover_inventory_file, recover_inventory_in_memory,
    validate_inventory, HardwareRecoveryAction, MAX_STORE_FILE_SIZE,
};
use aiosh_core::hardware_service::HardwareService;

#[test]
fn test_hval1_hval2_hval3_healthy_inventory_validation() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("hardware_inventory.json");

    let mut inv = HardwareInventory::new("test-host", "x86_64", "6.6.0-aios");
    inv.add_device(
        HardwareDevice::new("pci_0000_00_02_0", "Intel UHD Graphics", DeviceClass::Gpu, DeviceBus::Pci)
            .with_vendor("8086", Some("Intel Corporation".into()))
            .with_device("9bc4", Some("CometLake-S GT2".into()))
            .with_driver("i915"),
    ).expect("add device 1");

    inv.add_device(
        HardwareDevice::new("net_eth0", "Intel I219-V Ethernet", DeviceClass::Network, DeviceBus::Pci)
            .with_vendor("8086", Some("Intel Corporation".into()))
            .with_device("0d55", None)
            .with_driver("e1000e"),
    ).expect("add device 2");

    let rep = validate_inventory(&inv, &store_path, false);

    assert!(rep.validate_invariants().is_ok(), "invariants must hold: {:?}", rep.validate_invariants());
    assert!(rep.healthy, "inventory should be healthy");
    assert_eq!(rep.total_devices, 2);
    assert_eq!(rep.valid_devices, 2);
    assert_eq!(rep.invalid_devices, 0);
    assert!(!rep.drift_detected);
    assert!(rep.summary_mismatches.is_empty());
    assert!(rep.errors.is_empty());
}

#[test]
fn test_hval1_hval2_corrupted_inventory_in_memory_recovery() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("corrupted_inv.json");

    let mut inv = HardwareInventory::new("test-host", "x86_64", "6.6.0-aios");
    // Valid device
    inv.devices.push(
        HardwareDevice::new("dev_valid", "Valid Device", DeviceClass::Pci, DeviceBus::Pci)
            .with_vendor("10de", None)
            .with_device("1eb8", None),
    );
    // Invalid device 1: empty name
    inv.devices.push(HardwareDevice {
        id: "dev_invalid_name".into(),
        name: "".into(),
        class: DeviceClass::Usb,
        bus: DeviceBus::Usb,
        vendor_id: None,
        device_id: None,
        vendor_name: None,
        device_name: None,
        driver: None,
        sysfs_path: None,
        dev_path: None,
        attributes: std::collections::BTreeMap::new(),
    });
    // Invalid device 2: duplicate ID of dev_valid
    inv.devices.push(
        HardwareDevice::new("dev_valid", "Duplicate ID Device", DeviceClass::Pci, DeviceBus::Pci),
    );
    // Invalid device 3: bad hex vendor_id
    inv.devices.push(
        HardwareDevice::new("dev_bad_hex", "Bad Hex Device", DeviceClass::Block, DeviceBus::Scsi)
            .with_vendor("NOT_HEX", None),
    );

    // Initial summary is empty or mismatched
    inv.summary.insert("pci".into(), 99); // Wrong summary count

    let initial_rep = validate_inventory(&inv, &store_path, false);
    assert!(!initial_rep.healthy, "corrupted inventory must not be healthy");
    assert_eq!(initial_rep.total_devices, 4);
    assert_eq!(initial_rep.valid_devices, 1);
    assert_eq!(initial_rep.invalid_devices, 3);
    assert!(!initial_rep.summary_mismatches.is_empty());

    // Recover in-memory
    let actions = recover_inventory_in_memory(&mut inv);
    assert!(actions.iter().any(|a| matches!(a, HardwareRecoveryAction::PruneInvalidDevices { pruned_count: 3 })));
    assert!(actions.iter().any(|a| matches!(a, HardwareRecoveryAction::RecomputeSummary)));

    // Post-recovery validation
    let post_rep = validate_inventory(&inv, &store_path, false);
    assert!(post_rep.validate_invariants().is_ok());
    assert!(post_rep.healthy, "recovered inventory must be healthy");
    assert_eq!(post_rep.total_devices, 1);
    assert_eq!(post_rep.valid_devices, 1);
    assert_eq!(post_rep.invalid_devices, 0);
    assert_eq!(inv.summary.get("pci"), Some(&1));
}

#[test]
fn test_hval4_unparseable_json_quarantine_and_recovery() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("broken_hardware.json");

    let broken_content = "{ \"devices\": [ bad json content ...";
    fs::write(&store_path, broken_content).expect("write broken file");

    let check_rep = check_inventory_file(&store_path, false).expect("check");
    assert!(!check_rep.healthy);
    assert!(check_rep.errors[0].contains("store JSON deserialize failure"));

    let recovery_rep = recover_inventory_file(&store_path, None).expect("recover");
    assert!(recovery_rep.recovered);
    assert!(recovery_rep.backup_path.is_some(), "backup quarantine path must be created");

    // Verify backup exists on disk
    let bak_path = recovery_rep.backup_path.unwrap();
    assert!(Path::new(&bak_path).exists(), "quarantine backup file must exist");
    assert_eq!(fs::read_to_string(&bak_path).unwrap(), broken_content);

    // Verify recovered store file on disk is valid and healthy
    let final_check = check_inventory_file(&store_path, false).expect("check healed");
    assert!(final_check.healthy);
    assert!(final_check.validate_invariants().is_ok());
}

#[test]
fn test_hval5_oversized_store_file_rejection() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("oversized.json");

    // Create a sparse file or oversized file representation
    let file = fs::File::create(&store_path).expect("create file");
    file.set_len(MAX_STORE_FILE_SIZE + 1024).expect("set oversized len");

    let rep = check_inventory_file(&store_path, false).expect("check");
    assert!(!rep.healthy);
    assert!(rep.errors.iter().any(|e| e.contains("exceeds maximum permitted size")));
}

#[test]
fn test_hval6_sysfs_drift_detection() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("drift_inv.json");

    let mut inv = HardwareInventory::new("test-host", "x86_64", "6.6.0-aios");
    inv.add_device(
        HardwareDevice::new("pci_phantom", "Phantom GPU", DeviceClass::Gpu, DeviceBus::Pci)
            .with_paths(Some("/sys/bus/pci/devices/0000:99:99.9_nonexistent".into()), None),
    ).expect("add phantom device");

    let rep = validate_inventory(&inv, &store_path, true);
    assert!(!rep.healthy);
    assert!(rep.drift_detected, "drift must be detected for missing sysfs path");
    assert_eq!(rep.stale_paths.len(), 1);
    assert_eq!(rep.stale_paths[0], "/sys/bus/pci/devices/0000:99:99.9_nonexistent");
}

#[test]
fn test_hardware_service_validate_and_recover_store() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("service_managed.json");

    let service = HardwareService::with_roots(dir.path().join("sys"), dir.path().join("proc"));

    // Check non-existent store
    let check_missing = service.validate_store(&store_path, false).expect("validate missing");
    assert!(!check_missing.healthy);
    assert!(check_missing.errors[0].contains("does not exist"));

    // Recover non-existent store creates clean store
    let rec = service.recover_store(&store_path).expect("recover store");
    assert!(rec.recovered);
    assert!(rec.final_validation.healthy);

    // Second validation is healthy
    let check_healed = service.validate_store(&store_path, false).expect("validate healed");
    assert!(check_healed.healthy);
}

#[test]
fn test_hval_hardening() {
    let dir = tempdir().expect("tempdir");

    // 1. Path traversal rejected
    let traversal_path = dir.path().join("../evil.json");
    let rep = check_inventory_file(&traversal_path, false).expect("check traversal");
    assert!(!rep.healthy);
    assert!(rep.errors.iter().any(|e| e.contains("parent directory traversal")));

    let rec_err = recover_inventory_file(&traversal_path, None);
    assert!(rec_err.is_err());
    assert!(rec_err.unwrap_err().contains("parent directory traversal"));

    // 2. Control characters rejected
    let ctrl_path = dir.path().join("evil\x00store.json");
    let rep_ctrl = check_inventory_file(&ctrl_path, false).expect("check ctrl");
    assert!(!rep_ctrl.healthy);
    assert!(rep_ctrl.errors.iter().any(|e| e.contains("control characters")));

    // 3. Non-json extension rejected
    let txt_path = dir.path().join("hardware.txt");
    let rep_txt = check_inventory_file(&txt_path, false).expect("check txt");
    assert!(!rep_txt.healthy);
    assert!(rep_txt.errors.iter().any(|e| e.contains(".json")));
}

