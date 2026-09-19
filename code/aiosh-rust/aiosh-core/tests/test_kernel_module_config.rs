//! Automated unit test suite for Kernel Module Management Configuration (T-01645).
//!
//! Enforces invariants CFG-KM1..CFG-KM5:
//! - CFG-KM1: Store and target path validity and presence
//! - CFG-KM2: Path length (≤ 1024 bytes) and control character sanitization
//! - CFG-KM3: Mutual exclusion between autoloading and blacklisting
//! - CFG-KM4: Document size limits (≤ 10 MiB)
//! - CFG-KM5: Two-way modprobe.d and modules-load.d import and export

use aiosh_core::kernel_module::ModprobeRule;
use aiosh_core::kernel_module_config::{
    import_modprobe_file_to_store, import_modules_load_file_to_store, parse_modprobe_conf_line,
    parse_modules_load_conf_line, KernelModuleManagementConfig,
};
use aiosh_core::kernel_module_service::{KernelModuleStore, MAX_MODULE_DOC_BYTES};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_km_config_defaults_and_validation() {
    let cfg = KernelModuleManagementConfig::default();
    assert_eq!(cfg.validate(), Ok(()));
    assert_eq!(cfg.default_store_path, PathBuf::from(".aios/kernel_modules.json"));
    assert_eq!(cfg.modprobe_d_path, PathBuf::from("/etc/modprobe.d/aios.conf"));
    assert_eq!(cfg.modules_load_d_path, PathBuf::from("/etc/modules-load.d/aios.conf"));
    assert_eq!(cfg.max_doc_bytes, MAX_MODULE_DOC_BYTES);
    assert!(cfg.strict_conflict_prevention);
    assert!(!cfg.allow_custom_commands);
}

#[test]
fn test_km_config_path_invariants() {
    let mut cfg = KernelModuleManagementConfig::default();

    // 1. Empty default_store_path
    cfg.default_store_path = PathBuf::from("");
    assert!(cfg.validate().unwrap_err().contains("CFG-KM1 violation"));

    // 2. Control character in path
    cfg = KernelModuleManagementConfig::default();
    cfg.default_store_path = PathBuf::from("store\x07.json");
    assert!(cfg.validate().unwrap_err().contains("CFG-KM2 violation"));

    // 3. Exceeds 1024 bytes
    cfg = KernelModuleManagementConfig::default();
    cfg.default_store_path = PathBuf::from("a".repeat(1025));
    assert!(cfg.validate().unwrap_err().contains("CFG-KM2 violation"));

    // 4. Exactly 1024 bytes passes
    cfg = KernelModuleManagementConfig::default();
    cfg.default_store_path = PathBuf::from("a".repeat(1024));
    assert_eq!(cfg.validate(), Ok(()));

    // 5. Exceeds max_doc_bytes
    cfg = KernelModuleManagementConfig::default();
    cfg.max_doc_bytes = MAX_MODULE_DOC_BYTES + 1;
    assert!(cfg.validate().unwrap_err().contains("CFG-KM4 violation"));
}

