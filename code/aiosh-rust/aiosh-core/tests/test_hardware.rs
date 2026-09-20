//! Unit tests for Hardware Detection data model (HD1..HD5).

use aiosh_core::hardware::{
    validate_device_id, validate_hex_id, validate_path, DeviceBus, DeviceClass, HardwareDevice,
    HardwareInventory,
};

#[test]
fn test_device_class_and_bus_string_mapping() {
    assert_eq!(DeviceClass::Cpu.as_str(), "cpu");
    assert_eq!(DeviceClass::Memory.as_str(), "memory");
    assert_eq!(DeviceClass::Block.as_str(), "block");
    assert_eq!(DeviceClass::Network.as_str(), "network");
    assert_eq!(DeviceClass::Gpu.as_str(), "gpu");
    assert_eq!(DeviceClass::Pci.as_str(), "pci");
    assert_eq!(DeviceClass::Usb.as_str(), "usb");
    assert_eq!(DeviceClass::System.as_str(), "system");
    assert_eq!(DeviceClass::Other.as_str(), "other");

    assert_eq!(DeviceClass::from_str_loose("NIC"), DeviceClass::Network);
    assert_eq!(DeviceClass::from_str_loose("storage"), DeviceClass::Block);
    assert_eq!(DeviceClass::from_str_loose("processor"), DeviceClass::Cpu);
    assert_eq!(DeviceClass::from_str_loose("graphics"), DeviceClass::Gpu);
    assert_eq!(DeviceClass::from_str_loose("unknown_foo"), DeviceClass::Other);

    assert_eq!(DeviceBus::Pci.as_str(), "pci");
    assert_eq!(DeviceBus::Usb.as_str(), "usb");
    assert_eq!(DeviceBus::Platform.as_str(), "platform");
    assert_eq!(DeviceBus::Scsi.as_str(), "scsi");
    assert_eq!(DeviceBus::Virtio.as_str(), "virtio");
    assert_eq!(DeviceBus::System.as_str(), "system");
    assert_eq!(DeviceBus::Unknown.as_str(), "unknown");

    assert_eq!(DeviceBus::from_str_loose("pcie"), DeviceBus::Pci);
    assert_eq!(DeviceBus::from_str_loose("sata"), DeviceBus::Scsi);
}

#[test]
fn test_hardware_device_validation_valid() {
    let dev = HardwareDevice::new("pci:0000:00:02.0", "Intel Iris Xe Graphics", DeviceClass::Gpu, DeviceBus::Pci)
        .with_vendor("8086", Some("Intel Corporation".into()))
        .with_device("4680", Some("Alder Lake-UP3 GT2 [Iris Xe Graphics]".into()))
        .with_driver("i915")
        .with_paths(Some("/sys/bus/pci/devices/0000:00:02.0".into()), Some("/dev/dri/card0".into()))
        .with_attribute("vram_mb", "4096");

    assert!(dev.validate().is_ok());
    assert_eq!(dev.id, "pci:0000:00:02.0");
    assert_eq!(dev.vendor_id.as_deref(), Some("8086"));
    assert_eq!(dev.device_id.as_deref(), Some("4680"));
    assert_eq!(dev.attributes.get("vram_mb").map(|s| s.as_str()), Some("4096"));
}

#[test]
fn test_hardware_device_validation_invalid_id() {
    // Empty ID
    assert!(validate_device_id("").is_err());
    assert!(validate_device_id("   ").is_err());

    // Whitespace in ID
    assert!(validate_device_id("device 1").is_err());

    // Control characters in ID
    assert!(validate_device_id("dev\x00ice").is_err());
    assert!(validate_device_id("dev\nice").is_err());

    let bad_dev = HardwareDevice::new("bad dev", "Valid Name", DeviceClass::Cpu, DeviceBus::System);
    assert!(bad_dev.validate().is_err());

    let empty_name_dev = HardwareDevice::new("cpu:0", "   ", DeviceClass::Cpu, DeviceBus::System);
    assert!(empty_name_dev.validate().is_err());
}

#[test]
fn test_hardware_device_validation_invalid_hex() {
    // Valid 4-hex
    assert!(validate_hex_id("8086", "vendor_id").is_ok());
    assert!(validate_hex_id("10de", "vendor_id").is_ok());
    assert!(validate_hex_id("ABCD", "vendor_id").is_ok());

    // Invalid lengths or characters
    assert!(validate_hex_id("808", "vendor_id").is_err());
    assert!(validate_hex_id("80860", "vendor_id").is_err());
    assert!(validate_hex_id("808g", "vendor_id").is_err());
    assert!(validate_hex_id("80-6", "vendor_id").is_err());

    let dev_bad_hex = HardwareDevice::new("pci:00:01.0", "GPU", DeviceClass::Gpu, DeviceBus::Pci)
        .with_vendor("10ZZ", None);
    assert!(dev_bad_hex.validate().is_err());
}

#[test]
fn test_hardware_device_validation_invalid_paths() {
    // Traversal rejection
    assert!(validate_path("/sys/devices/../../etc/shadow", "sysfs_path").is_err());

    // Control characters
    assert!(validate_path("/sys/devices/\x00bad", "sysfs_path").is_err());

    // Empty path
    assert!(validate_path("", "sysfs_path").is_err());

    let dev_bad_path = HardwareDevice::new("net:eth0", "NIC", DeviceClass::Network, DeviceBus::Pci)
        .with_paths(Some("/sys/class/net/../eth0".into()), None);
    assert!(dev_bad_path.validate().is_err());
}

