//! Unit tests for Hardware Detection Core Service (HS1..HS5).

use std::fs;
use tempfile::tempdir;

use aiosh_core::hardware::DeviceClass;
use aiosh_core::hardware_service::{HardwareScanOptions, HardwareService};

#[test]
fn test_hardware_service_mock_sysfs_scan() {
    let dir = tempdir().expect("tempdir");
    let sysfs = dir.path().join("sys");
    let procfs = dir.path().join("proc");

    // 1. Mock PCI GPU
    let pci_dev = sysfs.join("bus/pci/devices/0000_00_02.0");
    fs::create_dir_all(&pci_dev).expect("create pci dev dir");
    fs::write(pci_dev.join("vendor"), "0x8086\n").unwrap();
    fs::write(pci_dev.join("device"), "0x4680\n").unwrap();
    fs::write(pci_dev.join("class"), "0x030000\n").unwrap();
    fs::write(pci_dev.join("driver"), "i915").unwrap();

    // 2. Mock USB Device
    let usb_dev = sysfs.join("bus/usb/devices/1-1");
    fs::create_dir_all(&usb_dev).expect("create usb dev dir");
    fs::write(usb_dev.join("idVendor"), "046d\n").unwrap();
    fs::write(usb_dev.join("idProduct"), "c52b\n").unwrap();
    fs::write(usb_dev.join("manufacturer"), "Logitech\n").unwrap();
    fs::write(usb_dev.join("product"), "Logitech Unifying Receiver\n").unwrap();
    fs::write(usb_dev.join("speed"), "12\n").unwrap();

    // 3. Mock Storage Block Device
    let block_dev = sysfs.join("class/block/nvme0n1");
    fs::create_dir_all(&block_dev).expect("create block dev dir");
    fs::write(block_dev.join("size"), "3907029168\n").unwrap();
    fs::create_dir_all(block_dev.join("queue")).unwrap();
    fs::write(block_dev.join("queue/rotational"), "0\n").unwrap();
    fs::create_dir_all(block_dev.join("device")).unwrap();
    fs::write(block_dev.join("device/model"), "Samsung SSD 990 PRO 2TB\n").unwrap();

    // 4. Mock Network Device
    let net_dev = sysfs.join("class/net/eth0");
    fs::create_dir_all(&net_dev).expect("create net dev dir");
    fs::write(net_dev.join("address"), "00:11:22:33:44:55\n").unwrap();
    fs::write(net_dev.join("operstate"), "up\n").unwrap();
    fs::write(net_dev.join("speed"), "1000\n").unwrap();

    // 5. Mock CPU Topology
    let cpu_dev0 = sysfs.join("devices/system/cpu/cpu0");
    let cpu_dev1 = sysfs.join("devices/system/cpu/cpu1");
    fs::create_dir_all(&cpu_dev0).unwrap();
    fs::create_dir_all(&cpu_dev1).unwrap();
    fs::create_dir_all(&procfs).unwrap();
    fs::write(procfs.join("cpuinfo"), "model name\t: Intel Core i7-12700K\n").unwrap();

    // 6. Mock System DMI
    let dmi_dev = sysfs.join("class/dmi/id");
    fs::create_dir_all(&dmi_dev).unwrap();
    fs::write(dmi_dev.join("sys_vendor"), "Supermicro\n").unwrap();
    fs::write(dmi_dev.join("product_name"), "SuperServer E300-9D\n").unwrap();
    fs::write(dmi_dev.join("bios_version"), "2.1b\n").unwrap();

    // Instantiate service with mock roots
    let service = HardwareService::with_roots(&sysfs, &procfs);
    assert_eq!(service.sysfs_root(), sysfs);
    assert_eq!(service.procfs_root(), procfs);

    let options = HardwareScanOptions::default();
    let inv = service.scan(&options).expect("scan succeeds");

    // Invariants must pass
    assert!(inv.validate_invariants().is_ok());

    // Check discovered device count
    assert_eq!(inv.devices.len(), 6);

    // Verify summary parity (HD3)
    assert_eq!(inv.summary.get("gpu"), Some(&1));
    assert_eq!(inv.summary.get("usb"), Some(&1));
    assert_eq!(inv.summary.get("block"), Some(&1));
    assert_eq!(inv.summary.get("network"), Some(&1));
    assert_eq!(inv.summary.get("cpu"), Some(&1));
    assert_eq!(inv.summary.get("system"), Some(&1));

    // Verify deterministic ordering (HS3)
    let ids: Vec<&str> = inv.devices.iter().map(|d| d.id.as_str()).collect();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    assert_eq!(ids, sorted_ids);

    // Check specific device properties
    let pci = inv.get_device("pci:0000:00:02.0").expect("pci dev found");
    assert_eq!(pci.class, DeviceClass::Gpu);
    assert_eq!(pci.vendor_id.as_deref(), Some("8086"));
    assert_eq!(pci.device_id.as_deref(), Some("4680"));
    assert_eq!(pci.driver.as_deref(), Some("i915"));

    let usb = inv.get_device("usb:1-1").expect("usb dev found");
    assert_eq!(usb.class, DeviceClass::Usb);
    assert_eq!(usb.vendor_id.as_deref(), Some("046d"));
    assert_eq!(usb.device_id.as_deref(), Some("c52b"));
    assert_eq!(usb.name, "Logitech Unifying Receiver");

    let block = inv.get_device("block:nvme0n1").expect("block dev found");
    assert_eq!(block.class, DeviceClass::Block);
    assert_eq!(block.attributes.get("rotational").map(|s| s.as_str()), Some("0"));

    let net = inv.get_device("net:eth0").expect("net dev found");
    assert_eq!(net.class, DeviceClass::Network);
    assert_eq!(net.attributes.get("mac_address").map(|s| s.as_str()), Some("00:11:22:33:44:55"));

    let cpu = inv.get_device("cpu:0").expect("cpu dev found");
    assert_eq!(cpu.class, DeviceClass::Cpu);
    assert_eq!(cpu.name, "Intel Core i7-12700K");
    assert_eq!(cpu.attributes.get("cores").map(|s| s.as_str()), Some("2"));

    let sys = inv.get_device("system:dmi").expect("sys dev found");
    assert_eq!(sys.class, DeviceClass::System);
    assert_eq!(sys.vendor_name.as_deref(), Some("Supermicro"));
    assert_eq!(sys.device_name.as_deref(), Some("SuperServer E300-9D"));

    // Verify cache
    let cached = service.get_cached_inventory();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().devices.len(), 6);

    service.invalidate_cache();
    assert!(service.get_cached_inventory().is_none());
}

