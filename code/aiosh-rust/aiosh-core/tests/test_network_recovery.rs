use std::fs;
use std::path::Path;
use tempfile::tempdir;

use aiosh_core::network::{
    InterfaceType, NetworkInterface, NetworkState, OperState, Route,
};
use aiosh_core::network_recovery::{
    check_network_file, recover_network_file, recover_network_state_in_memory,
    validate_network_state as validate_network_state_integrity,
    validate_network_store_path, NetworkRecoveryAction, MAX_NETWORK_STORE_SIZE,
};

#[test]
fn test_nval1_interface_counts_parity() {
    let mut state = NetworkState::new("node-01");
    state.interfaces.push(NetworkInterface::new("lo", InterfaceType::Loopback));
    state.interfaces.push(NetworkInterface::new("", InterfaceType::Ethernet)); // invalid: empty name
    state.dns.nameservers.push("1.1.1.1".into());

    let report = validate_network_state_integrity(&state, Path::new("/test/state.json"));
    assert_eq!(report.total_interfaces, 2);
    assert_eq!(report.valid_interfaces, 1);
    assert_eq!(report.invalid_interfaces, 1);
    assert!(!report.healthy);
    assert!(report.validate_invariants().is_ok());
}

#[test]
fn test_nval2_dangling_routes_detection_and_pruning() {
    let mut state = NetworkState::new("node-01");
    let lo = NetworkInterface::new("lo", InterfaceType::Loopback);
    state.interfaces.push(lo);
    state.dns.nameservers.push("1.1.1.1".into());

    let mut route_good = Route::new("127.0.0.0/8", 100);
    route_good.interface = Some("lo".into());

    let mut route_dangling = Route::new("10.0.0.0/24", 200);
    route_dangling.interface = Some("eth999".into()); // nonexistent dev

    state.routes.push(route_good);
    state.routes.push(route_dangling);

    let initial = validate_network_state_integrity(&state, Path::new("/test/state.json"));
    assert_eq!(initial.dangling_routes.len(), 1);
    assert!(!initial.healthy);

    let report = recover_network_state_in_memory(&mut state, Path::new("/test/state.json"));
    assert!(report.recovered);
    assert_eq!(state.routes.len(), 1);
    assert_eq!(state.routes[0].destination, "127.0.0.0/8");
    assert!(report.actions_taken.iter().any(|a| matches!(a, NetworkRecoveryAction::PruneDanglingRoutes { pruned_count: 1 })));
}

#[test]
fn test_nval3_dns_missing_and_fallback() {
    let mut state = NetworkState::new("node-01");
    state.interfaces.push(NetworkInterface::new("lo", InterfaceType::Loopback));
    // DNS nameservers empty

    let initial = validate_network_state_integrity(&state, Path::new("/test/state.json"));
    assert!(!initial.dns_configured);
    assert!(!initial.healthy);

    let report = recover_network_state_in_memory(&mut state, Path::new("/test/state.json"));
    assert!(report.recovered);
    assert!(state.dns.nameservers.contains(&"1.1.1.1".to_string()));
    assert!(state.dns.nameservers.contains(&"8.8.8.8".to_string()));
    assert!(report.actions_taken.iter().any(|a| matches!(a, NetworkRecoveryAction::SetDefaultDnsFallback { .. })));
}

#[test]
fn test_nval4_loopback_restoration() {
    let mut state = NetworkState::new("node-01");
    // No interfaces at all
    state.dns.nameservers.push("1.1.1.1".into());

    let initial = validate_network_state_integrity(&state, Path::new("/test/state.json"));
    assert!(initial.missing_loopback);
    assert!(!initial.healthy);

    let report = recover_network_state_in_memory(&mut state, Path::new("/test/state.json"));
    assert!(report.recovered);
    assert_eq!(state.interfaces.len(), 1);
    assert_eq!(state.interfaces[0].name, "lo");
    assert_eq!(state.interfaces[0].operstate, OperState::Up);
    assert!(report.actions_taken.contains(&NetworkRecoveryAction::RestoreLoopback));
}

#[test]
fn test_nval4_healthy_state_passes_clean() {
    let mut state = NetworkState::new("node-01");
    state.interfaces.push(NetworkInterface::new("lo", InterfaceType::Loopback));
    state.dns.nameservers.push("1.1.1.1".into());

    let report = recover_network_state_in_memory(&mut state, Path::new("/test/state.json"));
    assert!(report.recovered);
    assert_eq!(report.actions_taken, vec![NetworkRecoveryAction::NoneRequired]);
    assert!(report.final_validation.healthy);
}

#[test]
fn test_nval5_quarantine_corrupted_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("network_state.json");
    fs::write(&file_path, "{ corrupted json syntax !!!").unwrap();

    let initial = check_network_file(&file_path);
    assert!(!initial.healthy);
    assert!(initial.errors[0].contains("JSON parse error"));

    let report = recover_network_file(&file_path).unwrap();
    assert!(report.recovered);
    assert!(report.backup_path.is_some());

    let backup_path_str = report.backup_path.unwrap();
    let backup_path = Path::new(&backup_path_str);
    assert!(backup_path.exists());
    assert_eq!(fs::read_to_string(backup_path).unwrap(), "{ corrupted json syntax !!!");

    // Re-check original file: now healthy
    let final_check = check_network_file(&file_path);
    assert!(final_check.healthy);
    assert!(final_check.validate_invariants().is_ok());
}

#[test]
fn test_nval6_path_hygiene_and_validation() {
    let dir = tempdir().unwrap();
    let valid_path = dir.path().join("state.json");
    assert!(validate_network_store_path(&valid_path).is_ok());

    // Non-json extension
    let bad_ext = dir.path().join("state.txt");
    assert!(validate_network_store_path(&bad_ext).is_err());

    // Traversal
    let bad_traversal = dir.path().join("../state.json");
    assert!(validate_network_store_path(&bad_traversal).is_err());

    // Control char
    let bad_ctrl = dir.path().join("bad\0name.json");
    assert!(validate_network_store_path(&bad_ctrl).is_err());
}

#[test]
fn test_nval6_oversized_store_rejected() {
    let dir = tempdir().unwrap();
    let big_path = dir.path().join("oversized.json");
    let big_data = " ".repeat((MAX_NETWORK_STORE_SIZE + 100) as usize);
    fs::write(&big_path, big_data).unwrap();

    let report = check_network_file(&big_path);
    assert!(!report.healthy);
    assert!(report.errors[0].contains("exceeds maximum permitted limit"));
}
