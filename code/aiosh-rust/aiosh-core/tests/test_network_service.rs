//! Unit tests for Network Bootstrap Core Service (NSERV1..NSERV6).

use std::fs;
use aiosh_core::network::{InterfaceType, OperState};
use aiosh_core::network_service::NetworkService;
use tempfile::TempDir;

#[test]
fn test_nserv1_mock_service_initialization() {
    let tmp = TempDir::new().expect("create temp dir");
    let sysfs = tmp.path().join("sysfs");
    let procfs = tmp.path().join("procfs");
    let resolv = tmp.path().join("resolv.conf");

    let svc = NetworkService::with_paths(&sysfs, &procfs, &resolv);
    assert_eq!(svc.sysfs_net_root(), sysfs.as_path());
    assert_eq!(svc.procfs_root(), procfs.as_path());
    assert_eq!(svc.resolv_conf_path(), resolv.as_path());
}

#[test]
fn test_nserv2_scan_interfaces_and_fallback() {
    let tmp = TempDir::new().expect("create temp dir");
    let sysfs = tmp.path().join("sys/class/net");
    let procfs = tmp.path().join("proc/net");
    let resolv = tmp.path().join("etc/resolv.conf");

    fs::create_dir_all(&sysfs).expect("create sysfs net dir");

    // 1. eth0: complete mock interface
    let eth0_dir = sysfs.join("eth0");
    fs::create_dir_all(&eth0_dir).expect("create eth0 dir");
    fs::write(eth0_dir.join("operstate"), "up\n").expect("write operstate");
    fs::write(eth0_dir.join("address"), "52:54:00:12:34:56\n").expect("write address");
    fs::write(eth0_dir.join("mtu"), "1500\n").expect("write mtu");
    fs::write(eth0_dir.join("type"), "1\n").expect("write type");
    fs::write(eth0_dir.join("flags"), "0x1003\n").expect("write flags");

    // 2. lo: loopback interface
    let lo_dir = sysfs.join("lo");
    fs::create_dir_all(&lo_dir).expect("create lo dir");
    fs::write(lo_dir.join("operstate"), "up\n").expect("write operstate");
    fs::write(lo_dir.join("address"), "00:00:00:00:00:00\n").expect("write address");
    fs::write(lo_dir.join("mtu"), "65535\n").expect("write mtu");
    fs::write(lo_dir.join("type"), "772\n").expect("write type");
    fs::write(lo_dir.join("flags"), "0x9\n").expect("write flags");

    // 3. wlan0: partial/empty sysfs dir (fallback testing)
    let wlan0_dir = sysfs.join("wlan0");
    fs::create_dir_all(&wlan0_dir).expect("create wlan0 dir");

    let svc = NetworkService::with_paths(&sysfs, &procfs, &resolv);
    let ifaces = svc.scan_interfaces().expect("scan interfaces");

    // Must discover 3 interfaces, sorted alphabetically: eth0, lo, wlan0
    assert_eq!(ifaces.len(), 3);
    assert_eq!(ifaces[0].name, "eth0");
    assert_eq!(ifaces[0].iftype, InterfaceType::Ethernet);
    assert_eq!(ifaces[0].operstate, OperState::Up);
    assert_eq!(ifaces[0].mac_address.as_deref(), Some("52:54:00:12:34:56"));
    assert_eq!(ifaces[0].mtu, 1500);
    assert!(ifaces[0].flags.contains(&"UP".to_string()));
    assert!(ifaces[0].flags.contains(&"BROADCAST".to_string()));

    assert_eq!(ifaces[1].name, "lo");
    assert_eq!(ifaces[1].iftype, InterfaceType::Loopback);
    assert_eq!(ifaces[1].mac_address, None); // all-zeros loopback suppressed

    assert_eq!(ifaces[2].name, "wlan0");
    assert_eq!(ifaces[2].operstate, OperState::Unknown);
    assert_eq!(ifaces[2].mtu, 1500); // default
    assert_eq!(ifaces[2].mac_address, None);

    // Test get_interface
    let eth0 = svc.get_interface("eth0").expect("get eth0").expect("eth0 found");
    assert_eq!(eth0.name, "eth0");

    let nonexistent = svc.get_interface("eth99").expect("get nonexistent");
    assert!(nonexistent.is_none());

    // Invalid name rejected (NET1, NSERV5)
    assert!(svc.get_interface("eth;inject").is_err());
    assert!(svc.get_interface("").is_err());
}

#[test]
fn test_nserv3_route_parsing() {
    let tmp = TempDir::new().expect("create temp dir");
    let sysfs = tmp.path().join("sysfs");
    let procfs = tmp.path().join("proc");
    let resolv = tmp.path().join("resolv.conf");

    fs::create_dir_all(&procfs).expect("create proc dir");

    // Route table in /proc/net/route format
    let route_content = "\
Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n\
eth0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n\
eth0\t0001A8C0\t00000000\t0001\t0\t0\t50\t00FFFFFF\t0\t0\t0\n\
eth1\t0000000A\t00000000\t0001\t0\t0\t10\t000000FF\t0\t0\t0\n";

    fs::write(procfs.join("route"), route_content).expect("write route file");

    let svc = NetworkService::with_paths(&sysfs, &procfs, &resolv);
    let routes = svc.scan_routes().expect("scan routes");

    assert_eq!(routes.len(), 3);

    // Routes are sorted by metric ascending (NET6)
    // 1. Metric 10: eth1 10.0.0.0/8
    assert_eq!(routes[0].metric, 10);
    assert_eq!(routes[0].destination, "10.0.0.0/8");
    assert_eq!(routes[0].interface.as_deref(), Some("eth1"));
    assert_eq!(routes[0].gateway, None);

    // 2. Metric 50: eth0 192.168.1.0/24
    assert_eq!(routes[1].metric, 50);
    assert_eq!(routes[1].destination, "192.168.1.0/24");
    assert_eq!(routes[1].interface.as_deref(), Some("eth0"));
    assert_eq!(routes[1].gateway, None);

    // 3. Metric 100: eth0 0.0.0.0/0 via 192.168.1.1
    assert_eq!(routes[2].metric, 100);
    assert_eq!(routes[2].destination, "0.0.0.0/0");
    assert_eq!(routes[2].gateway.as_deref(), Some("192.168.1.1"));
    assert_eq!(routes[2].interface.as_deref(), Some("eth0"));
}

