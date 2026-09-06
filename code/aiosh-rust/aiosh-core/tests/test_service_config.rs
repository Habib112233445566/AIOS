//! Dedicated automated unit test suite for Init & Service Supervision Configuration (T-01345).
//!
//! Enforces invariants SC1..SC7:
//! - SC1: Store path validity and boundary enforcement
//! - SC2: Timeout bounds [1 .. 3,600] seconds for start and stop timeouts
//! - SC3: Store size ceiling bounds [64 KiB .. 100 MiB]
//! - SC4: Entity count bounds [10 .. 100,000]
//! - SC5: Restart throttling bounds [backoff 1..300s, burst 1..50]
//! - SC6: Resolution precedence (file > env > default)
//! - SC7: Configuration file size cap (max 64 KiB)

use aiosh_core::service_config::{
    ServiceConfig, DEFAULT_MAX_ENTITY_COUNT, DEFAULT_MAX_RESTART_BURST,
    DEFAULT_MAX_STORE_SIZE_BYTES, DEFAULT_RESTART_BACKOFF_SECS, DEFAULT_SERVICE_STORE_PATH,
    DEFAULT_TIMEOUT_START_SECS, DEFAULT_TIMEOUT_STOP_SECS, MAX_ALLOWED_ENTITY_COUNT,
    MAX_ALLOWED_STORE_SIZE_BYTES, MAX_RESTART_BACKOFF_SECS, MAX_RESTART_BURST, MAX_TIMEOUT_SECS,
    MIN_ENTITY_COUNT, MIN_RESTART_BACKOFF_SECS, MIN_RESTART_BURST, MIN_STORE_SIZE_BYTES,
    MIN_TIMEOUT_SECS,
};
use std::path::PathBuf;

#[test]
fn test_service_config_defaults_and_validation() {
    let cfg = ServiceConfig::default();
    assert_eq!(cfg.validate(), Ok(()));
    assert_eq!(cfg.store_path, PathBuf::from(DEFAULT_SERVICE_STORE_PATH));
    assert_eq!(cfg.default_timeout_start_secs, DEFAULT_TIMEOUT_START_SECS);
    assert_eq!(cfg.default_timeout_stop_secs, DEFAULT_TIMEOUT_STOP_SECS);
    assert_eq!(cfg.max_store_size_bytes, DEFAULT_MAX_STORE_SIZE_BYTES);
    assert_eq!(cfg.max_entity_count, DEFAULT_MAX_ENTITY_COUNT);
    assert!(cfg.auto_persist);
    assert_eq!(cfg.restart_backoff_secs, DEFAULT_RESTART_BACKOFF_SECS);
    assert_eq!(cfg.max_restart_burst, DEFAULT_MAX_RESTART_BURST);
}

#[test]
fn test_service_config_sc1_store_path_invariants() {
    let mut cfg = ServiceConfig::default();

    // Empty path
    cfg.store_path = PathBuf::from("");
    assert!(cfg.validate().unwrap_err().contains("SC1 violation"));

    // Exceeds 1024 bytes
    cfg.store_path = PathBuf::from("a".repeat(1025));
    assert!(cfg.validate().unwrap_err().contains("SC1 violation"));

    // Exactly 1024 bytes (boundary pass)
    cfg.store_path = PathBuf::from("a".repeat(1024));
    assert_eq!(cfg.validate(), Ok(()));

    // Control character
    cfg.store_path = PathBuf::from(".aios/services\n.json");
    assert!(cfg.validate().unwrap_err().contains("SC1 violation"));

    // Null byte
    cfg.store_path = PathBuf::from(".aios/services\0.json");
    assert!(cfg.validate().unwrap_err().contains("SC1 violation"));
}

