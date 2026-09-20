//! Unit tests for Hardware Detection Observability Subsystem (HO1..HO6).

use aiosh_core::hardware::{DeviceBus, DeviceClass, HardwareDevice, HardwareInventory};
use aiosh_core::hardware_observability::HardwareObservabilityReport;
use aiosh_core::hardware_policy::{HardwarePolicyMode, HardwareSecurityPolicy};
use aiosh_core::hardware_service::{HardwareScanOptions, HardwareService};

fn create_sample_inventory() -> HardwareInventory {
    let mut inv = HardwareInventory::new("test-host", "x86_64", "6.6.13-aios");

    let mut gpu = HardwareDevice::new("pci:0000:00:02.0", "VGA Controller", DeviceClass::Gpu, DeviceBus::Pci);
    gpu.vendor_id = Some("8086".into());
    gpu.device_id = Some("9a49".into());
    gpu.driver = Some("i915".into());
    gpu.attributes.insert("driver".into(), "i915".into());
    inv.add_device(gpu).unwrap();

    let mut net = HardwareDevice::new("net:eth0", "Ethernet Controller", DeviceClass::Network, DeviceBus::Pci);
    net.vendor_id = Some("8086".into());
    net.device_id = Some("15f3".into());
    net.driver = Some("e1000e".into());
    net.attributes.insert("address".into(), "00:11:22:33:44:55".into());
    net.attributes.insert("speed".into(), "1000".into());
    inv.add_device(net).unwrap();

    let mut block = HardwareDevice::new("block:sda", "Storage Disk", DeviceClass::Block, DeviceBus::Scsi);
    block.attributes.insert("uuid".into(), "1234-5678-abcd".into());
    block.attributes.insert("size".into(), "2097152".into());
    // Note: block has driver: None (unbound)
    inv.add_device(block).unwrap();

    let mut usb = HardwareDevice::new("usb:1-1", "USB Mouse", DeviceClass::Usb, DeviceBus::Usb);
    usb.driver = None; // unbound
    inv.add_device(usb).unwrap();

    inv
}

#[test]
fn test_ho1_class_breakdown_parity() {
    let inv = create_sample_inventory();
    let report = HardwareObservabilityReport::generate(&inv, None);

    assert_eq!(report.total_devices, 4);
    let class_sum: usize = report.class_breakdown.values().sum();
    assert_eq!(report.total_devices, class_sum);
    assert_eq!(report.class_breakdown.get("gpu"), Some(&1));
    assert_eq!(report.class_breakdown.get("network"), Some(&1));
    assert_eq!(report.class_breakdown.get("block"), Some(&1));
    assert_eq!(report.class_breakdown.get("usb"), Some(&1));
}

#[test]
fn test_ho2_bus_breakdown_parity() {
    let inv = create_sample_inventory();
    let report = HardwareObservabilityReport::generate(&inv, None);

    assert_eq!(report.total_devices, 4);
    let bus_sum: usize = report.bus_breakdown.values().sum();
    assert_eq!(report.total_devices, bus_sum);
    assert_eq!(report.bus_breakdown.get("pci"), Some(&2));
    assert_eq!(report.bus_breakdown.get("scsi"), Some(&1));
    assert_eq!(report.bus_breakdown.get("usb"), Some(&1));
}

#[test]
fn test_ho3_driver_binding_accounting() {
    let inv = create_sample_inventory();
    let report = HardwareObservabilityReport::generate(&inv, None);

    assert_eq!(report.driver_binding_count, 2); // gpu, net
    assert_eq!(report.unbound_device_count, 2); // block, usb
    assert_eq!(report.total_devices, report.driver_binding_count + report.unbound_device_count);
}

#[test]
fn test_ho4_driver_binding_rate() {
    let inv = create_sample_inventory();
    let report = HardwareObservabilityReport::generate(&inv, None);

    // 2 bound out of 4 = 0.5
    assert_eq!(report.driver_binding_rate, 0.5);

    // Empty inventory test
    let empty_inv = HardwareInventory::new("empty-host", "x86_64", "6.6.13-aios");
    let empty_report = HardwareObservabilityReport::generate(&empty_inv, None);
    assert_eq!(empty_report.total_devices, 0);
    assert_eq!(empty_report.driver_binding_rate, 0.0);
}

#[test]
fn test_ho5_policy_compliance_summary() {
    let inv = create_sample_inventory();

    let mut policy = HardwareSecurityPolicy::default();
    policy.mode = HardwarePolicyMode::Audit;
    policy.prohibited_device_ids.push("usb:1-1".into());

    let report = HardwareObservabilityReport::generate(&inv, Some(&policy));

    assert_eq!(report.total_devices, 4);
    assert_eq!(report.policy_violations_count, 1);
    assert_eq!(report.policy_compliant_count, 3);
    assert_eq!(report.prohibited_devices_found, vec!["usb:1-1".to_string()]);
    assert_eq!(report.redacted_devices_count, 2); // net (address) and block (uuid)
}

#[test]
fn test_ho6_json_roundtrip() {
    let inv = create_sample_inventory();
    let report = HardwareObservabilityReport::generate(&inv, None);

    let json_str = serde_json::to_string_pretty(&report).expect("serialize");
    let deserialized: HardwareObservabilityReport = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(report, deserialized);
}

#[test]
fn test_observability_service_integration() {
    let service = HardwareService::new();
    let options = HardwareScanOptions {
        classes: None,
        include_attributes: true,
    };
    let report = service.generate_observability_report(&options, None).expect("service report");
    // Verified return shape
    assert_eq!(report.total_devices, report.class_breakdown.values().sum::<usize>());
    assert_eq!(report.total_devices, report.bus_breakdown.values().sum::<usize>());
    assert_eq!(report.total_devices, report.driver_binding_count + report.unbound_device_count);
}

#[test]
fn test_hardening_metadata_sanitization() {
    let inv = HardwareInventory::new("host\x1b[2J\x00_test", "x86_64\r\n", "6.6\t.13");
    let report = HardwareObservabilityReport::generate(&inv, None);

    assert_eq!(report.hostname, "host[2J_test");
    assert_eq!(report.architecture, "x86_64");
    assert_eq!(report.kernel_version, "6.6.13");
}

#[test]
fn test_hardening_prohibited_devices_cap() {
    let mut inv = HardwareInventory::new("test-host", "x86_64", "6.6.13-aios");
    let mut policy = HardwareSecurityPolicy::default();
    policy.mode = HardwarePolicyMode::Audit;

    // Create 1,200 prohibited devices
    for i in 0..1200 {
        let id = format!("pci:0000:00:{:04x}.0", i);
        policy.prohibited_device_ids.push(id.clone());
        let dev = HardwareDevice::new(id, "Device", DeviceClass::Other, DeviceBus::Pci);
        inv.add_device(dev).unwrap();
    }

    let report = HardwareObservabilityReport::generate(&inv, Some(&policy));
    assert_eq!(report.total_devices, 1200);
    // Cap is 1,000
    assert_eq!(report.prohibited_devices_found.len(), 1000);
}

