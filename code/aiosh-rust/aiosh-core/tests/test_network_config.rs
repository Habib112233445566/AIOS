//! Unit tests for Network Configuration Subsystem (NCONF1..NCONF6)

use std::fs;
use std::path::PathBuf;
use aiosh_core::network_config::{NetworkConfig, MAX_CONFIG_FILE_BYTES};
use tempfile::tempdir;

#[test]
fn test_network_config_default_valid() {
    let cfg = NetworkConfig::default();
    assert!(cfg.validate().is_ok());
    assert_eq!(cfg.default_store_path, PathBuf::from(".aios/network_state.json"));
    assert_eq!(cfg.sysfs_net_path, PathBuf::from("/sys/class/net"));
    assert_eq!(cfg.procfs_path, PathBuf::from("/proc/net"));
    assert_eq!(cfg.resolv_conf_path, PathBuf::from("/etc/resolv.conf"));
    assert_eq!(cfg.max_interfaces, 1024);
    assert_eq!(cfg.max_routes, 4096);
    assert_eq!(cfg.max_dns_servers, 32);
    assert_eq!(cfg.max_payload_bytes, 10_485_760);
    assert_eq!(cfg.scan_timeout_secs, 30);
    assert_eq!(cfg.fallback_dns_servers, vec!["1.1.1.1", "8.8.8.8"]);
}

#[test]
fn test_nconf1_path_hygiene_empty() {
    let mut cfg = NetworkConfig::default();
    cfg.default_store_path = PathBuf::from("");
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.sysfs_net_path = PathBuf::from("   ");
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.procfs_path = PathBuf::from("");
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.resolv_conf_path = PathBuf::from("  ");
    assert!(cfg.validate().is_err());
}

#[test]
fn test_nconf1_path_hygiene_control_chars() {
    let mut cfg = NetworkConfig::default();
    cfg.default_store_path = PathBuf::from("path/with/\n/newline");
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.sysfs_net_path = PathBuf::from("/sys/class/net\0bad");
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.procfs_path = PathBuf::from("/proc/net\tbad");
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.resolv_conf_path = PathBuf::from("/etc/resolv\r.conf");
    assert!(cfg.validate().is_err());
}

#[test]
fn test_nconf1_path_hygiene_max_length() {
    let mut cfg = NetworkConfig::default();
    let long_path = "a".repeat(1025);
    cfg.default_store_path = PathBuf::from(long_path);
    assert!(cfg.validate().is_err());
}

#[test]
fn test_nconf1_path_hygiene_traversal() {
    let mut cfg = NetworkConfig::default();
    cfg.default_store_path = PathBuf::from("../escaped/store.json");
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("parent directory traversal"));

    let mut cfg = NetworkConfig::default();
    cfg.sysfs_net_path = PathBuf::from("/sys/../etc");
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.resolv_conf_path = PathBuf::from("/etc/resolv/../../etc/passwd");
    assert!(cfg.validate().is_err());
}