#[test]
fn test_service_config_sc2_timeout_invariants() {
    let mut cfg = ServiceConfig::default();

    // Startup timeout below minimum (<1s)
    cfg.default_timeout_start_secs = MIN_TIMEOUT_SECS - 1;
    assert!(cfg.validate().unwrap_err().contains("SC2 violation"));

    // Exact minimum boundary (1s)
    cfg.default_timeout_start_secs = MIN_TIMEOUT_SECS;
    assert_eq!(cfg.validate(), Ok(()));

    // Exact maximum boundary (3600s)
    cfg.default_timeout_start_secs = MAX_TIMEOUT_SECS;
    assert_eq!(cfg.validate(), Ok(()));

    // Exceeds maximum (>3600s)
    cfg.default_timeout_start_secs = MAX_TIMEOUT_SECS + 1;
    assert!(cfg.validate().unwrap_err().contains("SC2 violation"));

    // Reset and test stop timeout
    cfg = ServiceConfig::default();
    cfg.default_timeout_stop_secs = MIN_TIMEOUT_SECS - 1;
    assert!(cfg.validate().unwrap_err().contains("SC2 violation"));

    cfg.default_timeout_stop_secs = MIN_TIMEOUT_SECS;
    assert_eq!(cfg.validate(), Ok(()));

    cfg.default_timeout_stop_secs = MAX_TIMEOUT_SECS;
    assert_eq!(cfg.validate(), Ok(()));

    cfg.default_timeout_stop_secs = MAX_TIMEOUT_SECS + 1;
    assert!(cfg.validate().unwrap_err().contains("SC2 violation"));
}

#[test]
fn test_service_config_sc3_sc4_sc5_boundary_invariants() {
    let mut cfg = ServiceConfig::default();

    // SC3: Store size below minimum (<64 KiB)
    cfg.max_store_size_bytes = MIN_STORE_SIZE_BYTES - 1;
    assert!(cfg.validate().unwrap_err().contains("SC3 violation"));

    // SC3: Exact minimum boundary (64 KiB)
    cfg.max_store_size_bytes = MIN_STORE_SIZE_BYTES;
    assert_eq!(cfg.validate(), Ok(()));

    // SC3: Exact maximum boundary (100 MiB)
    cfg.max_store_size_bytes = MAX_ALLOWED_STORE_SIZE_BYTES;
    assert_eq!(cfg.validate(), Ok(()));

    // SC3: Exceeds maximum (>100 MiB)
    cfg.max_store_size_bytes = MAX_ALLOWED_STORE_SIZE_BYTES + 1;
    assert!(cfg.validate().unwrap_err().contains("SC3 violation"));

    // SC4: Entity count below minimum (<10)
    cfg = ServiceConfig::default();
    cfg.max_entity_count = MIN_ENTITY_COUNT - 1;
    assert!(cfg.validate().unwrap_err().contains("SC4 violation"));

    // SC4: Exact minimum boundary (10)
    cfg.max_entity_count = MIN_ENTITY_COUNT;
    assert_eq!(cfg.validate(), Ok(()));

    // SC4: Exact maximum boundary (100,000)
    cfg.max_entity_count = MAX_ALLOWED_ENTITY_COUNT;
    assert_eq!(cfg.validate(), Ok(()));

    // SC4: Exceeds maximum (>100,000)
    cfg.max_entity_count = MAX_ALLOWED_ENTITY_COUNT + 1;
    assert!(cfg.validate().unwrap_err().contains("SC4 violation"));

    // SC5: Restart backoff delay bounds [1..300]
    cfg = ServiceConfig::default();
    cfg.restart_backoff_secs = MIN_RESTART_BACKOFF_SECS - 1;
    assert!(cfg.validate().unwrap_err().contains("SC5 violation"));

    cfg.restart_backoff_secs = MIN_RESTART_BACKOFF_SECS;
    assert_eq!(cfg.validate(), Ok(()));

    cfg.restart_backoff_secs = MAX_RESTART_BACKOFF_SECS;
    assert_eq!(cfg.validate(), Ok(()));

    cfg.restart_backoff_secs = MAX_RESTART_BACKOFF_SECS + 1;
    assert!(cfg.validate().unwrap_err().contains("SC5 violation"));

    // SC5: Restart max burst attempts [1..50]
    cfg = ServiceConfig::default();
    cfg.max_restart_burst = MIN_RESTART_BURST - 1;
    assert!(cfg.validate().unwrap_err().contains("SC5 violation"));

    cfg.max_restart_burst = MIN_RESTART_BURST;
    assert_eq!(cfg.validate(), Ok(()));

    cfg.max_restart_burst = MAX_RESTART_BURST;
    assert_eq!(cfg.validate(), Ok(()));

    cfg.max_restart_burst = MAX_RESTART_BURST + 1;
    assert!(cfg.validate().unwrap_err().contains("SC5 violation"));
}

