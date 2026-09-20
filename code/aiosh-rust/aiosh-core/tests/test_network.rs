//! Unit tests for Network Bootstrap Data Model (NET1..NET6).

use aiosh_core::network::{
    validate_interface_name, validate_ip_address, validate_mac_address, validate_mtu,
    validate_network_interface, validate_network_state, validate_route, DnsConfig, InterfaceType,
    IpAddress, IpFamily, NetworkInterface, NetworkState, OperState, Route, MAX_IFACE_NAME_LEN,
    MAX_MTU, MIN_MTU,
};

#[test]
fn test_net1_interface_name_validation() {
    // Valid names
    assert!(validate_interface_name("lo").is_ok());
    assert!(validate_interface_name("eth0").is_ok());
    assert!(validate_interface_name("enp3s0").is_ok());
    assert!(validate_interface_name("wlan0").is_ok());
    assert!(validate_interface_name("br-lan").is_ok());
    assert!(validate_interface_name("vlan.100").is_ok());

    // Invalid: empty
    assert!(validate_interface_name("").is_err());
    assert!(validate_interface_name("   ").is_err());

    // Invalid: exceeds 15 characters (IFNAMSIZ - 1)
    let long_name = "a".repeat(MAX_IFACE_NAME_LEN + 1);
    assert!(validate_interface_name(&long_name).is_err());

    // Invalid: special characters and spaces
    assert!(validate_interface_name("eth 0").is_err());
    assert!(validate_interface_name("eth;rm").is_err());
    assert!(validate_interface_name("eth/0").is_err());
    assert!(validate_interface_name("eth\x00").is_err());
}

#[test]
fn test_net2_mac_address_validation() {
    // Valid MACs
    assert!(validate_mac_address("00:11:22:33:44:55").is_ok());
    assert!(validate_mac_address("aa:bb:cc:dd:ee:ff").is_ok());
    assert!(validate_mac_address("AA:BB:CC:DD:EE:FF").is_ok());
    assert!(validate_mac_address("").is_ok()); // Empty allowed for loopback

    // Invalid: wrong octet count
    assert!(validate_mac_address("00:11:22:33:44").is_err());
    assert!(validate_mac_address("00:11:22:33:44:55:66").is_err());

    // Invalid: non-hex characters
    assert!(validate_mac_address("00:11:22:33:44:GG").is_err());
    assert!(validate_mac_address("00:11:22:33:44:5 ").is_err());
}

#[test]
fn test_net3_ip_address_and_cidr_validation() {
    // Valid IPv4
    let v4 = IpAddress::new_v4("192.168.1.100", 24);
    assert!(v4.validate().is_ok());

    // Valid IPv6
    let v6 = IpAddress::new_v6("fe80::1", 64);
    assert!(v6.validate().is_ok());

    // Valid CIDR parsing
    let cidr_v4 = IpAddress::from_cidr("10.0.0.1/8").expect("parse cidr v4");
    assert_eq!(cidr_v4.address, "10.0.0.1");
    assert_eq!(cidr_v4.prefix_len, 8);
    assert_eq!(cidr_v4.family, IpFamily::V4);

    let cidr_v6 = IpAddress::from_cidr("2001:db8::1/64").expect("parse cidr v6");
    assert_eq!(cidr_v6.address, "2001:db8::1");
    assert_eq!(cidr_v6.prefix_len, 64);
    assert_eq!(cidr_v6.family, IpFamily::V6);

    // Invalid IPv4 prefix
    let v4_bad = IpAddress::new_v4("192.168.1.1", 33);
    assert!(v4_bad.validate().is_err());

    // Invalid IPv6 prefix
    let v6_bad = IpAddress::new_v6("::1", 129);
    assert!(v6_bad.validate().is_err());

    // Malformed IP
    assert!(IpAddress::from_cidr("999.999.999.999/24").is_err());
    assert!(IpAddress::from_cidr("invalid-ip/24").is_err());
    assert!(IpAddress::from_cidr("192.168.1.1").is_err()); // Missing slash
}

#[test]
fn test_net4_mtu_bounds() {
    assert!(validate_mtu(MIN_MTU).is_ok()); // 68
    assert!(validate_mtu(1500).is_ok());
    assert!(validate_mtu(9000).is_ok());
    assert!(validate_mtu(MAX_MTU).is_ok()); // 65535

    assert!(validate_mtu(MIN_MTU - 1).is_err());
    assert!(validate_mtu(MAX_MTU + 1).is_err());
    assert!(validate_mtu(0).is_err());
}

