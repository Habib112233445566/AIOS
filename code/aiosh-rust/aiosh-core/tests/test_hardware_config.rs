//! Unit tests for Hardware Detection Configuration Subsystem (HCFG1..HCFG5)

use std::path::PathBuf;
use aiosh_core::hardware::DeviceClass;
use aiosh_core::hardware_config::HardwareConfig;
use tempfile::tempdir;

#[test]
fn test_hardware_config_default_valid() {
    let cfg = HardwareConfig::default();
    assert!(cfg.validate().is_ok());
    assert_eq!(cfg.default_store_path, PathBuf::from(".aios/hardware_inventory.json"));
    assert_eq!(cfg.sysfs_path, PathBuf::from("/sys"));
    assert_eq!(cfg.procfs_path, PathBuf::from("/proc"));
    assert_eq!(cfg.enabled_classes, None);
    assert!(cfg.include_attributes);
    assert_eq!(cfg.max_devices, 10_000);
    assert_eq!(cfg.max_payload_bytes, 10_485_760);
    assert_eq!(cfg.scan_timeout_secs, 30);
}

#[test]
fn test_hcfg1_path_hygiene_empty() {
    let mut cfg = HardwareConfig::default();
    cfg.default_store_path = PathBuf::from("");
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.sysfs_path = PathBuf::from("   ");
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.procfs_path = PathBuf::from("");
    assert!(cfg.validate().is_err());
}

#[test]
fn test_hcfg1_path_hygiene_control_chars() {
    let mut cfg = HardwareConfig::default();
    cfg.default_store_path = PathBuf::from("path/with/\n/newline");
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.sysfs_path = PathBuf::from("/sys\0bad");
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.procfs_path = PathBuf::from("/proc\tbad");
    assert!(cfg.validate().is_err());
}

#[test]
fn test_hcfg1_path_hygiene_max_length() {
    let mut cfg = HardwareConfig::default();
    let long_path = "a".repeat(1025);
    cfg.default_store_path = PathBuf::from(long_path);
    assert!(cfg.validate().is_err());
}

#[test]
fn test_hcfg2_class_filtering_valid() {
    let mut cfg = HardwareConfig::default();
    cfg.enabled_classes = Some(vec![DeviceClass::Cpu, DeviceClass::Gpu, DeviceClass::Block]);
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_hcfg2_class_filtering_duplicate() {
    let mut cfg = HardwareConfig::default();
    cfg.enabled_classes = Some(vec![DeviceClass::Cpu, DeviceClass::Gpu, DeviceClass::Cpu]);
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("HCFG2 violation: duplicate DeviceClass"));
}

#[test]
fn test_hcfg2_class_filtering_max_count() {
    let mut cfg = HardwareConfig::default();
    cfg.enabled_classes = Some(vec![
        DeviceClass::Cpu,
        DeviceClass::Memory,
        DeviceClass::Block,
        DeviceClass::Network,
        DeviceClass::Gpu,
        DeviceClass::Pci,
        DeviceClass::Usb,
        DeviceClass::System,
        DeviceClass::Other,
        DeviceClass::Cpu, // 10 items
    ]);
    assert!(cfg.validate().is_err());
}

#[test]
fn test_hcfg3_resource_bounds_max_devices() {
    let mut cfg = HardwareConfig::default();
    cfg.max_devices = 0;
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.max_devices = 50_001;
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.max_devices = 50_000;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_hcfg3_resource_bounds_max_payload() {
    let mut cfg = HardwareConfig::default();
    cfg.max_payload_bytes = 1023;
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.max_payload_bytes = 104_857_601;
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.max_payload_bytes = 1024;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_hcfg4_timeout_bounds() {
    let mut cfg = HardwareConfig::default();
    cfg.scan_timeout_secs = 0;
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.scan_timeout_secs = 301;
    assert!(cfg.validate().is_err());

    let mut cfg = HardwareConfig::default();
    cfg.scan_timeout_secs = 300;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_hcfg5_json_roundtrip() {
    let mut cfg = HardwareConfig::default();
    cfg.enabled_classes = Some(vec![DeviceClass::Cpu, DeviceClass::Network]);
    cfg.scan_timeout_secs = 45;

    let json_str = serde_json::to_string_pretty(&cfg).expect("serialize");
    let deserialized: HardwareConfig = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(cfg, deserialized);
}

#[test]
fn test_hcfg5_load_from_path_missing_file() {
    let non_existent = PathBuf::from("does_not_exist_hardware_config_12345.json");
    let cfg = HardwareConfig::load_from_path(&non_existent).expect("load missing");
    assert_eq!(cfg, HardwareConfig::default());
}

#[test]
fn test_hcfg5_save_and_load_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("subdir").join("hw_cfg.json");

    let mut cfg = HardwareConfig::default();
    cfg.scan_timeout_secs = 60;
    cfg.max_devices = 2500;
    cfg.enabled_classes = Some(vec![DeviceClass::Pci, DeviceClass::Usb]);

    cfg.save_to_path(&file_path).expect("save_to_path");
    assert!(file_path.exists());

    let loaded = HardwareConfig::load_from_path(&file_path).expect("load_from_path");
    assert_eq!(cfg, loaded);
}

#[test]
fn test_hardware_config_from_env() {
    std::env::set_var("AIOSH_HARDWARE_SYSFS", "/custom/sys");
    std::env::set_var("AIOSH_HARDWARE_PROCFS", "/custom/proc");
    std::env::set_var("AIOSH_HARDWARE_STORE", "/custom/store.json");
    std::env::set_var("AIOSH_HARDWARE_INCLUDE_ATTRS", "false");
    std::env::set_var("AIOSH_HARDWARE_TIMEOUT_SECS", "15");

    let cfg = HardwareConfig::from_env();
    assert_eq!(cfg.sysfs_path, PathBuf::from("/custom/sys"));
    assert_eq!(cfg.procfs_path, PathBuf::from("/custom/proc"));
    assert_eq!(cfg.default_store_path, PathBuf::from("/custom/store.json"));
    assert!(!cfg.include_attributes);
    assert_eq!(cfg.scan_timeout_secs, 15);

    // Clean up
    std::env::remove_var("AIOSH_HARDWARE_SYSFS");
    std::env::remove_var("AIOSH_HARDWARE_PROCFS");
    std::env::remove_var("AIOSH_HARDWARE_STORE");
    std::env::remove_var("AIOSH_HARDWARE_INCLUDE_ATTRS");
    std::env::remove_var("AIOSH_HARDWARE_TIMEOUT_SECS");
}
