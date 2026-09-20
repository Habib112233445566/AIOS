//! Unit tests for Network Bootstrap Observability Subsystem (NOBS1..NOBS6).

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use aiosh_core::network::{
    DnsConfig, InterfaceType, IpAddress, NetworkInterface, NetworkState, OperState, Route,
};
use aiosh_core::network_observability::{
    validate_observability_path, InterfaceStatistics, NetworkHealthVerdict,
    NetworkObservabilityService, MAX_OBSERVABILITY_FILE_BYTES,
};

fn create_test_state() -> NetworkState {
    let mut state = NetworkState::new("test-obs-host");

    // lo
    let mut lo = NetworkInterface::new("lo", InterfaceType::Loopback);
    lo.operstate = OperState::Unknown;
    lo.ip_addresses.push(IpAddress::new_v4("127.0.0.1", 8));
    state.interfaces.push(lo);

    // eth0
    let mut eth0 = NetworkInterface::new("eth0", InterfaceType::Ethernet);
    eth0.operstate = OperState::Up;
    eth0.mac_address = Some("00:11:22:33:44:55".into());
    eth0.ip_addresses.push(IpAddress::new_v4("192.168.1.50", 24));
    state.interfaces.push(eth0);

    // Default route
    let mut route = Route::new("0.0.0.0/0", 100);
    route.gateway = Some("192.168.1.1".into());
    route.interface = Some("eth0".into());
    state.routes.push(route);

    // DNS
    state.dns = DnsConfig::default().with_nameserver("1.1.1.1");

    state
}

#[test]
fn test_nobs1_procfs_parsing() {
    let dir = tempdir().unwrap();
    let procfs_dev = dir.path().join("dev");
    let content = "Inter-|   Receive                                                |  Transmit\n face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n    lo: 1000       10    0    0    0     0          0         0     1000       10    0    0    0     0       0          0\n  eth0: 500000    4000    2    1    0     0          0         0   300000    2500    0    0    0     5       0          0\n";
    fs::write(&procfs_dev, content).unwrap();

    let service = NetworkObservabilityService::with_paths(
        procfs_dev,
        dir.path().join("sysfs_empty"),
        10,
    );

    let stats = service.collect_statistics();
    assert_eq!(stats.len(), 2);

    let eth0 = stats.iter().find(|s| s.interface_name == "eth0").unwrap();
    assert_eq!(eth0.rx_bytes, 500000);
    assert_eq!(eth0.rx_packets, 4000);
    assert_eq!(eth0.rx_errors, 2);
    assert_eq!(eth0.rx_dropped, 1);
    assert_eq!(eth0.tx_bytes, 300000);
    assert_eq!(eth0.tx_packets, 2500);
    assert_eq!(eth0.collisions, 5);
}

#[test]
fn test_nobs2_missing_procfs_fallback() {
    let dir = tempdir().unwrap();
    let non_existent_dev = dir.path().join("non_existent_procfs_dev");
    let non_existent_sys = dir.path().join("non_existent_sysfs");

    let service = NetworkObservabilityService::with_paths(non_existent_dev, non_existent_sys, 10);
    let stats = service.collect_statistics();
    assert!(stats.is_empty());
}

#[test]
fn test_nobs2_sysfs_carrier_enrichment() {
    let dir = tempdir().unwrap();
    let sysfs = dir.path().join("net");
    fs::create_dir_all(sysfs.join("eth0")).unwrap();
    fs::write(sysfs.join("eth0/carrier"), "1\n").unwrap();

    fs::create_dir_all(sysfs.join("eth1")).unwrap();
    fs::write(sysfs.join("eth1/carrier"), "0\n").unwrap();

    let service = NetworkObservabilityService::with_paths(
        dir.path().join("no_procfs"),
        sysfs,
        10,
    );

    let stats = service.collect_statistics();
    assert_eq!(stats.len(), 2);

    let eth0 = stats.iter().find(|s| s.interface_name == "eth0").unwrap();
    assert_eq!(eth0.carrier, Some(true));

    let eth1 = stats.iter().find(|s| s.interface_name == "eth1").unwrap();
    assert_eq!(eth1.carrier, Some(false));
}

#[test]
fn test_nobs3_health_healthy() {
    let service = NetworkObservabilityService::new();
    let state = create_test_state();
    let stats = vec![InterfaceStatistics {
        interface_name: "eth0".into(),
        rx_packets: 1000,
        rx_dropped: 0,
        carrier: Some(true),
        ..Default::default()
    }];

    let report = service.evaluate_health(&state, &stats);
    assert_eq!(report.verdict, NetworkHealthVerdict::Healthy);
    assert!(report.issues.is_empty());
    assert!(report.default_route_present);
    assert!(report.dns_configured);
}

