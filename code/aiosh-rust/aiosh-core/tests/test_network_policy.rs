//! Unit tests for Network Bootstrap Security Policy Subsystem (NPOL1..NPOL6).

use tempfile::tempdir;

use aiosh_core::network::{
    DnsConfig, InterfaceType, IpAddress, NetworkInterface, NetworkState, OperState, Route,
};
use aiosh_core::network_policy::{
    validate_policy_path, NetworkPolicyMode, NetworkSecurityPolicy, MAX_POLICY_FILE_BYTES,
};

fn create_sample_network_state() -> NetworkState {
    let mut state = NetworkState::new("test-host");

    // lo
    let mut lo = NetworkInterface::new("lo", InterfaceType::Loopback);
    lo.operstate = OperState::Unknown;
    lo.mtu = 65535;
    lo.ip_addresses.push(IpAddress::new_v4("127.0.0.1", 8));
    lo.flags = vec!["UP".into(), "LOOPBACK".into()];
    state.interfaces.push(lo);

    // eth0
    let mut eth0 = NetworkInterface::new("eth0", InterfaceType::Ethernet);
    eth0.operstate = OperState::Up;
    eth0.mtu = 1500;
    eth0.mac_address = Some("00:11:22:33:44:55".into());
    eth0.ip_addresses.push(IpAddress::new_v4("192.168.1.50", 24));
    eth0.flags = vec!["UP".into(), "BROADCAST".into(), "MULTICAST".into()];
    state.interfaces.push(eth0);

    // route
    let mut default_route = Route::new("0.0.0.0/0", 100);
    default_route.gateway = Some("192.168.1.1".into());
    default_route.interface = Some("eth0".into());
    state.routes.push(default_route);

    // dns
    state.dns = DnsConfig::default()
        .with_nameserver("1.1.1.1")
        .with_nameserver("8.8.8.8");

    state
}

#[test]
fn test_policy_default_valid() {
    let policy = NetworkSecurityPolicy::default();
    assert!(policy.validate().is_ok());
    assert_eq!(policy.mode, NetworkPolicyMode::Enforcing);
    assert!(!policy.allow_promiscuous);
    assert!(policy.require_mac_for_ethernet);
    assert_eq!(policy.max_interfaces_allowed, 1024);
    assert_eq!(policy.max_routes_allowed, 4096);
    assert_eq!(policy.max_dns_servers_allowed, 32);
}

#[test]
fn test_npol1_disallowed_interface_type() {
    let mut policy = NetworkSecurityPolicy::default();
    policy.disallowed_interface_types.push(InterfaceType::TunTap);

    let mut state = create_sample_network_state();
    let mut tun = NetworkInterface::new("tun0", InterfaceType::TunTap);
    tun.operstate = OperState::Up;
    tun.mtu = 1500;
    state.interfaces.push(tun);

    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_IFACE_DISALLOWED_TYPE" && v.target == "tun0"));
}

#[test]
fn test_npol1_prohibited_interface_name() {
    let mut policy = NetworkSecurityPolicy::default();
    policy.prohibited_interface_names.push("wlan0".into());

    let mut state = create_sample_network_state();
    let mut wlan = NetworkInterface::new("wlan0", InterfaceType::Wireless);
    wlan.operstate = OperState::Down;
    wlan.mtu = 1500;
    state.interfaces.push(wlan);

    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_IFACE_PROHIBITED_NAME" && v.target == "wlan0"));
}

#[test]
fn test_npol1_whitelist_interface_name() {
    let mut policy = NetworkSecurityPolicy::default();
    // Only allow lo and eth0
    policy.allowed_interface_names = Some(vec!["lo".into(), "eth0".into()]);

    let mut state = create_sample_network_state();
    let mut eth1 = NetworkInterface::new("eth1", InterfaceType::Ethernet);
    eth1.operstate = OperState::Up;
    eth1.mtu = 1500;
    state.interfaces.push(eth1);

    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_IFACE_NOT_WHITELISTED" && v.target == "eth1"));
}

#[test]
fn test_npol1_promiscuous_mode_violation() {
    let policy = NetworkSecurityPolicy::default(); // allow_promiscuous = false
    let mut state = create_sample_network_state();
    state.interfaces[1].flags.push("PROMISC".into());

    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_IFACE_PROMISCUOUS" && v.target == "eth0"));
}

