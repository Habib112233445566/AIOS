//! Dedicated automated unit test suite for Package Management Configuration (T-01245).
//!
//! Enforces invariants PC1..PC6:
//! - PC1: Store path validity and boundary enforcement
//! - PC2: Store size ceiling bounds [64 KiB .. 100 MiB]
//! - PC3: Entity count bounds [10 .. 100,000]
//! - PC4: Repository transport security (HTTPS / file only)
//! - PC5: Resolution precedence (file > env > default)
//! - PC6: Configuration file size cap (max 64 KiB)

use aiosh_core::package::PackageFormat;
use aiosh_core::package_config::{
    PackageConfig, DEFAULT_MAX_ENTITY_COUNT, DEFAULT_MAX_STORE_SIZE_BYTES,
    DEFAULT_PACKAGE_STORE_PATH, MAX_ALLOWED_ENTITY_COUNT, MAX_ALLOWED_STORE_SIZE_BYTES,
    MIN_ENTITY_COUNT, MIN_STORE_SIZE_BYTES,
};
use std::path::PathBuf;

#[test]
fn test_package_config_defaults_and_validation() {
    let cfg = PackageConfig::default();
    assert_eq!(cfg.validate(), Ok(()));
    assert_eq!(cfg.store_path, PathBuf::from(DEFAULT_PACKAGE_STORE_PATH));
    assert_eq!(cfg.default_format, PackageFormat::Deb);
    assert_eq!(cfg.max_store_size_bytes, DEFAULT_MAX_STORE_SIZE_BYTES);
    assert_eq!(cfg.max_entity_count, DEFAULT_MAX_ENTITY_COUNT);
    assert!(!cfg.auto_persist);
    assert_eq!(cfg.allowed_repositories.len(), 2);
}

#[test]
fn test_package_config_pc1_store_path_invariants() {
    let mut cfg = PackageConfig::default();

    // Empty path
    cfg.store_path = PathBuf::from("");
    assert!(cfg.validate().unwrap_err().contains("PC1 violation"));

    // Exceeds 1024 bytes
    cfg.store_path = PathBuf::from("a".repeat(1025));
    assert!(cfg.validate().unwrap_err().contains("PC1 violation"));

    // Exactly 1024 bytes (boundary pass)
    cfg.store_path = PathBuf::from("a".repeat(1024));
    assert_eq!(cfg.validate(), Ok(()));

    // Control character
    cfg.store_path = PathBuf::from(".aios/packages\n.json");
    assert!(cfg.validate().unwrap_err().contains("PC1 violation"));

    // Null byte
    cfg.store_path = PathBuf::from(".aios/packages\0.json");
    assert!(cfg.validate().unwrap_err().contains("PC1 violation"));
}

#[test]
fn test_package_config_pc2_store_size_invariants() {
    let mut cfg = PackageConfig::default();

    // Below minimum (<64 KiB)
    cfg.max_store_size_bytes = MIN_STORE_SIZE_BYTES - 1;
    assert!(cfg.validate().unwrap_err().contains("PC2 violation"));

    // Exact minimum boundary (64 KiB)
    cfg.max_store_size_bytes = MIN_STORE_SIZE_BYTES;
    assert_eq!(cfg.validate(), Ok(()));

    // Exact maximum boundary (100 MiB)
    cfg.max_store_size_bytes = MAX_ALLOWED_STORE_SIZE_BYTES;
    assert_eq!(cfg.validate(), Ok(()));

    // Exceeds maximum (>100 MiB)
    cfg.max_store_size_bytes = MAX_ALLOWED_STORE_SIZE_BYTES + 1;
    assert!(cfg.validate().unwrap_err().contains("PC2 violation"));
}

#[test]
fn test_package_config_pc3_entity_count_invariants() {
    let mut cfg = PackageConfig::default();

    // Below minimum (<10)
    cfg.max_entity_count = MIN_ENTITY_COUNT - 1;
    assert!(cfg.validate().unwrap_err().contains("PC3 violation"));

    // Exact minimum boundary (10)
    cfg.max_entity_count = MIN_ENTITY_COUNT;
    assert_eq!(cfg.validate(), Ok(()));

    // Exact maximum boundary (100,000)
    cfg.max_entity_count = MAX_ALLOWED_ENTITY_COUNT;
    assert_eq!(cfg.validate(), Ok(()));

    // Exceeds maximum (>100,000)
    cfg.max_entity_count = MAX_ALLOWED_ENTITY_COUNT + 1;
    assert!(cfg.validate().unwrap_err().contains("PC3 violation"));
}

