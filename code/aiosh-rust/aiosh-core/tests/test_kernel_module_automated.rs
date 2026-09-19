//! Automated integration test suite for Kernel Module Management (T-01654).
//!
//! Enforces invariants AT-KM1..AT-KM4:
//! - AT-KM1: Compound state transitions across full lifecycle
//! - AT-KM2: Scale and density boundaries (1000 rules, 200 autoloads)
//! - AT-KM3: Document ceiling enforcement (10 MiB)
//! - AT-KM4: Corruption and truncation resilience

use aiosh_core::kernel_module::ModprobeRule;
use aiosh_core::kernel_module_service::{KernelModuleService, KernelModuleStore, MAX_MODULE_DOC_BYTES};
use tempfile::tempdir;

#[test]
fn test_at_km1_compound_lifecycle() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("lifecycle_store.json");

    // 1. Initialize store
    let mut store = KernelModuleStore::new("lifecycle-v1", "Test lifecycle compound");
    assert_eq!(store.config.rules.len(), 0);
    assert_eq!(store.config.autoload_modules.len(), 0);

    // 2. Blacklist modules
    store.add_blacklist("cramfs").expect("blacklist cramfs");
    store.add_blacklist("freevxfs").expect("blacklist freevxfs");
    assert_eq!(store.config.rules.len(), 2);

    // 3. Autoload modules
    store.add_autoload("overlay").expect("autoload overlay");
    store.add_autoload("kvm_intel").expect("autoload kvm_intel");
    assert_eq!(store.config.autoload_modules.len(), 2);

    // 4. Set options
    store
        .add_options("i915", vec!["enable_guc=3".into(), "enable_fbc=1".into()])
        .expect("set options");
    assert_eq!(store.config.rules.len(), 3);

    // 5. Conflict checks (pre-commit mutual exclusion)
    assert!(store.add_blacklist("overlay").is_err());
    assert!(store.add_autoload("cramfs").is_err());

    // 6. Save to disk and reload
    store.save_to_path(&store_path).expect("save store");
    let reloaded = KernelModuleStore::load_from_path(&store_path).expect("load store");
    assert_eq!(reloaded, store);

    // 7. Unautoload and Unblacklist
    assert!(store.remove_autoload("kvm_intel"));
    assert_eq!(store.config.autoload_modules.len(), 1);

    assert!(store.remove_blacklist("freevxfs"));
    assert_eq!(store.config.rules.len(), 2);

    // 8. Apply canonical preset
    let mut service = KernelModuleService::new(store);
    service.apply_preset("cis_hardened_baseline").expect("apply preset");
    assert!(service.store.config.rules.len() > 5);

    // 9. Export
    let modprobe_conf = service.store.export_modprobe_conf();
    assert!(modprobe_conf.contains("install cramfs /bin/true"));
    let modules_load_conf = service.store.export_modules_load_conf();
    assert!(modules_load_conf.contains("overlay"));
}

#[test]
fn test_at_km2_scale_limits() {
    let dir = tempdir().expect("tempdir");
    let store_path = dir.path().join("scale_store.json");

    let mut store = KernelModuleStore::new("scale-test", "Density and scale test");

    // Add 1000 rules
    for i in 0..1000 {
        let rule = ModprobeRule::Blacklist {
            module: format!("mod_{:04}", i),
        };
        store.config.rules.push(rule);
    }

    // Add 200 autoload modules
    for i in 0..200 {
        store.config.autoload_modules.push(format!("auto_{:04}", i));
    }

    // Validate, save, and reload
    store.validate().expect("large store should validate");
    store.save_to_path(&store_path).expect("should save 1200 entities");

    let reloaded = KernelModuleStore::load_from_path(&store_path).expect("should reload 1200 entities");
    assert_eq!(reloaded.config.rules.len(), 1000);
    assert_eq!(reloaded.config.autoload_modules.len(), 200);
}

#[test]
fn test_at_km3_document_size_ceiling() {
    let dir = tempdir().expect("tempdir");
    let oversized_file = dir.path().join("oversized.json");

    // Create a sparse/fake file exceeding 10 MiB
    let file = std::fs::File::create(&oversized_file).expect("create file");
    file.set_len(MAX_MODULE_DOC_BYTES + 1024).expect("set length > 10 MiB");

    // Attempting to load must fail AT-KM3
    let err = KernelModuleStore::load_from_path(&oversized_file).unwrap_err();
    assert!(err.contains("exceeds maximum limit"));
}

#[test]
fn test_at_km4_corrupted_store_handling() {
    let dir = tempdir().expect("tempdir");
    let corrupt_file = dir.path().join("corrupt_store.json");

    // Truncated JSON
    std::fs::write(&corrupt_file, r#"{"config": {"id": "test", "description": "broken"#).expect("write corrupt");

    let err = KernelModuleStore::load_from_path(&corrupt_file).unwrap_err();
    assert!(err.contains("failed to parse"));

    // Ensure file was not deleted or modified by failed read
    assert!(corrupt_file.exists());
}
