//! Unit tests for Hardware Detection Security Policy Subsystem (HSEC1..HSEC5).

use tempfile::tempdir;

use aiosh_core::hardware::{DeviceBus, DeviceClass, HardwareDevice, HardwareInventory};
use aiosh_core::hardware_policy::{HardwarePolicyMode, HardwareSecurityPolicy};

fn create_sample_inventory() -> HardwareInventory {
    let mut inv = HardwareInventory::new("test-host", "x86_64", "6.6.13-aios");

    let mut gpu = HardwareDevice::new("pci:0000:00:02.0", "VGA Controller", DeviceClass::Gpu, DeviceBus::Pci);
    gpu.vendor_id = Some("8086".into());
    gpu.device_id = Some("9a49".into());
    inv.add_device(gpu).unwrap();

    let mut net = HardwareDevice::new("net:eth0", "Ethernet Controller", DeviceClass::Network, DeviceBus::Pci);
    net.vendor_id = Some("8086".into());
    net.device_id = Some("15f3".into());
    net.attributes.insert("address".into(), "00:11:22:33:44:55".into());
    net.attributes.insert("serial_number".into(), "SN12345678".into());
    net.attributes.insert("speed".into(), "1000".into());
    inv.add_device(net).unwrap();

    let mut block = HardwareDevice::new("block:sda", "Storage Disk", DeviceClass::Block, DeviceBus::Scsi);
    block.attributes.insert("uuid".into(), "1234-5678-abcd".into());
    block.attributes.insert("size".into(), "2097152".into());
    inv.add_device(block).unwrap();

    inv
}

#[test]
fn test_policy_default_valid() {
    let policy = HardwareSecurityPolicy::default();
    assert!(policy.validate().is_ok());
    assert_eq!(policy.mode, HardwarePolicyMode::Enforcing);
    assert!(policy.redact_sensitive_attributes);
    assert_eq!(policy.max_devices_allowed, 10_000);
}

#[test]
fn test_hsec1_precedence_prohibited_device() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.prohibited_device_ids.push("net:eth0".into());

    let mut inv = create_sample_inventory();
    let report = policy.apply_and_sanitize(&mut inv);

    // Fatal violation in Enforcing mode yields "deny"
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "HPOL-ID" && v.device_id == "net:eth0"));

    // Prohibited device should be stripped from inventory
    assert!(inv.get_device("net:eth0").is_none());
    assert_eq!(inv.devices.len(), 2);
}

#[test]
fn test_hsec1_audit_mode_verdict() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.mode = HardwarePolicyMode::Audit;
    policy.prohibited_device_ids.push("net:eth0".into());

    let mut inv = create_sample_inventory();
    let report = policy.apply_and_sanitize(&mut inv);

    // In Audit mode, verdict is "audit" and devices are not stripped
    assert_eq!(report.verdict, "audit");
    assert!(inv.get_device("net:eth0").is_some());
    assert_eq!(inv.devices.len(), 3);
}

#[test]
fn test_hsec1_permissive_mode_verdict() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.mode = HardwarePolicyMode::Permissive;
    policy.prohibited_device_ids.push("net:eth0".into());

    let mut inv = create_sample_inventory();
    let report = policy.apply_and_sanitize(&mut inv);

    assert_eq!(report.verdict, "allow");
    assert!(inv.get_device("net:eth0").is_some());
}

#[test]
fn test_hsec2_attribute_redaction() {
    let policy = HardwareSecurityPolicy::default();
    let mut inv = create_sample_inventory();
    let report = policy.apply_and_sanitize(&mut inv);

    assert!(report.devices_redacted >= 2);

    let net = inv.get_device("net:eth0").unwrap();
    assert_eq!(net.attributes.get("address").unwrap(), "<REDACTED>");
    assert_eq!(net.attributes.get("serial_number").unwrap(), "<REDACTED>");
    assert_eq!(net.attributes.get("speed").unwrap(), "1000"); // Non-sensitive remains unmasked

    let block = inv.get_device("block:sda").unwrap();
    assert_eq!(block.attributes.get("uuid").unwrap(), "<REDACTED>");
    assert_eq!(block.attributes.get("size").unwrap(), "2097152");
}

#[test]
fn test_hsec2_redaction_disabled() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.redact_sensitive_attributes = false;

    let mut inv = create_sample_inventory();
    let _ = policy.apply_and_sanitize(&mut inv);

    let net = inv.get_device("net:eth0").unwrap();
    assert_eq!(net.attributes.get("address").unwrap(), "00:11:22:33:44:55");
}

