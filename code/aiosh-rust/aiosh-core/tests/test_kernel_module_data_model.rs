//! Integration Tests for Kernel Module Management Data Model (KM1..KM5)

use aiosh_core::kernel_module::{
    cis_hardened_preset, container_isolation_preset, parse_modprobe_conf,
    pentest_wireless_preset, validate_config, validate_module_name, validate_parameter,
    ModprobeRule, ModuleInfo, ModuleState,
};

#[test]
fn test_km1_module_name_boundary_and_syntax() {
    // Valid names
    assert!(validate_module_name("overlay").is_ok());
    assert!(validate_module_name("ath9k_htc").is_ok());
    assert!(validate_module_name("br_netfilter").is_ok());
    assert!(validate_module_name("nf_tables").is_ok());
    assert!(validate_module_name("wireguard").is_ok());
    assert!(validate_module_name("tun").is_ok());

    // Boundary: min length (1 char)
    assert!(validate_module_name("a").is_ok());
    assert!(validate_module_name("1").is_ok());

    // Boundary: max length (64 chars)
    let max_len_name = "a".repeat(64);
    assert!(validate_module_name(&max_len_name).is_ok());

    // Negative: oversized (65 chars)
    let over_len_name = "a".repeat(65);
    assert!(validate_module_name(&over_len_name).is_err());

    // Negative: empty
    assert!(validate_module_name("").is_err());

    // Negative: invalid characters
    assert!(validate_module_name("module-with-dash").is_err());
    assert!(validate_module_name("module.ko").is_err());
    assert!(validate_module_name("../etc/passwd").is_err());
    assert!(validate_module_name("mod;rm").is_err());
    assert!(validate_module_name("mod|sh").is_err());
    assert!(validate_module_name("mod`id`").is_err());
    assert!(validate_module_name("mod$HOME").is_err());
}

#[test]
fn test_km2_parameter_safety() {
    assert!(validate_parameter("metacopy", "on").is_ok());
    assert!(validate_parameter("nohwcrypt", "1").is_ok());
    assert!(validate_parameter("rtw_vht_enable", "1").is_ok());
    assert!(validate_parameter("ports", "80,443").is_ok());

    // Negative: empty key
    assert!(validate_parameter("", "val").is_err());

    // Negative: illegal characters in value
    assert!(validate_parameter("key", "val;reboot").is_err());
    assert!(validate_parameter("key", "val|sh").is_err());
    assert!(validate_parameter("key", "val&sh").is_err());
    assert!(validate_parameter("key", "val`id`").is_err());
    assert!(validate_parameter("key", "val$UID").is_err());
    assert!(validate_parameter("key", "val\nkey2=val2").is_err());

    // Negative: oversized value (> 1024 bytes)
    let long_val = "x".repeat(1025);
    assert!(validate_parameter("key", &long_val).is_err());
}

#[test]
fn test_km3_conflict_invariants() {
    let mut config = container_isolation_preset().config;
    assert!(validate_config(&config).is_ok());

    // Conflict: Autoloaded module cannot be blacklisted
    config.rules.push(ModprobeRule::Blacklist {
        module: "tun".to_string(),
    });
    let err = validate_config(&config).unwrap_err();
    assert!(err.contains("cannot be both autoloaded and blacklisted"));

    // Conflict: Autoloaded module cannot be disabled via install /bin/true
    let mut config2 = container_isolation_preset().config;
    config2.rules.push(ModprobeRule::Install {
        module: "overlay".to_string(),
        command: "/bin/true".to_string(),
    });
    let err2 = validate_config(&config2).unwrap_err();
    assert!(err2.contains("cannot be both autoloaded and disabled"));
}

#[test]
fn test_km4_cis_hardened_preset_completeness() {
    let preset = cis_hardened_preset();
    assert_eq!(preset.name, "cis_hardened_baseline");
    assert!(validate_config(&preset.config).is_ok());

    let conf = preset.config.to_modprobe_conf();
    let rules = parse_modprobe_conf(&conf).expect("parsed successfully");

    // Must disable legacy filesystems
    for fs in &["cramfs", "freevxfs", "jffs2", "hfs", "hfsplus", "udf"] {
        assert!(rules.iter().any(|r| match r {
            ModprobeRule::Install { module, command } => module == fs && command.contains("/bin/true"),
            _ => false,
        }));
        assert!(rules.iter().any(|r| match r {
            ModprobeRule::Blacklist { module } => module == fs,
            _ => false,
        }));
    }

    // Must disable obsolete protocols
    for proto in &["dccp", "sctp", "rds", "tipc"] {
        assert!(rules.iter().any(|r| match r {
            ModprobeRule::Install { module, command } => module == proto && command.contains("/bin/true"),
            _ => false,
        }));
        assert!(rules.iter().any(|r| match r {
            ModprobeRule::Blacklist { module } => module == proto,
            _ => false,
        }));
    }
}

#[test]
fn test_km5_modprobe_and_autoload_generation_roundtrip() {
    let preset = pentest_wireless_preset();
    let modprobe_conf = preset.config.to_modprobe_conf();
    let autoload_conf = preset.config.to_modules_load_conf();

    // Verify modprobe.d roundtrip
    let parsed_rules = parse_modprobe_conf(&modprobe_conf).expect("parsed successfully");
    assert_eq!(parsed_rules, preset.config.rules);

    // Verify modules-load.d content
    assert!(autoload_conf.contains("cfg80211\n"));
    assert!(autoload_conf.contains("mac80211\n"));
}

#[test]
fn test_proc_modules_real_world_samples() {
    let samples = [
        "overlay 151552 1 - Live 0x0000000000000000",
        "tun 61440 2 - Live 0x0000000000000000",
        "ath9k_htc 90112 0 - Live 0x0000000000000000",
        "mac80211 983040 1 ath9k_htc, Live 0x0000000000000000",
        "cfg80211 884736 2 ath9k_htc,mac80211, Live 0x0000000000000000",
        "wireguard 94208 0 - Live 0x0000000000000000",
    ];

    for line in samples {
        let info = ModuleInfo::parse_proc_modules_line(line).expect("failed parsing line");
        assert_eq!(info.state, ModuleState::Live);
        assert!(info.size_bytes > 0);
    }
}
