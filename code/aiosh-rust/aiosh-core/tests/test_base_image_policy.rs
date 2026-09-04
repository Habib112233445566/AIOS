//! Unit test suite for AIOS Linux Base Image Security Policy subsystem.
//! Validates criteria P1..P7.

use aiosh_core::base_image::*;
use aiosh_core::base_image_policy::*;
use aiosh_core::base_image_service::ImageStore;

#[test]
fn test_p1_kernel_hardening_invariants() {
    let policy = BaseImageSecurityPolicy::default();
    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);

    // Baseline passes
    assert!(policy.evaluate(&manifest).allowed);

    // Negative case: nokaslr
    manifest.kernel.cmdline = "console=tty0 nokaslr quiet".into();
    let v1 = policy.evaluate(&manifest);
    assert!(!v1.allowed);
    assert!(v1.violations.iter().any(|v| v.description.contains("nokaslr")));

    // Negative case: mitigations=off
    manifest.kernel.cmdline = "console=tty0 mitigations=off quiet".into();
    let v2 = policy.evaluate(&manifest);
    assert!(!v2.allowed);
    assert!(v2.violations.iter().any(|v| v.description.contains("mitigations=off")));

    // Negative case: pti=off
    manifest.kernel.cmdline = "console=tty0 pti=off quiet".into();
    let v3 = policy.evaluate(&manifest);
    assert!(!v3.allowed);
    assert!(v3.violations.iter().any(|v| v.description.contains("pti=off")));
}

#[test]
fn test_p2_lsm_invariants() {
    let policy = BaseImageSecurityPolicy::default();
    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);

    manifest.kernel.cmdline = "console=tty0 selinux=0 quiet".into();
    let v1 = policy.evaluate(&manifest);
    assert!(!v1.allowed);
    assert!(v1.violations.iter().any(|v| v.description.contains("selinux=0")));

    manifest.kernel.cmdline = "console=tty0 apparmor=0 quiet".into();
    let v2 = policy.evaluate(&manifest);
    assert!(!v2.allowed);
    assert!(v2.violations.iter().any(|v| v.description.contains("apparmor=0")));

    manifest.kernel.cmdline = "console=tty0 enforcing=0 quiet".into();
    let v3 = policy.evaluate(&manifest);
    assert!(!v3.allowed);
    assert!(v3.violations.iter().any(|v| v.description.contains("enforcing=0")));
}

#[test]
fn test_p3_init_bypass_invariants() {
    let policy = BaseImageSecurityPolicy::default();
    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);

    manifest.kernel.cmdline = "console=tty0 init=/bin/sh quiet".into();
    let v1 = policy.evaluate(&manifest);
    assert!(!v1.allowed);
    assert!(v1.violations.iter().any(|v| v.description.contains("init=/bin/sh")));

    manifest.kernel.cmdline = "console=tty0 single quiet".into();
    let v2 = policy.evaluate(&manifest);
    assert!(!v2.allowed);
    assert!(v2.violations.iter().any(|v| v.description.contains("single")));
}

#[test]
fn test_p4_package_blacklist() {
    let policy = BaseImageSecurityPolicy::default();
    let manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);

    let prohibited = ["telnet", "rsh-client", "rsh-server", "rlogin", "rexec", "nis", "yp-tools"];
    for bad_pkg in &prohibited {
        let mut test_manifest = manifest.clone();
        test_manifest.rootfs.packages.push((*bad_pkg).into());
        let verdict = policy.evaluate(&test_manifest);
        assert!(!verdict.allowed);
        assert!(verdict.violations.iter().any(|v| v.description.contains(bad_pkg)));
    }
}

#[test]
fn test_p5_arch_and_fs_whitelists() {
    let policy = BaseImageSecurityPolicy::default();
    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);

    // Disallowed arch
    manifest.rootfs.architecture = "mips".into();
    let v1 = policy.evaluate(&manifest);
    assert!(!v1.allowed);
    assert!(v1.violations.iter().any(|v| v.rule_id == "P5_ARCHITECTURE_WHITELIST"));

    // Reset arch, set disallowed fs
    manifest.rootfs.architecture = "x86_64".into();
    manifest.rootfs.filesystem_type = "ntfs".into();
    let v2 = policy.evaluate(&manifest);
    assert!(!v2.allowed);
    assert!(v2.violations.iter().any(|v| v.rule_id == "P6_FILESYSTEM_WHITELIST"));
}

#[test]
fn test_p6_enforcement_modes_and_policy_override() {
    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
    manifest.rootfs.packages.push("telnet".into());

    // Enforcing
    let mut policy = BaseImageSecurityPolicy::default();
    policy.mode = BaseImagePolicyMode::Enforcing;
    assert!(!policy.evaluate(&manifest).allowed);

    // Audit
    policy.mode = BaseImagePolicyMode::Audit;
    let audit_verdict = policy.evaluate(&manifest);
    assert!(audit_verdict.allowed);
    assert!(!audit_verdict.violations.is_empty());
    assert!(!audit_verdict.violations[0].fatal);

    // Permissive
    policy.mode = BaseImagePolicyMode::Permissive;
    let perm_verdict = policy.evaluate(&manifest);
    assert!(perm_verdict.allowed);
    assert!(perm_verdict.violations.is_empty());

    // Config override from provider closure
    let custom_policy = BaseImageSecurityPolicy::from_source(|k| match k {
        "AIOSH_BASE_IMAGE_POLICY_MODE" => Some("permissive".into()),
        "AIOSH_BASE_IMAGE_ALLOWED_ARCH" => Some("x86_64,aarch64".into()),
        _ => None,
    }).expect("valid policy");

    assert_eq!(custom_policy.mode, BaseImagePolicyMode::Permissive);
    assert_eq!(custom_policy.allowed_architectures, vec!["x86_64", "aarch64"]);
}

#[test]
fn test_p7_store_policy_filtering() {
    let policy = BaseImageSecurityPolicy::default();
    let mut store = ImageStore::new();

    // Inject one invalid image into store
    let mut bad_manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
    bad_manifest.id = "insecure-custom-raw".into();
    bad_manifest.rootfs.packages.push("telnet".into());
    store.register_image(bad_manifest).expect("registers fine in store");

    let verdicts = policy.check_all(&store);
    assert_eq!(verdicts.len(), 5);
    let allowed_count = verdicts.iter().filter(|v| v.allowed).count();
    assert_eq!(allowed_count, 4);

    let compliant = policy.filter_compliant_manifests(&store);
    assert_eq!(compliant.len(), 4);
    assert!(!compliant.iter().any(|m| m.id == "insecure-custom-raw"));
}