#[test]
fn test_hsec3_disallowed_class() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.disallowed_classes.push(DeviceClass::Gpu);

    let inv = create_sample_inventory();
    let report = policy.evaluate(&inv);

    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "HPOL-CLASS" && v.device_id == "pci:0000:00:02.0"));
}

#[test]
fn test_hsec3_disallowed_bus() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.disallowed_buses.push(DeviceBus::Scsi);

    let inv = create_sample_inventory();
    let report = policy.evaluate(&inv);

    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "HPOL-BUS" && v.device_id == "block:sda"));
}

#[test]
fn test_hsec4_deterministic_report() {
    let policy = HardwareSecurityPolicy::default();
    let inv1 = create_sample_inventory();
    let inv2 = create_sample_inventory();

    let r1 = policy.evaluate(&inv1);
    let r2 = policy.evaluate(&inv2);

    assert_eq!(r1, r2);
}

#[test]
fn test_hsec5_validation_bounds() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.max_devices_allowed = 0;
    assert!(policy.validate().is_err());

    let mut policy = HardwareSecurityPolicy::default();
    policy.max_devices_allowed = 50_001;
    assert!(policy.validate().is_err());

    let mut policy = HardwareSecurityPolicy::default();
    policy.prohibited_device_ids.push("".into());
    assert!(policy.validate().is_err());

    let mut policy = HardwareSecurityPolicy::default();
    policy.allowed_vendor_ids = Some(vec!["ZZZZ".into()]); // Non-hex
    assert!(policy.validate().is_err());

    let mut policy = HardwareSecurityPolicy::default();
    policy.allowed_vendor_ids = Some(vec!["8086".into()]); // Valid hex
    assert!(policy.validate().is_ok());
}

#[test]
fn test_hsec5_json_roundtrip() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.mode = HardwarePolicyMode::Audit;
    policy.prohibited_device_ids.push("pci:0000:01:00.0".into());

    let json_str = serde_json::to_string_pretty(&policy).expect("serialize");
    let deserialized: HardwareSecurityPolicy = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(policy, deserialized);
}

#[test]
fn test_hsec5_load_save_and_missing_fallback() {
    let dir = tempdir().expect("tempdir");
    let missing_path = dir.path().join("missing_policy.json");
    let loaded = HardwareSecurityPolicy::load_from_path(&missing_path).expect("load missing");
    assert_eq!(loaded, HardwareSecurityPolicy::default());

    let save_path = dir.path().join("sub").join("policy.json");
    let mut policy = HardwareSecurityPolicy::default();
    policy.max_devices_allowed = 5000;
    policy.save_to_path(&save_path).expect("save");
    assert!(save_path.exists());

    let reloaded = HardwareSecurityPolicy::load_from_path(&save_path).expect("reload");
    assert_eq!(policy, reloaded);
}

#[test]
fn test_hardening_path_traversal_rejected() {
    let dir = tempdir().expect("tempdir");
    let traversal_path = dir.path().join("..").join("evil_policy.json");
    assert!(HardwareSecurityPolicy::load_from_path(&traversal_path).is_err());

    let policy = HardwareSecurityPolicy::default();
    assert!(policy.save_to_path(&traversal_path).is_err());
}

#[test]
fn test_hardening_oversized_policy_file_rejected() {
    let dir = tempdir().expect("tempdir");
    let huge_path = dir.path().join("huge_policy.json");
    // Write 1.5 MB of data
    let huge_data = vec![b' '; 1_500_000];
    std::fs::write(&huge_path, &huge_data).expect("write huge file");

    let res = HardwareSecurityPolicy::load_from_path(&huge_path);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exceeds maximum allowed size"));
}

#[test]
fn test_hardening_case_insensitive_vendor_matching() {
    let inventory = create_sample_inventory();

    let mut policy = HardwareSecurityPolicy::default();
    // Allow uppercase "8086" while dev has lowercase "8086"
    policy.allowed_vendor_ids = Some(vec!["8086".into()]);

    let report = policy.evaluate(&inventory);
    assert!(!report.violations.iter().any(|v| v.rule_id == "HPOL-VENDOR"));
}

#[test]
fn test_hardening_list_bound_limits() {
    let mut policy = HardwareSecurityPolicy::default();
    policy.prohibited_device_ids = (0..10_001).map(|i| format!("dev_{}", i)).collect();
    assert!(policy.validate().is_err());

    let mut policy = HardwareSecurityPolicy::default();
    policy.allowed_vendor_ids = Some((0..10_001).map(|_| "8086".into()).collect());
    assert!(policy.validate().is_err());
}

