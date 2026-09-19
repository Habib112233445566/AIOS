//! Unit test suite for AIOS Kernel Module Management Security Policy (SP-KM1..SP-KM6).

use aiosh_core::kernel_module::ModprobeRule;
use aiosh_core::kernel_module_policy::*;
use aiosh_core::kernel_module_service::KernelModuleStore;

#[test]
fn test_sp_km1_policy_bounds_and_disjointness() {
    let mut policy = KernelModuleSecurityPolicy::default();
    assert!(policy.validate().is_ok());
    assert_eq!(policy.mode, KernelModulePolicyMode::Enforcing);

    // Negative: disjointness violation (module in both prohibited and protected)
    policy.prohibited_modules.push("ext4".into());
    let err1 = policy.validate();
    assert!(err1.is_err());
    assert!(err1.unwrap_err().contains("both prohibited and protected"));

    policy.prohibited_modules.pop();

    // Negative: invalid prohibited module name
    policy.prohibited_modules.push("bad/name".into());
    let err2 = policy.validate();
    assert!(err2.is_err());
    policy.prohibited_modules.pop();

    // Negative: relative install command
    policy.allowed_install_commands.push("bin/true".into());
    let err3 = policy.validate();
    assert!(err3.is_err());
    assert!(err3.unwrap_err().contains("must be absolute path"));
    policy.allowed_install_commands.pop();

    // Negative: install command with path traversal
    policy.allowed_install_commands.push("/bin/../bin/true".into());
    let err4 = policy.validate();
    assert!(err4.is_err());
    assert!(err4.unwrap_err().contains("path traversal"));
    policy.allowed_install_commands.pop();

    // Negative: max_parameter_value_len out of bounds
    policy.max_parameter_value_len = 0;
    let err5 = policy.validate();
    assert!(err5.is_err());
    assert!(err5.unwrap_err().contains("max_parameter_value_len out of range"));
}

#[test]
fn test_sp_km2_prohibited_module_enforcement() {
    let policy = KernelModuleSecurityPolicy::default();

    // Positive: blacklisting a prohibited module is allowed and encouraged
    let rule_bl = ModprobeRule::Blacklist {
        module: "cramfs".into(),
    };
    let verdict_bl = policy.evaluate_rule(&rule_bl);
    assert!(verdict_bl.allowed);
    assert!(verdict_bl.violations.is_empty());

    // Negative: setting options on a prohibited module is forbidden
    let rule_opt = ModprobeRule::Options {
        module: "cramfs".into(),
        options: vec!["debug=1".into()],
    };
    let verdict_opt = policy.evaluate_rule(&rule_opt);
    assert!(!verdict_opt.allowed);
    assert!(verdict_opt.violations.iter().any(|v| v.rule_id == "SP-KM2-PROHIBITED-MODULE"));

    // Negative: autoloading a prohibited module is forbidden
    let verdict_al = policy.evaluate_autoload("dccp");
    assert!(!verdict_al.allowed);
    assert!(verdict_al.violations.iter().any(|v| v.rule_id == "SP-KM2-PROHIBITED-AUTOLOAD"));

    // Positive: autoloading an allowed module
    let verdict_al_ok = policy.evaluate_autoload("dummy_net");
    assert!(verdict_al_ok.allowed);
    assert!(verdict_al_ok.violations.is_empty());
}

#[test]
fn test_sp_km3_protected_module_guard() {
    let policy = KernelModuleSecurityPolicy::default();

    // Negative: blacklisting a protected module is blocked
    let rule_ext4_bl = ModprobeRule::Blacklist {
        module: "ext4".into(),
    };
    let verdict_ext4 = policy.evaluate_rule(&rule_ext4_bl);
    assert!(!verdict_ext4.allowed);
    assert!(verdict_ext4.violations.iter().any(|v| v.rule_id == "SP-KM3-PROTECTED-MODULE"));

    // Negative: disabling a protected module via install /bin/false is blocked
    let rule_overlay_dis = ModprobeRule::Install {
        module: "overlay".into(),
        command: "/bin/false".into(),
    };
    let verdict_overlay = policy.evaluate_rule(&rule_overlay_dis);
    assert!(!verdict_overlay.allowed);
    assert!(verdict_overlay.violations.iter().any(|v| v.rule_id == "SP-KM3-PROTECTED-MODULE"));

    // Negative: blacklisting dm_mod is blocked
    let rule_dm = ModprobeRule::Blacklist {
        module: "dm_mod".into(),
    };
    assert!(!policy.evaluate_rule(&rule_dm).allowed);

    // Positive: non-protected module can be blacklisted
    let rule_floppy = ModprobeRule::Blacklist {
        module: "floppy".into(),
    };
    assert!(policy.evaluate_rule(&rule_floppy).allowed);
}