#[test]
fn test_km_config_parse_modprobe_directives() {
    // Blacklist
    let res = parse_modprobe_conf_line("blacklist nouveau").expect("parse blacklist");
    assert_eq!(res, Some(ModprobeRule::Blacklist { module: "nouveau".into() }));

    // Options with multiple parameters
    let res = parse_modprobe_conf_line("options i915 enable_guc=3 enable_fbc=1").expect("parse options");
    assert_eq!(
        res,
        Some(ModprobeRule::Options {
            module: "i915".into(),
            options: vec!["enable_guc=3".into(), "enable_fbc=1".into()],
        })
    );

    // Install with command
    let res = parse_modprobe_conf_line("install cramfs /bin/true").expect("parse install");
    assert_eq!(
        res,
        Some(ModprobeRule::Install {
            module: "cramfs".into(),
            command: "/bin/true".into(),
        })
    );

    // Alias
    let res = parse_modprobe_conf_line("alias net_pf_33 dccp").expect("parse alias");
    assert_eq!(
        res,
        Some(ModprobeRule::Alias {
            alias: "net_pf_33".into(),
            module: "dccp".into(),
        })
    );

    // Softdep
    let res = parse_modprobe_conf_line("softdep ext4 pre: crc32c post: mbcache").expect("parse softdep");
    assert_eq!(
        res,
        Some(ModprobeRule::Softdep {
            module: "ext4".into(),
            pre: vec!["crc32c".into()],
            post: vec!["mbcache".into()],
        })
    );

    // Remove
    let res = parse_modprobe_conf_line("remove dccp /bin/true").expect("parse remove");
    assert_eq!(
        res,
        Some(ModprobeRule::Remove {
            module: "dccp".into(),
            command: "/bin/true".into(),
        })
    );

    // Comments and empty lines
    assert_eq!(parse_modprobe_conf_line("   # A comment").expect("parse comment"), None);
    assert_eq!(parse_modprobe_conf_line("   ").expect("parse empty"), None);
}

#[test]
fn test_km_config_parse_modprobe_negative() {
    // Missing arguments
    assert!(parse_modprobe_conf_line("blacklist").is_err());
    assert!(parse_modprobe_conf_line("options i915").is_err());
    assert!(parse_modprobe_conf_line("install cramfs").is_err());
    assert!(parse_modprobe_conf_line("alias myalias").is_err());

    // Invalid module name
    assert!(parse_modprobe_conf_line("blacklist bad-mod").is_err());
    assert!(parse_modprobe_conf_line("blacklist bad;mod").is_err());

    // Unrecognized directive
    assert!(parse_modprobe_conf_line("bogus directive").is_err());
}

#[test]
fn test_km_config_parse_modules_load() {
    assert_eq!(parse_modules_load_conf_line("overlay").expect("parse module"), Some("overlay".into()));
    assert_eq!(parse_modules_load_conf_line("  kvm_intel  ").expect("parse trimmed"), Some("kvm_intel".into()));
    assert_eq!(parse_modules_load_conf_line("# comment").expect("parse comment"), None);
    assert_eq!(parse_modules_load_conf_line("; semicolon comment").expect("parse semicolon comment"), None);
    assert_eq!(parse_modules_load_conf_line("").expect("parse empty"), None);

    // Invalid module name in modules-load
    assert!(parse_modules_load_conf_line("bad-mod").is_err());
}

#[test]
fn test_km_config_file_ingestion_and_conflict_detection() {
    let dir = tempdir().expect("tempdir");
    let modprobe_path = dir.path().join("aios_test.conf");
    let modules_load_path = dir.path().join("aios_load.conf");

    std::fs::write(
        &modprobe_path,
        "blacklist cramfs\nblacklist freevxfs\noptions i915 enable_guc=3\n",
    )
    .expect("write modprobe");

    std::fs::write(&modules_load_path, "overlay\nbr_netfilter\n").expect("write modules-load");

    let mut store = KernelModuleStore::new("ingest-test", "ingestion test store");

    // 1. Ingest modprobe file
    let mod_count = import_modprobe_file_to_store(&mut store, &modprobe_path).expect("import modprobe");
    assert_eq!(mod_count, 3);
    assert_eq!(store.config.rules.len(), 3);

    // 2. Ingest modules-load file
    let load_count = import_modules_load_file_to_store(&mut store, &modules_load_path).expect("import modules-load");
    assert_eq!(load_count, 2);
    assert_eq!(store.config.autoload_modules.len(), 2);

    // 3. Conflict detection: attempting to autoload blacklisted cramfs must fail (CFG-KM3)
    assert!(store.add_autoload("cramfs").is_err());

    // 4. Conflict detection: attempting to blacklist autoloaded overlay must fail (CFG-KM3)
    assert!(store.add_blacklist("overlay").is_err());
}
