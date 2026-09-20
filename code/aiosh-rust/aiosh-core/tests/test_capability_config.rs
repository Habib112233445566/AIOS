use std::fs;
use std::path::PathBuf;
use aiosh_core::capability::{CapabilityConstraints, CapabilityRight, CapabilityScope};
use aiosh_core::capability_config::{
    CapabilityConfig, DEFAULT_CAPABILITY_STORE_PATH, DEFAULT_MAX_CAPABILITY_COUNT,
    DEFAULT_MAX_STORE_BYTES, MAX_CAPABILITY_COUNT, MAX_EXPIRES_SECS, MAX_STORE_BYTES,
    MIN_STORE_BYTES,
};
use aiosh_core::capability_service::CapabilityService;

#[test]
fn test_capability_config_defaults() {
    let cfg = CapabilityConfig::default();
    assert_eq!(cfg.version, "1.0.0");
    assert_eq!(cfg.store_path, PathBuf::from(DEFAULT_CAPABILITY_STORE_PATH));
    assert_eq!(cfg.max_store_bytes, DEFAULT_MAX_STORE_BYTES);
    assert_eq!(cfg.max_capabilities, DEFAULT_MAX_CAPABILITY_COUNT);
    assert_eq!(cfg.default_expires_secs, None);
    assert!(cfg.enforce_strict_monotonic);
    assert!(cfg.auto_prune_on_load);
    assert!(cfg.validate().is_ok());

    // Getters
    assert_eq!(cfg.store_path(), &PathBuf::from(DEFAULT_CAPABILITY_STORE_PATH));
    assert_eq!(cfg.max_store_bytes(), DEFAULT_MAX_STORE_BYTES);
    assert_eq!(cfg.max_capabilities(), DEFAULT_MAX_CAPABILITY_COUNT);
    assert_eq!(cfg.default_expires_secs(), None);
    assert!(cfg.enforce_strict_monotonic());
    assert!(cfg.auto_prune_on_load());
}

#[test]
fn test_capability_config_json_serde() {
    let cfg = CapabilityConfig::default();
    let json = cfg.to_json().expect("Serialization failed");
    let loaded = CapabilityConfig::from_json(&json).expect("Deserialization failed");
    assert_eq!(cfg, loaded);

    let custom_json = r#"{
        "version": "1.1.0",
        "store_path": "custom/store.json",
        "max_store_bytes": 2048000,
        "max_capabilities": 500,
        "default_expires_secs": 3600,
        "enforce_strict_monotonic": true,
        "auto_prune_on_load": false
    }"#;
    let custom_cfg = CapabilityConfig::from_json(custom_json).expect("Parse custom json");
    assert_eq!(custom_cfg.version, "1.1.0");
    assert_eq!(custom_cfg.store_path, PathBuf::from("custom/store.json"));
    assert_eq!(custom_cfg.max_store_bytes, 2048000);
    assert_eq!(custom_cfg.max_capabilities, 500);
    assert_eq!(custom_cfg.default_expires_secs, Some(3600));
    assert!(!custom_cfg.auto_prune_on_load);
}

