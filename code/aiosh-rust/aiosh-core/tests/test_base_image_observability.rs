//! Unit test suite for AIOS Linux Base Image Observability subsystem.
//! Validates criteria OB1..OB5.

use aiosh_core::base_image::*;
use aiosh_core::base_image_observability::BaseImageObservabilityReport;
use aiosh_core::base_image_policy::{BaseImagePolicyMode, BaseImageSecurityPolicy};
use aiosh_core::base_image_service::ImageStore;

#[test]
fn test_ob1_ob2_ob3_categorical_breakdowns() {
    let store = ImageStore::new();
    let report = BaseImageObservabilityReport::generate(&store, None);

    assert_eq!(report.total_images, 4);
    assert!(report.validate().is_ok());

    // OB1
    let fmt_sum: usize = report.format_breakdown.values().sum();
    assert_eq!(fmt_sum, report.total_images);
    assert_eq!(report.format_breakdown.get("raw"), Some(&1));
    assert_eq!(report.format_breakdown.get("qcow2"), Some(&1));
    assert_eq!(report.format_breakdown.get("iso"), Some(&1));
    assert_eq!(report.format_breakdown.get("tarball"), Some(&1));

    // OB2
    let arch_sum: usize = report.architecture_breakdown.values().sum();
    assert_eq!(arch_sum, report.total_images);
    assert_eq!(report.architecture_breakdown.get("x86_64"), Some(&4));

    // OB3
    let distro_sum: usize = report.distro_breakdown.values().sum();
    assert_eq!(distro_sum, report.total_images);
    assert_eq!(report.distro_breakdown.get("debian-12-minimal-x86_64"), Some(&3));
    assert_eq!(report.distro_breakdown.get("alpine-319-container-x86_64"), Some(&1));
}

#[test]
fn test_ob4_policy_compliance_tracking() {
    let mut store = ImageStore::new();

    // Baseline: 4 images, all 4 compliant
    let default_policy = BaseImageSecurityPolicy::default();
    let r1 = BaseImageObservabilityReport::generate(&store, Some(&default_policy));
    assert_eq!(r1.policy_compliant_count, 4);

    // Add 1 non-compliant image with telnet
    let mut bad_manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
    bad_manifest.id = "insecure-debian".into();
    bad_manifest.rootfs.packages.push("telnet".into());
    store.register_image(bad_manifest).expect("register");

    // In Enforcing mode: 4/5 compliant
    let r2 = BaseImageObservabilityReport::generate(&store, Some(&default_policy));
    assert_eq!(r2.total_images, 5);
    assert_eq!(r2.policy_compliant_count, 4);
    assert!(r2.validate().is_ok());

    // In Permissive mode: 5/5 compliant
    let mut permissive_policy = BaseImageSecurityPolicy::default();
    permissive_policy.mode = BaseImagePolicyMode::Permissive;
    let r3 = BaseImageObservabilityReport::generate(&store, Some(&permissive_policy));
    assert_eq!(r3.policy_compliant_count, 5);
}

#[test]
fn test_ob5_size_budget_and_averages() {
    let store = ImageStore::new();
    let report = BaseImageObservabilityReport::generate(&store, None);

    assert!(report.total_size_budget_bytes > 0);
    assert_eq!(
        report.average_size_budget_bytes,
        report.total_size_budget_bytes / (report.total_images as u64)
    );

    // Empty store
    let empty_store = ImageStore::empty();
    let empty_report = BaseImageObservabilityReport::generate(&empty_store, None);
    assert_eq!(empty_report.total_images, 0);
    assert_eq!(empty_report.total_size_budget_bytes, 0);
    assert_eq!(empty_report.average_size_budget_bytes, 0);
    assert!(empty_report.validate().is_ok());
}

#[test]
fn test_kernel_version_aggregation() {
    let store = ImageStore::new();
    let report = BaseImageObservabilityReport::generate(&store, None);

    assert_eq!(report.kernel_versions.len(), 2);
    assert!(report.kernel_versions.iter().any(|v| v.contains("6.1.0-28-amd64")));
    assert!(report.kernel_versions.iter().any(|v| v.contains("6.1.66-0-lts")));
}

#[test]
fn test_synthetic_scale_and_negative_invariants() {
    let mut store = ImageStore::empty();
    for i in 0..25 {
        let manifest = BaseImageManifest {
            id: format!("scale-img-{:02}", i),
            version: "1.0.0".into(),
            format: match i % 4 {
                0 => ImageFormat::Raw,
                1 => ImageFormat::Qcow2,
                2 => ImageFormat::Iso,
                _ => ImageFormat::Tarball,
            },
            rootfs: RootfsSpec {
                distro_id: format!("distro-{}", i % 3),
                architecture: if i % 2 == 0 { "x86_64".into() } else { "aarch64".into() },
                filesystem_type: "ext4".into(),
                packages: vec!["base-files".into()],
                size_budget_bytes: (i as u64 + 1) * 50 * 1024 * 1024,
                hostname: format!("scale-node-{}", i),
            },
            kernel: KernelSpec {
                version: format!("6.8.{}", i % 5),
                cmdline: "quiet".into(),
                initramfs_generator: "dracut".into(),
            },
            created_at: "2026-09-04T00:00:00Z".into(),
            artifact_path: None,
            artifact_sha256: None,
            artifact_size_bytes: None,
        };
        store.register_image(manifest).expect("registers");
    }

    let report = BaseImageObservabilityReport::generate(&store, None);
    assert_eq!(report.total_images, 25);
    assert!(report.validate().is_ok());

    // Negative validation tests
    let mut bad1 = report.clone();
    bad1.format_breakdown.insert("raw".into(), 999);
    assert!(bad1.validate().is_err());

    let mut bad2 = report.clone();
    bad2.architecture_breakdown.insert("x86_64".into(), 999);
    assert!(bad2.validate().is_err());

    let mut bad3 = report.clone();
    bad3.policy_compliant_count = 1000;
    assert!(bad3.validate().is_err());

    let mut bad4 = report.clone();
    bad4.average_size_budget_bytes += 100;
    assert!(bad4.validate().is_err());
}