#[test]
fn test_nserv4_dns_parsing() {
    let tmp = TempDir::new().expect("create temp dir");
    let sysfs = tmp.path().join("sysfs");
    let procfs = tmp.path().join("proc");
    let resolv = tmp.path().join("resolv.conf");

    let resolv_content = "\
# Generated by AIOS network manager
nameserver 1.1.1.1
nameserver 8.8.8.8
nameserver 2606:4700:4700::1111
search aios.local corp.local
";

    fs::write(&resolv, resolv_content).expect("write resolv.conf");

    let svc = NetworkService::with_paths(&sysfs, &procfs, &resolv);
    let dns = svc.get_dns_config().expect("get dns config");

    assert_eq!(dns.nameservers.len(), 3);
    assert_eq!(dns.nameservers[0], "1.1.1.1");
    assert_eq!(dns.nameservers[1], "8.8.8.8");
    assert_eq!(dns.nameservers[2], "2606:4700:4700::1111");

    assert_eq!(dns.search_domains.len(), 2);
    assert_eq!(dns.search_domains[0], "aios.local");
    assert_eq!(dns.search_domains[1], "corp.local");

    assert!(dns.validate().is_ok());
}

#[test]
fn test_nserv5_bring_up_and_bring_down() {
    let tmp = TempDir::new().expect("create temp dir");
    let sysfs = tmp.path().join("sysfs");
    let procfs = tmp.path().join("proc");
    let resolv = tmp.path().join("resolv.conf");

    let eth0_dir = sysfs.join("eth0");
    fs::create_dir_all(&eth0_dir).expect("create eth0 dir");
    let oper_file = eth0_dir.join("operstate");
    fs::write(&oper_file, "down\n").expect("write initial operstate");

    let svc = NetworkService::with_paths(&sysfs, &procfs, &resolv);

    // Bring up
    svc.bring_up("eth0").expect("bring up eth0");
    assert_eq!(fs::read_to_string(&oper_file).expect("read operstate").trim(), "up");

    // Bring down
    svc.bring_down("eth0").expect("bring down eth0");
    assert_eq!(fs::read_to_string(&oper_file).expect("read operstate").trim(), "down");

    // Invalid name rejected
    assert!(svc.bring_up("eth0;reboot").is_err());
    assert!(svc.bring_down("../escape").is_err());
}

#[test]
fn test_nserv6_get_network_state_unified() {
    let tmp = TempDir::new().expect("create temp dir");
    let sysfs = tmp.path().join("sysfs");
    let procfs = tmp.path().join("proc");
    let resolv = tmp.path().join("resolv.conf");

    // Setup interface
    let eth0_dir = sysfs.join("eth0");
    fs::create_dir_all(&eth0_dir).expect("create eth0 dir");
    fs::write(eth0_dir.join("operstate"), "up\n").expect("write operstate");
    fs::write(eth0_dir.join("address"), "00:11:22:33:44:55\n").expect("write address");
    fs::write(eth0_dir.join("mtu"), "1500\n").expect("write mtu");

    // Setup route
    fs::create_dir_all(&procfs).expect("create proc dir");
    let route_content = "\
Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\n\
eth0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n";
    fs::write(procfs.join("route"), route_content).expect("write route");

    // Setup DNS
    fs::write(&resolv, "nameserver 1.1.1.1\n").expect("write resolv.conf");

    let svc = NetworkService::with_paths(&sysfs, &procfs, &resolv);
    let state = svc.get_network_state().expect("get network state");

    assert_eq!(state.interfaces.len(), 1);
    assert_eq!(state.interfaces[0].name, "eth0");
    assert_eq!(state.routes.len(), 1);
    assert_eq!(state.routes[0].destination, "0.0.0.0/0");
    assert_eq!(state.dns.nameservers.len(), 1);
    assert_eq!(state.dns.nameservers[0], "1.1.1.1");
    assert!(state.validate_invariants().is_ok());
}

#[test]
fn test_nserv_hardening_file_bounds() {
    use aiosh_core::network_service::{read_bounded_string, MAX_SYSFS_FILE_BYTES};

    let tmp = TempDir::new().expect("create temp dir");
    let test_file = tmp.path().join("bounded.txt");

    // Write 100 KB
    let big_data = "A".repeat(100 * 1024);
    fs::write(&test_file, big_data).expect("write big file");

    // Read bounded to MAX_SYSFS_FILE_BYTES (64 KB)
    let bounded = read_bounded_string(&test_file, MAX_SYSFS_FILE_BYTES).expect("read bounded");
    assert_eq!(bounded.len(), MAX_SYSFS_FILE_BYTES as usize);
}