#[test]
fn test_hardware_service_class_filtering() {
    let dir = tempdir().expect("tempdir");
    let sysfs = dir.path().join("sys");
    let procfs = dir.path().join("proc");

    // Create 1 PCI GPU and 1 Net device
    let pci_dev = sysfs.join("bus/pci/devices/0000_00_02.0");
    fs::create_dir_all(&pci_dev).unwrap();
    fs::write(pci_dev.join("vendor"), "0x8086\n").unwrap();
    fs::write(pci_dev.join("device"), "0x4680\n").unwrap();
    fs::write(pci_dev.join("class"), "0x030000\n").unwrap();

    let net_dev = sysfs.join("class/net/eth0");
    fs::create_dir_all(&net_dev).unwrap();
    fs::write(net_dev.join("address"), "00:11:22:33:44:55\n").unwrap();

    let service = HardwareService::with_roots(&sysfs, &procfs);

    // Scan only Gpu
    let options = HardwareScanOptions {
        classes: Some(vec![DeviceClass::Gpu]),
        include_attributes: true,
    };
    let inv = service.scan(&options).expect("scan");

    assert_eq!(inv.devices.len(), 1);
    assert_eq!(inv.devices[0].class, DeviceClass::Gpu);
    assert_eq!(inv.summary.get("gpu"), Some(&1));
    assert_eq!(inv.summary.get("network"), None);
}

#[test]
fn test_hardware_service_attribute_stripping() {
    let dir = tempdir().expect("tempdir");
    let sysfs = dir.path().join("sys");
    let procfs = dir.path().join("proc");

    let net_dev = sysfs.join("class/net/eth0");
    fs::create_dir_all(&net_dev).unwrap();
    fs::write(net_dev.join("address"), "00:11:22:33:44:55\n").unwrap();

    let service = HardwareService::with_roots(&sysfs, &procfs);

    let options = HardwareScanOptions {
        classes: None,
        include_attributes: false,
    };
    let inv = service.scan(&options).expect("scan");

    assert_eq!(inv.devices.len(), 1);
    assert!(inv.devices[0].attributes.is_empty());
}

#[test]
fn test_hardware_service_empty_sysfs_resilience() {
    let dir = tempdir().expect("tempdir");
    let sysfs = dir.path().join("sys");
    let procfs = dir.path().join("proc");
    fs::create_dir_all(&sysfs).unwrap();
    fs::create_dir_all(&procfs).unwrap();

    let service = HardwareService::with_roots(&sysfs, &procfs);
    let inv = service.scan(&HardwareScanOptions::default()).expect("scan empty");

    assert_eq!(inv.devices.len(), 0);
    assert!(inv.summary.is_empty());
    assert!(inv.validate_invariants().is_ok());
}