#[test]
fn test_service_config_sc6_env_resolution() {
    std::env::set_var("AIOS_SERVICE_STORE_PATH", "/tmp/test_service_store.json");
    std::env::set_var("AIOS_SERVICE_TIMEOUT_START_SECS", "45");
    std::env::set_var("AIOS_SERVICE_TIMEOUT_STOP_SECS", "60");
    std::env::set_var("AIOS_SERVICE_MAX_STORE_SIZE_BYTES", "20971520"); // 20 MiB
    std::env::set_var("AIOS_SERVICE_MAX_ENTITIES", "5000");
    std::env::set_var("AIOS_SERVICE_AUTO_PERSIST", "false");
    std::env::set_var("AIOS_SERVICE_RESTART_BACKOFF_SECS", "10");
    std::env::set_var("AIOS_SERVICE_MAX_RESTART_BURST", "8");

    let cfg = ServiceConfig::from_env().unwrap();
    assert_eq!(cfg.store_path, PathBuf::from("/tmp/test_service_store.json"));
    assert_eq!(cfg.default_timeout_start_secs, 45);
    assert_eq!(cfg.default_timeout_stop_secs, 60);
    assert_eq!(cfg.max_store_size_bytes, 20_971_520);
    assert_eq!(cfg.max_entity_count, 5_000);
    assert!(!cfg.auto_persist);
    assert_eq!(cfg.restart_backoff_secs, 10);
    assert_eq!(cfg.max_restart_burst, 8);

    // Clean up
    std::env::remove_var("AIOS_SERVICE_STORE_PATH");
    std::env::remove_var("AIOS_SERVICE_TIMEOUT_START_SECS");
    std::env::remove_var("AIOS_SERVICE_TIMEOUT_STOP_SECS");
    std::env::remove_var("AIOS_SERVICE_MAX_STORE_SIZE_BYTES");
    std::env::remove_var("AIOS_SERVICE_MAX_ENTITIES");
    std::env::remove_var("AIOS_SERVICE_AUTO_PERSIST");
    std::env::remove_var("AIOS_SERVICE_RESTART_BACKOFF_SECS");
    std::env::remove_var("AIOS_SERVICE_MAX_RESTART_BURST");
}

#[test]
fn test_service_config_sc7_file_roundtrip_and_size_cap() {
    let temp_dir = std::env::temp_dir();
    let config_file = temp_dir.join(format!("aios_svc_cfg_unit_test_{}.json", std::process::id()));

    let cfg = ServiceConfig {
        store_path: PathBuf::from("/var/lib/aios/services.json"),
        default_timeout_start_secs: 15,
        default_timeout_stop_secs: 20,
        max_store_size_bytes: 50 * 1024 * 1024,
        max_entity_count: 25_000,
        auto_persist: false,
        restart_backoff_secs: 15,
        max_restart_burst: 10,
    };

    let content = serde_json::to_string_pretty(&cfg).unwrap();
    std::fs::write(&config_file, content).unwrap();

    let loaded = ServiceConfig::from_file(&config_file).unwrap();
    assert_eq!(loaded, cfg);

    // Test resolve with explicit path
    let resolved = ServiceConfig::resolve(Some(&config_file)).unwrap();
    assert_eq!(resolved, cfg);

    let _ = std::fs::remove_file(&config_file);

    // Oversized config file (>64 KiB)
    let oversized_file = temp_dir.join(format!("aios_svc_cfg_oversized_{}.json", std::process::id()));
    let large_dummy = "{\"store_path\":\"".to_string() + &"a".repeat(70_000) + "\"}";
    std::fs::write(&oversized_file, large_dummy).unwrap();

    let err = ServiceConfig::from_file(&oversized_file).unwrap_err();
    assert!(err.contains("SC7 violation"));

    let _ = std::fs::remove_file(&oversized_file);
}
