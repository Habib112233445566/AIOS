//! Unit tests for Kernel Module Recovery & Validation subsystem (KR1..KR6).

use std::fs;
use std::path::Path;
use tempfile::tempdir;

use aiosh_core::kernel_module::ModprobeRule;
use aiosh_core::kernel_module_recovery::{
    check_store_file, recover_store_file, validate_kernel_module_store,
};
use aiosh_core::kernel_module_service::KernelModuleStore;

#[test]
fn test_kr1_kr2_kr3_healthy_store_validation() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("modules.json");

    let mut store = KernelModuleStore::new("test-store", "Healthy validation test store");
    store.config.rules.push(ModprobeRule::Blacklist {
        module: "dccp".into(),
    });
    store.config.rules.push(ModprobeRule::Alias {
        alias: "net-pf-33".into(),
        module: "dccp".into(),
    });
    store.config.rules.push(ModprobeRule::Options {
        module: "nf_conntrack".into(),
        options: vec!["hashsize=65536".into()],
    });
    store.add_autoload("overlay").expect("add autoload");

    let rep = validate_kernel_module_store(&store, &store_path);

    // Assert invariants KR1, KR2, KR3
    assert!(rep.validate_invariants().is_ok(), "invariants must hold: {:?}", rep.validate_invariants());
    assert!(rep.healthy, "store should be healthy");
    assert_eq!(rep.total_rules, 3);
    assert_eq!(rep.valid_rules, 3);
    assert_eq!(rep.invalid_rules, 0);
    assert_eq!(rep.total_autoload, 1);
    assert_eq!(rep.valid_autoload, 1);
    assert_eq!(rep.invalid_autoload, 0);
    assert!(rep.errors.is_empty());
}

#[test]
fn test_kr4_conflict_detection_and_resolution() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("conflict_store.json");

    let mut store = KernelModuleStore::new("conflict-store", "Store with conflict");
    // Add blacklist rule for sctp
    store.config.rules.push(ModprobeRule::Blacklist {
        module: "sctp".into(),
    });
    store.config.autoload_modules.push("sctp".into());
    // Write directly to disk bypassing save_to_path validation to simulate on-disk conflict
    fs::write(&store_path, serde_json::to_string_pretty(&store).unwrap()).expect("write conflict store");

    // Validate detection
    let initial_rep = check_store_file(&store_path).expect("check");
    assert!(!initial_rep.healthy, "store with conflict must not be healthy");
    assert_eq!(initial_rep.invalid_autoload, 1);
    assert!(initial_rep.errors.iter().any(|e| e.contains("KM3 conflict")));

    // Recover store
    let recovery_rep = recover_store_file(&store_path).expect("recover");
    assert!(recovery_rep.healthy, "recovered store must be healthy");
    assert!(recovery_rep.recovered, "must be marked recovered");
    assert!(recovery_rep.backup_path.is_some(), "quarantine backup must be created");

    // Verify backup exists
    let backup_path = recovery_rep.backup_path.unwrap();
    assert!(Path::new(&backup_path).exists(), "backup file must exist on disk");

    // Verify sctp is no longer in autoload
    let fixed_store = KernelModuleStore::load_from_path(&store_path).expect("load fixed");
    assert!(!fixed_store.config.autoload_modules.contains(&"sctp".to_string()));
    assert!(fixed_store.config.rules.iter().any(|r| match r {
        ModprobeRule::Blacklist { module } => module == "sctp",
        _ => false,
    }));
}

#[test]
fn test_kr5_unparseable_json_quarantine_and_reinitialization() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("corrupted.json");

    let broken_content = "{ \"rules\": [ invalid json syntax here ...";
    fs::write(&store_path, broken_content).expect("write corrupted");

    let check_rep = check_store_file(&store_path).expect("check");
    assert!(!check_rep.healthy);
    assert!(check_rep.errors[0].contains("store parse failure"));

    let recovery_rep = recover_store_file(&store_path).expect("recover");
    assert!(recovery_rep.healthy);
    assert!(recovery_rep.recovered);

    let backup_file = recovery_rep.backup_path.expect("backup created");
    let backup_content = fs::read_to_string(&backup_file).expect("read backup");
    assert_eq!(backup_content, broken_content);

    // Target file now contains a valid default store
    let loaded = KernelModuleStore::load_from_path(&store_path).expect("load recovered");
    assert_eq!(loaded.config.id, "default");
}

#[test]
fn test_kr6_partial_corruption_repair_and_backup() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("partial_corruption.json");

    let mut store = KernelModuleStore::new("partial", "Store with invalid rules");
    // Valid rule
    store.config.rules.push(ModprobeRule::Blacklist {
        module: "tipc".into(),
    });
    // Injected/invalid rule
    store.config.rules.push(ModprobeRule::Blacklist {
        module: "bad;rm -rf /".into(),
    });
    // Invalid autoload
    store.config.autoload_modules.push("invalid/autoload".into());
    store.config.autoload_modules.push("loop".into());
    fs::write(&store_path, serde_json::to_string_pretty(&store).unwrap()).expect("save");

    let check_rep = check_store_file(&store_path).expect("check");
    assert!(!check_rep.healthy);
    assert_eq!(check_rep.invalid_rules, 1);
    assert_eq!(check_rep.invalid_autoload, 1);

    let rec_rep = recover_store_file(&store_path).expect("recover");
    assert!(rec_rep.healthy);
    assert!(rec_rep.recovered);
    assert_eq!(rec_rep.valid_rules, 1);
    assert_eq!(rec_rep.valid_autoload, 1);

    let fixed = KernelModuleStore::load_from_path(&store_path).expect("load fixed");
    assert_eq!(fixed.config.rules.len(), 1);
    assert_eq!(fixed.config.autoload_modules, vec!["loop".to_string()]);
}

#[test]
fn test_non_existent_file_check_and_recovery() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("missing.json");

    let check_rep = check_store_file(&store_path).expect("check");
    assert!(!check_rep.healthy);
    assert!(check_rep.errors[0].contains("does not exist"));

    let rec_rep = recover_store_file(&store_path).expect("recover");
    assert!(rec_rep.healthy);
    assert!(rec_rep.recovered);
    assert!(store_path.exists());

    let fixed = KernelModuleStore::load_from_path(&store_path).expect("load fixed");
    assert_eq!(fixed.config.id, "default");
}
