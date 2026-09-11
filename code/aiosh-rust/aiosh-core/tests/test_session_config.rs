//! Dedicated automated unit test suite for User Session Bootstrap Configuration (T-01445).
//!
//! Enforces invariants SC1..SC7:
//! - SC1: Store path validity and boundary enforcement
//! - SC2: Max sessions per user bounds [1 .. 128]
//! - SC3: Total store capacity bounds [10 .. 10,000]
//! - SC4: Default idle timeout bounds [10 .. 86,400] seconds
//! - SC5: Store size ceiling bounds [64 KiB .. 100 MiB]
//! - SC6: Resolution precedence (file > env > default)
//! - SC7: Configuration file size cap (max 64 KiB) and fail-loud on invalid JSON

use aiosh_core::session_config::{
    SessionConfig, DEFAULT_AUTO_PERSIST, DEFAULT_IDLE_TIMEOUT_SECS, DEFAULT_MAX_SESSIONS_PER_USER,
    DEFAULT_MAX_STORE_SIZE_BYTES, DEFAULT_MAX_TOTAL_SESSIONS, DEFAULT_SESSION_STORE_PATH,
    MAX_ALLOWED_SESSIONS_PER_USER, MAX_ALLOWED_STORE_SIZE_BYTES, MAX_ALLOWED_TOTAL_SESSIONS,
    MAX_CONFIG_FILE_BYTES, MAX_IDLE_TIMEOUT_SECS, MIN_IDLE_TIMEOUT_SECS, MIN_SESSIONS_PER_USER,
    MIN_STORE_SIZE_BYTES, MIN_TOTAL_SESSIONS,
};
use std::path::PathBuf;

#[test]
fn test_session_config_defaults_and_validation() {
    let cfg = SessionConfig::default();
    assert_eq!(cfg.validate(), Ok(()));
    assert_eq!(cfg.store_path, PathBuf::from(DEFAULT_SESSION_STORE_PATH));
    assert_eq!(cfg.max_sessions_per_user, DEFAULT_MAX_SESSIONS_PER_USER);
    assert_eq!(cfg.max_total_sessions, DEFAULT_MAX_TOTAL_SESSIONS);
    assert_eq!(cfg.default_idle_timeout_seconds, DEFAULT_IDLE_TIMEOUT_SECS);
    assert_eq!(cfg.max_store_size_bytes, DEFAULT_MAX_STORE_SIZE_BYTES);
    assert_eq!(cfg.auto_persist, DEFAULT_AUTO_PERSIST);
}

#[test]
fn test_session_config_sc1_store_path_invariants() {
    let mut cfg = SessionConfig::default();

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
    cfg.store_path = PathBuf::from(".aios/sessions\n.json");
    assert!(cfg.validate().unwrap_err().contains("SC1 violation"));

    // Null byte
    cfg.store_path = PathBuf::from(".aios/sessions\0.json");
    assert!(cfg.validate().unwrap_err().contains("SC1 violation"));
}

#[test]
fn test_session_config_sc2_user_capacity_invariants() {
    let mut cfg = SessionConfig::default();

    // Below minimum (<1)
    cfg.max_sessions_per_user = 0;
    assert!(cfg.validate().unwrap_err().contains("SC2 violation"));

    // Exact minimum boundary (1)
    cfg.max_sessions_per_user = MIN_SESSIONS_PER_USER;
    assert_eq!(cfg.validate(), Ok(()));

    // Exact maximum boundary (128)
    cfg.max_sessions_per_user = MAX_ALLOWED_SESSIONS_PER_USER;
    assert_eq!(cfg.validate(), Ok(()));

    // Exceeds maximum (>128)
    cfg.max_sessions_per_user = MAX_ALLOWED_SESSIONS_PER_USER + 1;
    assert!(cfg.validate().unwrap_err().contains("SC2 violation"));
}

#[test]
fn test_session_config_sc3_total_capacity_invariants() {
    let mut cfg = SessionConfig::default();

    // Below minimum (<10)
    cfg.max_total_sessions = MIN_TOTAL_SESSIONS - 1;
    assert!(cfg.validate().unwrap_err().contains("SC3 violation"));

    // Exact minimum boundary (10)
    cfg.max_total_sessions = MIN_TOTAL_SESSIONS;
    assert_eq!(cfg.validate(), Ok(()));

    // Exact maximum boundary (10,000)
    cfg.max_total_sessions = MAX_ALLOWED_TOTAL_SESSIONS;
    assert_eq!(cfg.validate(), Ok(()));

    // Exceeds maximum (>10,000)
    cfg.max_total_sessions = MAX_ALLOWED_TOTAL_SESSIONS + 1;
    assert!(cfg.validate().unwrap_err().contains("SC3 violation"));
}

#[test]
fn test_session_config_sc4_idle_timeout_invariants() {
    let mut cfg = SessionConfig::default();

    // Below minimum (<10s)
    cfg.default_idle_timeout_seconds = MIN_IDLE_TIMEOUT_SECS - 1;
    assert!(cfg.validate().unwrap_err().contains("SC4 violation"));

    // Exact minimum boundary (10s)
    cfg.default_idle_timeout_seconds = MIN_IDLE_TIMEOUT_SECS;
    assert_eq!(cfg.validate(), Ok(()));

    // Exact maximum boundary (86,400s)
    cfg.default_idle_timeout_seconds = MAX_IDLE_TIMEOUT_SECS;
    assert_eq!(cfg.validate(), Ok(()));

    // Exceeds maximum (>86,400s)
    cfg.default_idle_timeout_seconds = MAX_IDLE_TIMEOUT_SECS + 1;
    assert!(cfg.validate().unwrap_err().contains("SC4 violation"));
}

