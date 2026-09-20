//! Integration Tests for Kernel Module Management Core Service (KS1..KS5)

use std::fs;
use aiosh_core::kernel_module_service::{KernelModuleService, KernelModuleStore, MAX_MODULE_DOC_BYTES};

#[test]
fn test_ks1_service_inspection_with_mock_procfs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mock_proc = dir.path().join("proc_modules");
    fs::write(
        &mock_proc,
        "overlay 151552 1 - Live 0x0000000000000000\nwireguard 94208 0 - Live 0x0000000000000000\n",
    ).expect("write mock");

    let store = KernelModuleStore::new("test-srv", "test service");
    let service = KernelModuleService::new(store).with_proc_modules_path(mock_proc);

    let loaded = service.list_loaded_modules().expect("list loaded");
    assert_eq!(loaded.len(), 2);

    let wg = service.get_module("wireguard").expect("get wg");
    assert!(wg.is_some());
    assert_eq!(wg.unwrap().size_bytes, 94208);
}

#[test]
fn test_ks2_conflict_validation_at_service_layer() {
    let mut store = KernelModuleStore::new("conflict-store", "test store");
    store.add_autoload("tun").expect("add autoload");

    // Conflict: blacklisting autoloaded module
    let err = store.add_blacklist("tun").unwrap_err();
    assert!(err.contains("is autoloaded; cannot blacklist"));

    // Conflict: autoloading blacklisted module
    store.add_blacklist("cramfs").expect("add blacklist");
    let err2 = store.add_autoload("cramfs").unwrap_err();
    assert!(err2.contains("is blacklisted or disabled; cannot autoload"));
}

#[test]
fn test_ks3_atomic_store_persistence_and_recovery() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store_path = dir.path().join("modules_store.json");

    let mut store = KernelModuleStore::new("persist-id", "persistence test");
    store.add_blacklist("cramfs").expect("add blacklist");
    store.add_options("ath9k_htc", vec!["nohwcrypt=1".to_string()]).expect("add options");
    store.add_autoload("overlay").expect("add autoload");

    store.save_to_path(&store_path).expect("save store");
    assert!(store_path.exists());

    let loaded = KernelModuleStore::load_from_path(&store_path).expect("load store");
    assert_eq!(loaded, store);
    assert_eq!(loaded.config.rules.len(), 2);
    assert_eq!(loaded.config.autoload_modules, vec!["overlay"]);
}

#[test]
fn test_ks4_idempotent_modprobe_rule_generation() {
    let mut store = KernelModuleStore::new("idempotent-id", "idempotent test");
    store.add_blacklist("sctp").expect("first");
    store.add_blacklist("sctp").expect("second");

    // Only 1 rule
    assert_eq!(store.config.rules.len(), 1);

    // Export modprobe conf
    let conf = store.export_modprobe_conf();
    assert_eq!(conf.matches("blacklist sctp").count(), 1);

    // Update options
    store.add_options("ath9k_htc", vec!["nohwcrypt=1".to_string()]).expect("opt1");
    store.add_options("ath9k_htc", vec!["nohwcrypt=1".to_string(), "ps_enable=1".to_string()]).expect("opt2");
    assert_eq!(store.config.rules.len(), 2);

    let conf2 = store.export_modprobe_conf();
    assert_eq!(conf2.matches("options ath9k_htc").count(), 1);
    assert!(conf2.contains("ps_enable=1"));
}

#[test]
fn test_ks5_preset_integration_and_export() {
    let store = KernelModuleStore::new("preset-id", "preset test");
    let mut service = KernelModuleService::new(store);

    let presets = service.list_presets();
    assert_eq!(presets.len(), 3);

    service.apply_preset("cis_hardened_baseline").expect("apply cis");
    let modprobe_conf = service.store.export_modprobe_conf();
    assert!(modprobe_conf.contains("install cramfs /bin/true"));
    assert!(modprobe_conf.contains("blacklist cramfs"));
    assert!(modprobe_conf.contains("install tipc /bin/true"));

    service.apply_preset("container_isolation_baseline").expect("apply container");
    let modules_load = service.store.export_modules_load_conf();
    assert!(modules_load.contains("overlay\n"));
    assert!(modules_load.contains("tun\n"));
}

#[test]
fn test_oversized_store_refusal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store_path = dir.path().join("huge_store.json");

    // Create file exceeding MAX_MODULE_DOC_BYTES
    let huge_file = fs::File::create(&store_path).expect("create file");
    huge_file.set_len(MAX_MODULE_DOC_BYTES + 1024).expect("set len");
    drop(huge_file);

    let err = KernelModuleStore::load_from_path(&store_path).unwrap_err();
    assert!(err.contains("exceeds maximum limit"));
}

#[test]
fn test_oversized_proc_modules_refusal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mock_proc = dir.path().join("huge_modules");

    let file = fs::File::create(&mock_proc).expect("create file");
    file.set_len((aiosh_core::kernel_module_service::MAX_PROC_MODULES_BYTES + 1024) as u64).expect("set len");
    drop(file);

    let store = KernelModuleStore::new("test", "desc");
    let service = KernelModuleService::new(store).with_proc_modules_path(mock_proc);

    let err = service.list_loaded_modules().unwrap_err();
    assert!(err.contains("exceeds maximum limit"));
}

#[test]
fn test_overlong_proc_modules_line_refusal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mock_proc = dir.path().join("long_line_modules");

    let long_line = "a".repeat(aiosh_core::kernel_module_service::MAX_MODULE_LINE_BYTES + 10);
    fs::write(&mock_proc, format!("{}\n", long_line)).expect("write");

    let store = KernelModuleStore::new("test", "desc");
    let service = KernelModuleService::new(store).with_proc_modules_path(mock_proc);

    let err = service.list_loaded_modules().unwrap_err();
    assert!(err.contains("exceeds maximum length"));
}