#[test]
fn test_net5_route_validation() {
    // Valid route with gateway
    let r1 = Route::new("0.0.0.0/0", 100).with_gateway("192.168.1.1");
    assert!(r1.validate().is_ok());

    // Valid route with interface
    let r2 = Route::new("192.168.1.0/24", 50).with_interface("eth0");
    assert!(r2.validate().is_ok());

    // Valid route with both
    let r3 = Route::new("10.0.0.0/8", 10).with_gateway("10.0.0.1").with_interface("eth1");
    assert!(r3.validate().is_ok());

    // Invalid: neither gateway nor interface
    let r_bad = Route::new("0.0.0.0/0", 100);
    assert!(r_bad.validate().is_err());

    // Invalid: empty destination
    let r_empty = Route::new("", 100).with_gateway("192.168.1.1");
    assert!(r_empty.validate().is_err());

    // Invalid: malformed destination CIDR
    let r_malformed = Route::new("bad-ip/24", 100).with_gateway("192.168.1.1");
    assert!(r_malformed.validate().is_err());
}

#[test]
fn test_net6_network_state_deterministic_ordering_and_queries() {
    let mut state = NetworkState::new("test-gateway");

    // Add DNS
    state.dns = DnsConfig {
        nameservers: vec!["1.1.1.1".into(), "8.8.8.8".into()],
        search_domains: vec!["aios.local".into()],
    };

    // Add interfaces in non-alphabetical order: wlan0, lo, eth0
    let if_wlan = NetworkInterface::new("wlan0", InterfaceType::Wireless)
        .with_operstate(OperState::Down)
        .with_mac("00:11:22:33:44:56");

    let if_lo = NetworkInterface::new("lo", InterfaceType::Loopback)
        .with_operstate(OperState::Up)
        .with_mtu(65536 - 1)
        .with_ip(IpAddress::new_v4("127.0.0.1", 8))
        .with_ip(IpAddress::new_v6("::1", 128))
        .with_flag("UP")
        .with_flag("LOOPBACK");

    let if_eth = NetworkInterface::new("eth0", InterfaceType::Ethernet)
        .with_operstate(OperState::Up)
        .with_mac("00:11:22:33:44:55")
        .with_mtu(1500)
        .with_ip(IpAddress::new_v4("192.168.1.50", 24))
        .with_flag("UP")
        .with_flag("RUNNING");

    state.add_interface(if_wlan).expect("add wlan");
    state.add_interface(if_lo).expect("add lo");
    state.add_interface(if_eth).expect("add eth");

    // Verify alphabetical ordering (NET6): eth0, lo, wlan0
    assert_eq!(state.interfaces.len(), 3);
    assert_eq!(state.interfaces[0].name, "eth0");
    assert_eq!(state.interfaces[1].name, "lo");
    assert_eq!(state.interfaces[2].name, "wlan0");

    // Add routes in non-metric order
    let r_local = Route::new("192.168.1.0/24", 50).with_interface("eth0");
    let r_default = Route::new("0.0.0.0/0", 100).with_gateway("192.168.1.1");
    let r_default_v6 = Route::new("::/0", 100).with_gateway("fe80::1");

    state.add_route(r_default).expect("add default route");
    state.add_route(r_local).expect("add local route");
    state.add_route(r_default_v6).expect("add default v6 route");

    // Routes ordered by metric then destination
    assert_eq!(state.routes[0].metric, 50);
    assert_eq!(state.routes[0].destination, "192.168.1.0/24");

    // Test queries
    assert_eq!(state.default_gateway(), Some("192.168.1.1"));
    assert_eq!(state.default_gateway_v6(), Some("fe80::1"));

    let up_ifaces = state.interfaces_up();
    assert_eq!(up_ifaces.len(), 2); // eth0 and lo
    assert_eq!(up_ifaces[0].name, "eth0");
    assert_eq!(up_ifaces[1].name, "lo");

    let found = state.find_interface_by_ip("192.168.1.50");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "eth0");
    assert_eq!(found.unwrap().primary_ipv4(), Some("192.168.1.50"));

    // Validate complete state
    assert!(state.validate_invariants().is_ok());

    // JSON serialization roundtrip
    let json_str = state.to_json_pretty().expect("to json");
    let restored = NetworkState::from_json(&json_str).expect("from json");
    assert_eq!(restored.hostname, "test-gateway");
    assert_eq!(restored.interfaces.len(), 3);
    assert_eq!(restored.routes.len(), 3);
}