#[test]
fn test_session_config_sc5_store_size_invariants() {
    let mut cfg = SessionConfig::default();

    // Below minimum (<64 KiB)
    cfg.max_store_size_bytes = MIN_STORE_SIZE_BYTES - 1;
    assert!(cfg.validate().unwrap_err().contains("SC5 violation"));

    // Exact minimum boundary (64 KiB)
    cfg.max_store_size_bytes = MIN_STORE_SIZE_BYTES;
    assert_eq!(cfg.validate(), Ok(()));

    // Exact maximum boundary (100 MiB)
    cfg.max_store_size_bytes = MAX_ALLOWED_STORE_SIZE_BYTES;
    assert_eq!(cfg.validate(), Ok(()));

    // Exceeds maximum (>100 MiB)
    cfg.max_store_size_bytes = MAX_ALLOWED_STORE_SIZE_BYTES + 1;
    assert!(cfg.validate().unwrap_err().contains("SC5 violation"));
}

#[test]
fn test_session_config_sc6_env_resolution_and_precedence() {
    std::env::set_var("AIOS_SESSION_STORE_PATH", "/tmp/env_session_store.json");
    std::env::set_var("AIOS_SESSION_MAX_PER_USER", "64");
    std::env::set_var("AIOS_SESSION_MAX_TOTAL", "5000");
    std::env::set_var("AIOS_SESSION_IDLE_TIMEOUT_SECS", "1800");
    std::env::set_var("AIOS_SESSION_MAX_STORE_SIZE_BYTES", "20971520");
    std::env::set_var("AIOS_SESSION_AUTO_PERSIST", "false");

    let cfg = SessionConfig::from_env().expect("from_env should succeed with valid vars");
    assert_eq!(cfg.store_path, PathBuf::from("/tmp/env_session_store.json"));
    assert_eq!(cfg.max_sessions_per_user, 64);
    assert_eq!(cfg.max_total_sessions, 5000);
    assert_eq!(cfg.default_idle_timeout_seconds, 1800);
    assert_eq!(cfg.max_store_size_bytes, 20 * 1024 * 1024);
    assert!(!cfg.auto_persist);

    // Precedence: explicit file overrides environment variables
    let temp_dir = std::env::temp_dir();
    let config_file = temp_dir.join(format!("aios_session_precedence_{}.json", std::process::id()));
    let mut file_cfg = SessionConfig::default();
    file_cfg.store_path = PathBuf::from("/tmp/file_session_store.json");
    file_cfg.max_sessions_per_user = 16;
    std::fs::write(&config_file, serde_json::to_string(&file_cfg).unwrap()).unwrap();

    let resolved = SessionConfig::resolve(Some(&config_file)).expect("resolve should use explicit file");
    assert_eq!(resolved.store_path, PathBuf::from("/tmp/file_session_store.json"));
    assert_eq!(resolved.max_sessions_per_user, 16);

    let _ = std::fs::remove_file(&config_file);

    // Clean up
    std::env::remove_var("AIOS_SESSION_STORE_PATH");
    std::env::remove_var("AIOS_SESSION_MAX_PER_USER");
    std::env::remove_var("AIOS_SESSION_MAX_TOTAL");
    std::env::remove_var("AIOS_SESSION_IDLE_TIMEOUT_SECS");
    std::env::remove_var("AIOS_SESSION_MAX_STORE_SIZE_BYTES");
    std::env::remove_var("AIOS_SESSION_AUTO_PERSIST");
}

#[test]
fn test_session_config_sc7_file_size_cap_and_malformed() {
    let temp_dir = std::env::temp_dir();

    // 1. Oversized file (> 64 KiB)
    let oversized_file = temp_dir.join(format!("aios_session_oversized_{}.json", std::process::id()));
    let large_data = vec![b' '; (MAX_CONFIG_FILE_BYTES + 10) as usize];
    std::fs::write(&oversized_file, large_data).unwrap();

    let err = SessionConfig::from_file(&oversized_file).unwrap_err();
    assert!(err.contains("SC7 violation"));
    let _ = std::fs::remove_file(&oversized_file);

    // 2. Malformed JSON
    let bad_json_file = temp_dir.join(format!("aios_session_bad_{}.json", std::process::id()));
    std::fs::write(&bad_json_file, "{ not valid json").unwrap();

    let err = SessionConfig::from_file(&bad_json_file).unwrap_err();
    assert!(err.contains("failed to parse session config JSON"));
    let _ = std::fs::remove_file(&bad_json_file);
}

#[test]
fn test_session_config_file_roundtrip() {
    let temp_dir = std::env::temp_dir();
    let config_file = temp_dir.join(format!("aios_session_rt_{}.json", std::process::id()));

    let mut cfg = SessionConfig::default();
    cfg.store_path = PathBuf::from(".aios/custom_sessions.json");
    cfg.max_sessions_per_user = 48;
    cfg.max_total_sessions = 2048;
    cfg.default_idle_timeout_seconds = 600;
    cfg.max_store_size_bytes = 15 * 1024 * 1024;
    cfg.auto_persist = false;

    let content = serde_json::to_string_pretty(&cfg).unwrap();
    std::fs::write(&config_file, content).unwrap();

    let loaded = SessionConfig::from_file(&config_file).unwrap();
    assert_eq!(loaded, cfg);

    let _ = std::fs::remove_file(&config_file);
}