#[test]
fn test_npol1_missing_mac_on_ethernet() {
    let policy = NetworkSecurityPolicy::default(); // require_mac_for_ethernet = true
    let mut state = create_sample_network_state();
    state.interfaces[1].mac_address = None;

    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_IFACE_MISSING_MAC" && v.target == "eth0"));
}

#[test]
fn test_npol2_orphan_route_rejected() {
    let policy = NetworkSecurityPolicy::default();
    let mut state = create_sample_network_state();
    let mut orphan_route = Route::new("10.0.0.0/8", 50);
    orphan_route.interface = Some("non_existent_iface".into());
    state.routes.push(orphan_route);

    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_ROUTE_ORPHAN_IFACE"));
}

#[test]
fn test_npol3_disallowed_dns_server() {
    let mut policy = NetworkSecurityPolicy::default();
    policy.disallowed_dns_servers.push("8.8.8.8".into());

    let state = create_sample_network_state();
    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_DNS_DISALLOWED_SERVER"));
}

#[test]
fn test_npol3_whitelist_dns_server() {
    let mut policy = NetworkSecurityPolicy::default();
    policy.allowed_dns_servers = Some(vec!["1.1.1.1".into()]); // 8.8.8.8 not in whitelist

    let state = create_sample_network_state();
    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_DNS_NOT_WHITELISTED"));
}

#[test]
fn test_npol4_capacity_limits() {
    let mut policy = NetworkSecurityPolicy::default();
    policy.max_interfaces_allowed = 1;

    let state = create_sample_network_state(); // has 2 interfaces
    let report = policy.evaluate(&state);
    assert_eq!(report.verdict, "deny");
    assert!(report.violations.iter().any(|v| v.rule_id == "RULE_IFACE_MAX_CAP"));
}

#[test]
fn test_modes_enforcing_audit_permissive() {
    let mut policy = NetworkSecurityPolicy::default();
    policy.prohibited_interface_names.push("eth0".into());

    let state = create_sample_network_state();

    // 1. Enforcing
    policy.mode = NetworkPolicyMode::Enforcing;
    let rep_enforcing = policy.evaluate(&state);
    assert_eq!(rep_enforcing.verdict, "deny");

    // 2. Audit
    policy.mode = NetworkPolicyMode::Audit;
    let rep_audit = policy.evaluate(&state);
    assert_eq!(rep_audit.verdict, "audit");

    // 3. Permissive
    policy.mode = NetworkPolicyMode::Permissive;
    let rep_permissive = policy.evaluate(&state);
    assert_eq!(rep_permissive.verdict, "allow");
}

#[test]
fn test_npol5_apply_and_sanitize_redaction() {
    let mut policy = NetworkSecurityPolicy::default();
    policy.redact_sensitive_addresses = true;

    let mut state = create_sample_network_state();
    let report = policy.apply_and_sanitize(&mut state);
    assert!(report.redacted);

    let eth0 = state.interfaces.iter().find(|i| i.name == "eth0").unwrap();
    assert_eq!(eth0.mac_address.as_deref(), Some("00:11:22:xx:xx:xx"));
    assert_eq!(eth0.ip_addresses[0].address, "192.168.1.xxx");
}

#[test]
fn test_npol6_policy_path_hygiene_and_persistence() {
    let dir = tempdir().unwrap();
    let policy_path = dir.path().join("policy.json");

    let mut policy = NetworkSecurityPolicy::default();
    policy.prohibited_interface_names.push("wlan0".into());
    policy.max_interfaces_allowed = 500;

    policy.save_to_path(&policy_path).unwrap();
    assert!(policy_path.exists());

    let loaded = NetworkSecurityPolicy::load_from_path(&policy_path).unwrap();
    assert_eq!(policy, loaded);

    // Path hygiene checks
    assert!(validate_policy_path(&dir.path().join("../bad")).is_err());
    assert!(validate_policy_path(&dir.path().join("has\0null")).is_err());
}

#[test]
fn test_npol6_oversized_policy_rejected() {
    let dir = tempdir().unwrap();
    let big_path = dir.path().join("oversized.json");
    std::fs::write(&big_path, vec![b' '; (MAX_POLICY_FILE_BYTES + 10) as usize]).unwrap();

    let res = NetworkSecurityPolicy::load_from_path(&big_path);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exceeds maximum allowed"));
}
