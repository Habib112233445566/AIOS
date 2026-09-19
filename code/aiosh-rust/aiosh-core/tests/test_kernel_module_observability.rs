//! Unit test suite for AIOS Kernel Module Management Observability Subsystem (KO1..KO6).

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use aiosh_core::kernel_module::ModprobeRule;
use aiosh_core::kernel_module_observability::*;
use aiosh_core::kernel_module_policy::KernelModuleSecurityPolicy;
use aiosh_core::kernel_module_service::{KernelModuleService, KernelModuleStore};

#[test]
fn test_ko1_empty_store_and_procfs_fallback() {
    let store = KernelModuleStore::new("test-empty", "Empty store");
    let mut service = KernelModuleService::new(store);
    service = service.with_proc_modules_path(PathBuf::from("nonexistent/proc/modules"));

    let report = KernelModuleObservabilityReport::generate(&service, None);
    assert_eq!(report.total_loaded_modules, 0);
    assert_eq!(report.total_memory_bytes, 0);
    assert_eq!(report.store_rules_count, 0);
    assert_eq!(report.autoload_modules_count, 0);
    assert_eq!(report.policy_compliant_count, 0);
    assert_eq!(report.policy_violations_count, 0);
    assert!(report.prohibited_modules_configured.is_empty());
    assert!(report.protected_modules_configured.is_empty());
}

#[test]
fn test_ko2_mock_procfs_module_aggregation_and_memory() {
    let td = tempdir().unwrap();
    let proc_modules_path = td.path().join("modules");

    let procfs_content = "\
overlay 151552 1 - Live 0x0000000000000000
ext4 983040 2 - Live 0x0000000000000000
nfs 311296 0 - Loading 0x0000000000000000
";
    fs::write(&proc_modules_path, procfs_content).unwrap();

    let store = KernelModuleStore::new("test-procfs", "Procfs test");
    let mut service = KernelModuleService::new(store);
    service = service.with_proc_modules_path(proc_modules_path);

    let report = KernelModuleObservabilityReport::generate(&service, None);

    // KO1 & KO3: Module count and memory
    assert_eq!(report.total_loaded_modules, 3);
    assert_eq!(report.total_memory_bytes, 151552 + 983040 + 311296);

    // KO2: State breakdown
    assert_eq!(report.state_breakdown.get("live"), Some(&2));
    assert_eq!(report.state_breakdown.get("loading"), Some(&1));
    assert_eq!(report.state_breakdown.get("unloading"), None);

    // KO4: Refcount distribution
    assert_eq!(report.ref_count_distribution.get("0"), Some(&1));
    assert_eq!(report.ref_count_distribution.get("1-2"), Some(&2));
    assert_eq!(report.ref_count_distribution.get("3-5"), Some(&0));
    assert_eq!(report.ref_count_distribution.get("6+"), Some(&0));
}

#[test]
fn test_ko3_rule_type_distribution_and_autoload() {
    let mut store = KernelModuleStore::new("test-rules", "Rules test");
    store.config.rules.push(ModprobeRule::Blacklist {
        module: "floppy".into(),
    });
    store.config.rules.push(ModprobeRule::Alias {
        alias: "net-pf-10".into(),
        module: "ipv6".into(),
    });
    store.config.rules.push(ModprobeRule::Options {
        module: "e1000e".into(),
        options: vec!["InterruptThrottleRate=1".into()],
    });
    store.config.rules.push(ModprobeRule::Install {
        module: "usb_storage".into(),
        command: "/bin/true".into(),
    });
    store.config.autoload_modules.push("dummy_net".into());

    let service = KernelModuleService::new(store);
    let report = KernelModuleObservabilityReport::generate(&service, None);

    assert_eq!(report.store_rules_count, 4);
    assert_eq!(report.autoload_modules_count, 1);
    assert_eq!(report.rule_type_breakdown.get("blacklist"), Some(&1));
    assert_eq!(report.rule_type_breakdown.get("alias"), Some(&1));
    assert_eq!(report.rule_type_breakdown.get("options"), Some(&1));
    assert_eq!(report.rule_type_breakdown.get("install"), Some(&1));
    assert_eq!(report.rule_type_breakdown.get("remove"), None);
}

#[test]
fn test_ko4_policy_compliance_and_prohibited_tracking() {
    let mut store = KernelModuleStore::new("test-policy", "Policy test");
    // Prohibited module options -> violation
    store.config.rules.push(ModprobeRule::Options {
        module: "cramfs".into(),
        options: vec!["debug=1".into()],
    });
    // Protected module blacklist -> violation
    store.config.rules.push(ModprobeRule::Blacklist {
        module: "ext4".into(),
    });
    // Clean rule -> compliant
    store.config.rules.push(ModprobeRule::Blacklist {
        module: "floppy".into(),
    });
    // Prohibited autoload -> violation
    store.config.autoload_modules.push("dccp".into());

    let service = KernelModuleService::new(store);
    let policy = KernelModuleSecurityPolicy::default();
    let report = KernelModuleObservabilityReport::generate(&service, Some(&policy));

    assert_eq!(report.store_rules_count, 3);
    assert_eq!(report.autoload_modules_count, 1);
    // Total items evaluated: 3 rules + 1 autoload = 4
    // 1 clean (floppy) -> policy_compliant_count = 1
    assert_eq!(report.policy_compliant_count, 1);
    assert!(report.policy_violations_count >= 3);

    // Prohibited modules tracked
    assert!(report.prohibited_modules_configured.contains(&"cramfs".to_string()));
    assert!(report.prohibited_modules_configured.contains(&"dccp".to_string()));

    // Protected modules tracked
    assert!(report.protected_modules_configured.contains(&"ext4".to_string()));
}

#[test]
fn test_ko5_deterministic_serialization() {
    let store = KernelModuleStore::new("test-ser", "Serialization test");
    let service = KernelModuleService::new(store);
    let report = KernelModuleObservabilityReport::generate(&service, None);

    let json_str = serde_json::to_string_pretty(&report).unwrap();
    let deserialized: KernelModuleObservabilityReport = serde_json::from_str(&json_str).unwrap();
    assert_eq!(report, deserialized);
}