#[test]
fn test_hardware_inventory_operations() {
    let mut inv = HardwareInventory::new("aios-node-01", "x86_64", "6.6.13-aios-hardened");
    assert_eq!(inv.hostname, "aios-node-01");
    assert_eq!(inv.architecture, "x86_64");
    assert_eq!(inv.devices.len(), 0);
    assert!(inv.summary.is_empty());

    // Add CPU
    let cpu = HardwareDevice::new("cpu:0", "Intel Core i7-12700K", DeviceClass::Cpu, DeviceBus::System)
        .with_attribute("cores", "12")
        .with_attribute("threads", "20");
    assert!(inv.add_device(cpu).is_ok());

    // Add Memory
    let mem = HardwareDevice::new("mem:dimm0", "DDR5 32GB 5600MHz", DeviceClass::Memory, DeviceBus::System)
        .with_attribute("size_bytes", "34359738368");
    assert!(inv.add_device(mem).is_ok());

    // Add NVMe Block
    let block = HardwareDevice::new("block:nvme0n1", "Samsung 990 PRO 2TB", DeviceClass::Block, DeviceBus::Pci)
        .with_vendor("144d", Some("Samsung Electronics".into()))
        .with_device("a80c", Some("NVMe SSD Controller PM9A1/980".into()))
        .with_driver("nvme")
        .with_paths(Some("/sys/class/block/nvme0n1".into()), Some("/dev/nvme0n1".into()))
        .with_attribute("size_gb", "2000");
    assert!(inv.add_device(block).is_ok());

    // Verify query methods
    assert_eq!(inv.devices.len(), 3);
    assert_eq!(inv.get_device("cpu:0").map(|d| d.name.as_str()), Some("Intel Core i7-12700K"));
    assert_eq!(inv.get_device("non_existent"), None);

    let cpu_list = inv.devices_by_class(DeviceClass::Cpu);
    assert_eq!(cpu_list.len(), 1);
    assert_eq!(cpu_list[0].id, "cpu:0");

    let pci_list = inv.devices_by_bus(DeviceBus::Pci);
    assert_eq!(pci_list.len(), 1);
    assert_eq!(pci_list[0].id, "block:nvme0n1");

    // Invariants must hold
    assert!(inv.validate_invariants().is_ok());

    // Verify summary
    assert_eq!(inv.summary.get("cpu"), Some(&1));
    assert_eq!(inv.summary.get("memory"), Some(&1));
    assert_eq!(inv.summary.get("block"), Some(&1));

    // Remove device
    let removed = inv.remove_device("mem:dimm0");
    assert!(removed.is_some());
    assert_eq!(inv.devices.len(), 2);
    assert_eq!(inv.summary.get("memory"), None);
    assert!(inv.validate_invariants().is_ok());
}

#[test]
fn test_hd1_duplicate_device_id_rejection() {
    let mut inv = HardwareInventory::new("host", "x86_64", "6.6.0");
    let dev1 = HardwareDevice::new("usb:1-1", "USB Flash Drive", DeviceClass::Block, DeviceBus::Usb);
    let dev2 = HardwareDevice::new("usb:1-1", "Another USB Device", DeviceClass::Usb, DeviceBus::Usb);

    assert!(inv.add_device(dev1).is_ok());
    let err = inv.add_device(dev2);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("duplicate device id: usb:1-1"));
}

#[test]
fn test_hd3_summary_parity() {
    let mut inv = HardwareInventory::new("host", "x86_64", "6.6.0");
    let dev = HardwareDevice::new("net:eth0", "Intel I219-V", DeviceClass::Network, DeviceBus::Pci);
    inv.add_device(dev).unwrap();

    assert!(inv.validate_invariants().is_ok());

    // Tamper with summary
    inv.summary.insert("network".into(), 99);
    let err = inv.validate_invariants();
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("HD3 violated"));
}

#[test]
fn test_hd5_json_roundtrip_and_deterministic_order() {
    let mut inv = HardwareInventory::new("aios-prod-box", "x86_64", "6.6.13");
    let dev = HardwareDevice::new("pci:0000:01:00.0", "NVIDIA RTX 4090", DeviceClass::Gpu, DeviceBus::Pci)
        .with_vendor("10de", Some("NVIDIA Corporation".into()))
        .with_device("2684", Some("AD102 [GeForce RTX 4090]".into()))
        .with_driver("nvidia")
        .with_attribute("cuda_cores", "16384")
        .with_attribute("vram_mb", "24576");
    inv.add_device(dev).unwrap();

    let json_str = inv.to_json().expect("to_json");
    assert!(json_str.contains("\"pci:0000:01:00.0\""));
    assert!(json_str.contains("\"10de\""));
    assert!(json_str.contains("\"2684\""));

    let restored = HardwareInventory::from_json(&json_str).expect("from_json");
    assert_eq!(restored.hostname, inv.hostname);
    assert_eq!(restored.architecture, inv.architecture);
    assert_eq!(restored.devices.len(), 1);
    assert_eq!(restored.devices[0], inv.devices[0]);
    assert_eq!(restored.summary, inv.summary);
}