#[test]
fn test_sp_km4_install_command_sanitization() {
    let policy = KernelModuleSecurityPolicy::default();

    // Positive: standard /bin/true install directive is allowed
    let rule_ok = ModprobeRule::Install {
        module: "usb_storage".into(),
        command: "/bin/true".into(),
    };
    let verdict_ok = policy.evaluate_rule(&rule_ok);
    assert!(verdict_ok.allowed);
    assert!(verdict_ok.violations.is_empty());

    // Negative: unapproved binary in install directive
    let rule_unapproved = ModprobeRule::Install {
        module: "usb_storage".into(),
        command: "/usr/bin/python3 -c malicious()".into(),
    };
    let verdict_unapproved = policy.evaluate_rule(&rule_unapproved);
    assert!(!verdict_unapproved.allowed);
    assert!(verdict_unapproved.violations.iter().any(|v| v.rule_id == "SP-KM4-UNAPPROVED-INSTALL-CMD"));

    // Negative: command injection attempt
    let rule_inject = ModprobeRule::Install {
        module: "usb_storage".into(),
        command: "/bin/true; rm -rf /".into(),
    };
    let verdict_inject = policy.evaluate_rule(&rule_inject);
    assert!(!verdict_inject.allowed);
    assert!(verdict_inject.violations.iter().any(|v| v.rule_id == "SP-KM4-INSTALL-COMMAND-INJECTION"));
}

#[test]
fn test_sp_km5_parameter_inspection_and_bounds() {
    let mut policy = KernelModuleSecurityPolicy::default();
    policy.max_parameter_value_len = 32;

    // Negative: disallowed parameter key
    let rule_disallowed_key = ModprobeRule::Options {
        module: "e1000e".into(),
        options: vec!["panic=1".into()],
    };
    let verdict_key = policy.evaluate_rule(&rule_disallowed_key);
    assert!(!verdict_key.allowed);
    assert!(verdict_key.violations.iter().any(|v| v.rule_id == "SP-KM5-DISALLOWED-PARAM-KEY"));

    // Negative: parameter value too long
    let rule_too_long = ModprobeRule::Options {
        module: "e1000e".into(),
        options: vec!["debug=this_is_an_extremely_long_parameter_value_that_exceeds_32_bytes".into()],
    };
    let verdict_len = policy.evaluate_rule(&rule_too_long);
    assert!(!verdict_len.allowed);
    assert!(verdict_len.violations.iter().any(|v| v.rule_id == "SP-KM5-PARAM-TOO-LONG"));

    // Negative: shell metacharacter in parameter value
    let rule_injection = ModprobeRule::Options {
        module: "e1000e".into(),
        options: vec!["debug=1;reboot".into()],
    };
    let verdict_inj = policy.evaluate_rule(&rule_injection);
    assert!(!verdict_inj.allowed);
    assert!(verdict_inj.violations.iter().any(|v| v.rule_id == "SP-KM5-DANGEROUS-PARAM-VALUE"));

    // Positive: valid options
    let rule_valid = ModprobeRule::Options {
        module: "e1000e".into(),
        options: vec!["InterruptThrottleRate=1".into(), "SmartPowerDownEnable=0".into()],
    };
    let verdict_val = policy.evaluate_rule(&rule_valid);
    assert!(verdict_val.allowed);
    assert!(verdict_val.violations.is_empty());
}

#[test]
fn test_sp_km6_tri_state_modes_and_store_evaluation() {
    let mut policy = KernelModuleSecurityPolicy::default();

    // 1. Enforcing mode: fatal violation blocks
    let rule_bad = ModprobeRule::Options {
        module: "cramfs".into(),
        options: vec!["debug=1".into()],
    };
    assert!(!policy.evaluate_rule(&rule_bad).allowed);

    // 2. Audit mode: fatal violation does NOT block, but violation is recorded
    policy.mode = KernelModulePolicyMode::Audit;
    let verdict_audit = policy.evaluate_rule(&rule_bad);
    assert!(verdict_audit.allowed);
    assert_eq!(verdict_audit.violations.len(), 1);
    assert_eq!(verdict_audit.violations[0].rule_id, "SP-KM2-PROHIBITED-MODULE");

    // 3. Permissive mode: prohibited module option is allowed, but protected module blacklisting is blocked
    policy.mode = KernelModulePolicyMode::Permissive;
    let verdict_permissive_ok = policy.evaluate_rule(&rule_bad);
    assert!(verdict_permissive_ok.allowed);

    let rule_protected_bl = ModprobeRule::Blacklist {
        module: "ext4".into(),
    };
    let verdict_permissive_block = policy.evaluate_rule(&rule_protected_bl);
    assert!(!verdict_permissive_block.allowed);
    assert!(verdict_permissive_block.violations.iter().any(|v| v.rule_id == "SP-KM3-PROTECTED-MODULE"));

    // 4. evaluate_store
    policy.mode = KernelModulePolicyMode::Enforcing;
    let mut store = KernelModuleStore::new("test-store", "Test Store");
    store.add_blacklist("floppy").unwrap();
    store.config.autoload_modules.push("dummy_net".into());

    let verdicts = policy.evaluate_store(&store);
    assert_eq!(verdicts.len(), 2);
    assert!(verdicts.iter().all(|v| v.allowed));

    // 5. from_source environment parsing
    let policy_env = KernelModuleSecurityPolicy::from_source(|k| {
        if k == "AIOS_KERNEL_MODULE_POLICY_MODE" {
            Some("audit".into())
        } else {
            None
        }
    }).unwrap();
    assert_eq!(policy_env.mode, KernelModulePolicyMode::Audit);
}