#[test]
fn test_nconf2_capacity_limits_max_interfaces() {
    let mut cfg = NetworkConfig::default();
    cfg.max_interfaces = 0;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.max_interfaces = 10_001;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.max_interfaces = 10_000;
    assert!(cfg.validate().is_ok());

    let mut cfg = NetworkConfig::default();
    cfg.max_interfaces = 1;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_nconf2_capacity_limits_max_routes() {
    let mut cfg = NetworkConfig::default();
    cfg.max_routes = 0;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.max_routes = 50_001;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.max_routes = 50_000;
    assert!(cfg.validate().is_ok());

    let mut cfg = NetworkConfig::default();
    cfg.max_routes = 1;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_nconf2_capacity_limits_max_dns_servers() {
    let mut cfg = NetworkConfig::default();
    cfg.max_dns_servers = 0;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.max_dns_servers = 65;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.max_dns_servers = 64;
    assert!(cfg.validate().is_ok());

    let mut cfg = NetworkConfig::default();
    cfg.max_dns_servers = 2;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_nconf3_resource_bounds_max_payload() {
    let mut cfg = NetworkConfig::default();
    cfg.max_payload_bytes = 1023;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.max_payload_bytes = 104_857_601;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.max_payload_bytes = 1024;
    assert!(cfg.validate().is_ok());

    let mut cfg = NetworkConfig::default();
    cfg.max_payload_bytes = 104_857_600;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_nconf3_timeout_bounds() {
    let mut cfg = NetworkConfig::default();
    cfg.scan_timeout_secs = 0;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.scan_timeout_secs = 301;
    assert!(cfg.validate().is_err());

    let mut cfg = NetworkConfig::default();
    cfg.scan_timeout_secs = 300;
    assert!(cfg.validate().is_ok());

    let mut cfg = NetworkConfig::default();
    cfg.scan_timeout_secs = 1;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_nconf4_fallback_dns_valid() {
    let mut cfg = NetworkConfig::default();
    cfg.fallback_dns_servers = vec![
        "1.1.1.1".to_string(),
        "8.8.8.8".to_string(),
        "2001:4860:4860::8888".to_string(),
    ];
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_nconf4_fallback_dns_invalid_ip() {
    let mut cfg = NetworkConfig::default();
    cfg.fallback_dns_servers = vec!["not.an.ip.address".to_string()];
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("not a valid IP address"));
}

#[test]
fn test_nconf4_fallback_dns_empty() {
    let mut cfg = NetworkConfig::default();
    cfg.fallback_dns_servers = vec!["".to_string()];
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("cannot be empty"));
}

#[test]
fn test_nconf4_fallback_dns_exceeds_max() {
    let mut cfg = NetworkConfig::default();
    cfg.max_dns_servers = 2;
    cfg.fallback_dns_servers = vec![
        "1.1.1.1".to_string(),
        "8.8.8.8".to_string(),
        "9.9.9.9".to_string(),
    ];
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("exceeds max_dns_servers"));
}

#[test]
fn test_nconf6_json_roundtrip() {
    let mut cfg = NetworkConfig::default();
    cfg.scan_timeout_secs = 45;
    cfg.max_interfaces = 512;
    cfg.fallback_dns_servers = vec!["9.9.9.9".to_string()];

    let json_str = serde_json::to_string_pretty(&cfg).expect("serialize");
    let deserialized: NetworkConfig = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(cfg, deserialized);
}

#[test]
fn test_nconf6_load_from_path_missing_file() {
    let non_existent = PathBuf::from("does_not_exist_network_config_12345.json");
    let cfg = NetworkConfig::load_from_path(&non_existent).expect("load missing");
    assert_eq!(cfg, NetworkConfig::default());
}

#[test]
fn test_nconf6_save_and_load_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("subdir").join("net_cfg.json");

    let mut cfg = NetworkConfig::default();
    cfg.scan_timeout_secs = 60;
    cfg.max_interfaces = 256;
    cfg.max_routes = 1024;
    cfg.fallback_dns_servers = vec!["8.8.4.4".to_string(), "1.0.0.1".to_string()];

    cfg.save_to_path(&file_path).expect("save_to_path");
    assert!(file_path.exists());

    let loaded = NetworkConfig::load_from_path(&file_path).expect("load_from_path");
    assert_eq!(cfg, loaded);

    let loaded_alias = NetworkConfig::from_file(&file_path).expect("from_file");
    assert_eq!(cfg, loaded_alias);
}

#[test]
fn test_nconf6_oversized_file_rejected() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("oversized.json");
    // Create file > 1MB
    let big_data = vec![b' '; (MAX_CONFIG_FILE_BYTES + 10) as usize];
    fs::write(&file_path, big_data).expect("write oversized");

    let err = NetworkConfig::load_from_path(&file_path).unwrap_err();
    assert!(err.contains("exceeds maximum allowed"));
}

static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_nconf5_from_env_valid() {
    let _lock = ENV_MUTEX.lock().unwrap();
    std::env::set_var("AIOS_NETWORK_STORE_PATH", "/custom/network_store.json");
    std::env::set_var("AIOS_NETWORK_SYSFS_PATH", "/custom/sys/class/net");
    std::env::set_var("AIOS_NETWORK_PROCFS_PATH", "/custom/proc/net");
    std::env::set_var("AIOS_NETWORK_RESOLV_PATH", "/custom/resolv.conf");
    std::env::set_var("AIOS_NETWORK_MAX_INTERFACES", "500");
    std::env::set_var("AIOS_NETWORK_MAX_ROUTES", "2000");
    std::env::set_var("AIOS_NETWORK_MAX_DNS", "16");
    std::env::set_var("AIOS_NETWORK_TIMEOUT", "45");

    let cfg = NetworkConfig::from_env();
    assert_eq!(cfg.default_store_path, PathBuf::from("/custom/network_store.json"));
    assert_eq!(cfg.sysfs_net_path, PathBuf::from("/custom/sys/class/net"));
    assert_eq!(cfg.procfs_path, PathBuf::from("/custom/proc/net"));
    assert_eq!(cfg.resolv_conf_path, PathBuf::from("/custom/resolv.conf"));
    assert_eq!(cfg.max_interfaces, 500);
    assert_eq!(cfg.max_routes, 2000);
    assert_eq!(cfg.max_dns_servers, 16);
    assert_eq!(cfg.scan_timeout_secs, 45);

    // Clean up
    std::env::remove_var("AIOS_NETWORK_STORE_PATH");
    std::env::remove_var("AIOS_NETWORK_SYSFS_PATH");
    std::env::remove_var("AIOS_NETWORK_PROCFS_PATH");
    std::env::remove_var("AIOS_NETWORK_RESOLV_PATH");
    std::env::remove_var("AIOS_NETWORK_MAX_INTERFACES");
    std::env::remove_var("AIOS_NETWORK_MAX_ROUTES");
    std::env::remove_var("AIOS_NETWORK_MAX_DNS");
    std::env::remove_var("AIOS_NETWORK_TIMEOUT");
}

#[test]
fn test_nconf5_from_env_invalid_fallback() {
    let _lock = ENV_MUTEX.lock().unwrap();
    // Setting an invalid value (e.g. traversal path) must cause from_env to safely fall back to default
    std::env::set_var("AIOS_NETWORK_SYSFS_PATH", "/sys/../etc");
    let cfg = NetworkConfig::from_env();
    assert_eq!(cfg, NetworkConfig::default());
    std::env::remove_var("AIOS_NETWORK_SYSFS_PATH");
}