#[test]
fn test_nobs3_health_degraded_no_default_route() {
    let service = NetworkObservabilityService::new();
    let mut state = create_test_state();
    state.routes.clear(); // remove default route

    let stats = vec![InterfaceStatistics {
        interface_name: "eth0".into(),
        carrier: Some(true),
        ..Default::default()
    }];

    let report = service.evaluate_health(&state, &stats);
    assert_eq!(report.verdict, NetworkHealthVerdict::Degraded);
    assert!(!report.default_route_present);
    assert!(report.issues.iter().any(|i| i.contains("No default gateway")));
}

#[test]
fn test_nobs3_health_degraded_no_dns() {
    let service = NetworkObservabilityService::new();
    let mut state = create_test_state();
    state.dns.nameservers.clear();

    let stats = vec![InterfaceStatistics {
        interface_name: "eth0".into(),
        carrier: Some(true),
        ..Default::default()
    }];

    let report = service.evaluate_health(&state, &stats);
    assert_eq!(report.verdict, NetworkHealthVerdict::Degraded);
    assert!(!report.dns_configured);
    assert!(report.issues.iter().any(|i| i.contains("No DNS nameservers")));
}

#[test]
fn test_nobs3_health_degraded_high_drops() {
    let service = NetworkObservabilityService::new();
    let state = create_test_state();
    let stats = vec![InterfaceStatistics {
        interface_name: "eth0".into(),
        rx_packets: 1000,
        rx_dropped: 100, // 10% drop rate (> 5%)
        carrier: Some(true),
        ..Default::default()
    }];

    let report = service.evaluate_health(&state, &stats);
    assert_eq!(report.verdict, NetworkHealthVerdict::Degraded);
    assert!(report.issues.iter().any(|i| i.contains("Elevated error/drop rate")));
}

#[test]
fn test_nobs3_health_critical_all_interfaces_down() {
    let service = NetworkObservabilityService::new();
    let mut state = create_test_state();
    for iface in &mut state.interfaces {
        if iface.name != "lo" {
            iface.operstate = OperState::Down;
        }
    }

    let stats = vec![InterfaceStatistics {
        interface_name: "eth0".into(),
        carrier: Some(false),
        ..Default::default()
    }];

    let report = service.evaluate_health(&state, &stats);
    assert_eq!(report.verdict, NetworkHealthVerdict::Critical);
}

#[test]
fn test_nobs3_health_critical_no_route_and_no_dns() {
    let service = NetworkObservabilityService::new();
    let mut state = create_test_state();
    state.routes.clear();
    state.dns.nameservers.clear();

    let stats = vec![InterfaceStatistics {
        interface_name: "eth0".into(),
        carrier: Some(true),
        ..Default::default()
    }];

    let report = service.evaluate_health(&state, &stats);
    assert_eq!(report.verdict, NetworkHealthVerdict::Critical);
}

#[test]
fn test_nobs4_history_ring_buffer_eviction() {
    let mut service = NetworkObservabilityService::with_paths(
        PathBuf::from("/no_procfs"),
        PathBuf::from("/no_sysfs"),
        3, // capacity 3
    );
    let state = create_test_state();

    for i in 1..=5 {
        let mut s = state.clone();
        s.timestamp = format!("2026-09-20T10:0{}:00Z", i);
        service.capture_snapshot(&s);
    }

    let history = service.get_history();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].timestamp, "2026-09-20T10:03:00Z");
    assert_eq!(history[1].timestamp, "2026-09-20T10:04:00Z");
    assert_eq!(history[2].timestamp, "2026-09-20T10:05:00Z");
}

#[test]
fn test_nobs6_persistence_atomic_and_path_hygiene() {
    let dir = tempdir().unwrap();
    let snapshot_path = dir.path().join("obs_snapshot.json");
    let mut service = NetworkObservabilityService::new();
    let state = create_test_state();
    let snapshot = service.capture_snapshot(&state);

    service.save_snapshot_to_path(&snapshot, &snapshot_path).unwrap();
    assert!(snapshot_path.exists());

    let loaded = NetworkObservabilityService::load_snapshot_from_path(&snapshot_path).unwrap();
    assert_eq!(snapshot, loaded);

    // Path hygiene checks
    assert!(validate_observability_path(&dir.path().join("../bad")).is_err());
    assert!(validate_observability_path(&dir.path().join("has\0null")).is_err());
}

#[test]
fn test_nobs6_oversized_snapshot_rejected() {
    let dir = tempdir().unwrap();
    let big_path = dir.path().join("oversized_snapshot.json");
    fs::write(&big_path, vec![b' '; (MAX_OBSERVABILITY_FILE_BYTES + 10) as usize]).unwrap();

    let res = NetworkObservabilityService::load_snapshot_from_path(&big_path);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("exceeds maximum"));
}