#[test]
fn test_capability_config_validation_rules() {
    let mut cfg = CapabilityConfig::default();

    // Invalid version
    cfg.version = "".into();
    assert!(cfg.validate().is_err());
    cfg.version = "a".repeat(33);
    assert!(cfg.validate().is_err());
    cfg.version = "1.0.0".into();

    // Invalid store_path
    cfg.store_path = PathBuf::from("");
    assert!(cfg.validate().is_err());
    cfg.store_path = PathBuf::from("../evil.json");
    assert!(cfg.validate().is_err());
    cfg.store_path = PathBuf::from("foo/\0/bar.json");
    assert!(cfg.validate().is_err());
    cfg.store_path = PathBuf::from("a".repeat(1025));
    assert!(cfg.validate().is_err());
    cfg.store_path = PathBuf::from(".aios/capability_store.json");

    // Invalid max_store_bytes
    cfg.max_store_bytes = MIN_STORE_BYTES - 1;
    assert!(cfg.validate().is_err());
    cfg.max_store_bytes = MAX_STORE_BYTES + 1;
    assert!(cfg.validate().is_err());
    cfg.max_store_bytes = DEFAULT_MAX_STORE_BYTES;

    // Invalid max_capabilities
    cfg.max_capabilities = 0;
    assert!(cfg.validate().is_err());
    cfg.max_capabilities = MAX_CAPABILITY_COUNT + 1;
    assert!(cfg.validate().is_err());
    cfg.max_capabilities = DEFAULT_MAX_CAPABILITY_COUNT;

    // Invalid default_expires_secs
    cfg.default_expires_secs = Some(0);
    assert!(cfg.validate().is_err());
    cfg.default_expires_secs = Some(MAX_EXPIRES_SECS + 1);
    assert!(cfg.validate().is_err());
    cfg.default_expires_secs = Some(86400);
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_capability_config_file_and_env() {
    let tmp_dir = std::env::temp_dir().join(format!("aios_cap_cfg_test_{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).unwrap();
    let file_path = tmp_dir.join("test_config.json");

    let cfg = CapabilityConfig {
        version: "2.0.0".into(),
        store_path: PathBuf::from("temp/store.json"),
        max_store_bytes: 5_000_000,
        max_capabilities: 250,
        default_expires_secs: Some(1800),
        enforce_strict_monotonic: false,
        auto_prune_on_load: true,
    };
    let json = cfg.to_json().unwrap();
    fs::write(&file_path, json).unwrap();

    let loaded = CapabilityConfig::from_path(&file_path).unwrap();
    assert_eq!(loaded, cfg);

    // Test env override
    std::env::set_var("AIOS_CAPABILITY_STORE_PATH", "env_store.json");
    std::env::set_var("AIOS_CAPABILITY_MAX_CAPABILITIES", "4200");
    std::env::set_var("AIOS_CAPABILITY_MAX_STORE_BYTES", "8000000");

    let env_cfg = CapabilityConfig::from_env().unwrap();
    assert_eq!(env_cfg.store_path, PathBuf::from("env_store.json"));
    assert_eq!(env_cfg.max_capabilities, 4200);
    assert_eq!(env_cfg.max_store_bytes, 8000000);

    // Clean up env vars
    std::env::remove_var("AIOS_CAPABILITY_STORE_PATH");
    std::env::remove_var("AIOS_CAPABILITY_MAX_CAPABILITIES");
    std::env::remove_var("AIOS_CAPABILITY_MAX_STORE_BYTES");
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_capability_service_from_config_and_capacity_enforcement() {
    let mut cfg = CapabilityConfig::default();
    cfg.max_capabilities = 2;

    let mut service = CapabilityService::from_config(cfg).unwrap();
    assert_eq!(service.config().max_capabilities(), 2);

    let c1 = service
        .issue_root_capability(
            "kernel",
            "agent:worker1",
            CapabilityScope::Filesystem {
                path: "/data/1".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read],
            CapabilityConstraints::default(),
        )
        .expect("issue c1");
    assert_eq!(service.len(), 1);

    let _c2 = service
        .issue_root_capability(
            "kernel",
            "agent:worker2",
            CapabilityScope::Filesystem {
                path: "/data/2".into(),
                recursive: true,
            },
            vec![CapabilityRight::Read],
            CapabilityConstraints::default(),
        )
        .expect("issue c2");
    assert_eq!(service.len(), 2);

    // Exceed capacity on issue_root_capability
    let err = service.issue_root_capability(
        "kernel",
        "agent:worker3",
        CapabilityScope::Filesystem {
            path: "/data/3".into(),
            recursive: true,
        },
        vec![CapabilityRight::Read],
        CapabilityConstraints::default(),
    );
    assert!(err.is_err());
    let err_str = err.unwrap_err().to_string();
    assert!(err_str.contains("registry capacity limit reached (2)"));

    // Exceed capacity on attenuate_capability
    let att_err = service.attenuate_capability(
        &c1.id,
        "agent:child",
        None,
        vec![CapabilityRight::Read],
        None,
    );
    assert!(att_err.is_err());
    let att_err_str = att_err.unwrap_err().to_string();
    assert!(att_err_str.contains("registry capacity limit reached (2)"));
}