#[test]
fn test_package_config_pc4_repository_security() {
    let mut cfg = PackageConfig::default();

    // Insecure HTTP
    cfg.allowed_repositories = vec!["http://deb.debian.org/debian".into()];
    assert!(cfg.validate().unwrap_err().contains("PC4 violation"));

    // FTP rejected
    cfg.allowed_repositories = vec!["ftp://ftp.debian.org/debian".into()];
    assert!(cfg.validate().unwrap_err().contains("PC4 violation"));

    // Secure HTTPS passes
    cfg.allowed_repositories = vec!["https://deb.debian.org/debian".into()];
    assert_eq!(cfg.validate(), Ok(()));

    // Local file:// mirror passes
    cfg.allowed_repositories = vec!["file:///var/cache/mirror".into()];
    assert_eq!(cfg.validate(), Ok(()));

    // Control character in URL
    cfg.allowed_repositories = vec!["https://deb.debian.org\n/debian".into()];
    assert!(cfg.validate().unwrap_err().contains("PC4 violation"));
}

#[test]
fn test_package_config_pc5_env_resolution() {
    // Set environment overrides
    std::env::set_var("AIOS_PACKAGE_STORE_PATH", "/tmp/test_packages.json");
    std::env::set_var("AIOS_PACKAGE_DEFAULT_FORMAT", "apk");
    std::env::set_var("AIOS_PACKAGE_MAX_STORE_SIZE_BYTES", "20971520"); // 20 MiB
    std::env::set_var("AIOS_PACKAGE_MAX_ENTITIES", "5000");
    std::env::set_var("AIOS_PACKAGE_AUTO_PERSIST", "true");
    std::env::set_var("AIOS_PACKAGE_ALLOWED_REPOS", "https://mirror.alpinelinux.org/alpine");

    let cfg = PackageConfig::from_env().unwrap();
    assert_eq!(cfg.store_path, PathBuf::from("/tmp/test_packages.json"));
    assert_eq!(cfg.default_format, PackageFormat::Apk);
    assert_eq!(cfg.max_store_size_bytes, 20_971_520);
    assert_eq!(cfg.max_entity_count, 5_000);
    assert!(cfg.auto_persist);
    assert_eq!(cfg.allowed_repositories, vec!["https://mirror.alpinelinux.org/alpine".to_string()]);

    // Clean up environment variables
    std::env::remove_var("AIOS_PACKAGE_STORE_PATH");
    std::env::remove_var("AIOS_PACKAGE_DEFAULT_FORMAT");
    std::env::remove_var("AIOS_PACKAGE_MAX_STORE_SIZE_BYTES");
    std::env::remove_var("AIOS_PACKAGE_MAX_ENTITIES");
    std::env::remove_var("AIOS_PACKAGE_AUTO_PERSIST");
    std::env::remove_var("AIOS_PACKAGE_ALLOWED_REPOS");
}

#[test]
fn test_package_config_pc6_file_roundtrip_and_size_cap() {
    let temp_dir = std::env::temp_dir();
    let config_file = temp_dir.join(format!("aios_pkg_cfg_unit_test_{}.json", std::process::id()));

    let cfg = PackageConfig {
        store_path: PathBuf::from("/var/lib/aios/pkg.json"),
        default_format: PackageFormat::Flatpak,
        max_store_size_bytes: 50 * 1024 * 1024,
        max_entity_count: 25_000,
        auto_persist: true,
        allowed_repositories: vec!["https://flathub.org/repo".into()],
    };

    let content = serde_json::to_string_pretty(&cfg).unwrap();
    std::fs::write(&config_file, content).unwrap();

    let loaded = PackageConfig::from_file(&config_file).unwrap();
    assert_eq!(loaded, cfg);

    // Test PC5 resolve with file path
    let resolved = PackageConfig::resolve(Some(&config_file)).unwrap();
    assert_eq!(resolved, cfg);

    let _ = std::fs::remove_file(&config_file);

    // Oversized config file (>64 KiB)
    let oversized_file = temp_dir.join(format!("aios_pkg_cfg_oversized_{}.json", std::process::id()));
    let large_dummy = "{\"store_path\":\"".to_string() + &"a".repeat(70_000) + "\"}";
    std::fs::write(&oversized_file, large_dummy).unwrap();

    let err = PackageConfig::from_file(&oversized_file).unwrap_err();
    assert!(err.contains("PC6 violation"));

    let _ = std::fs::remove_file(&oversized_file);
}
