//! Automated test suite for AIOS Linux Base Image Build subsystem.
//! Enforces criteria T1..T4.

use aiosh_core::base_image::*;
use aiosh_core::base_image_config::ImageBuildConfig;
use aiosh_core::base_image_service::ImageStore;

#[test]
fn test_t1_build_plan_determinism() {
    let store = ImageStore::new();
    let id = "debian-12-minimal-raw";
    let first_plan = store.generate_build_plan(id).expect("initial plan");

    for _ in 0..50 {
        let plan = store.generate_build_plan(id).expect("subsequent plan");
        assert_eq!(first_plan.image_id, plan.image_id);
        assert_eq!(first_plan.target_format, plan.target_format);
        assert_eq!(first_plan.stages, plan.stages);
        assert_eq!(first_plan.estimated_artifact_size_bytes, plan.estimated_artifact_size_bytes);
        assert_eq!(first_plan.estimated_total_duration_secs, plan.estimated_total_duration_secs);
    }
}

#[test]
fn test_t2_registry_stress_and_bulk_query() {
    let mut store = ImageStore::new();
    for i in 0..20 {
        let manifest = BaseImageManifest {
            id: format!("custom-img-{:02}", i),
            version: "1.0.0".into(),
            format: if i % 2 == 0 { ImageFormat::Raw } else { ImageFormat::Qcow2 },
            rootfs: RootfsSpec {
                distro_id: format!("distro-{}", i % 4),
                architecture: "x86_64".into(),
                filesystem_type: "ext4".into(),
                packages: vec!["coreutils".into(), "systemd".into()],
                size_budget_bytes: (i as u64 + 1) * 100 * 1024 * 1024,
                hostname: format!("node-{:02}", i),
            },
            kernel: KernelSpec {
                version: "6.6.0".into(),
                cmdline: "console=ttyS0 quiet".into(),
                initramfs_generator: "dracut".into(),
            },
            created_at: "2026-09-03T12:00:00Z".into(),
            artifact_path: None,
            artifact_sha256: None,
            artifact_size_bytes: None,
        };
        store.register_image(manifest).expect("register");
    }

    assert_eq!(store.list_images().len(), 24); // 4 canonical + 20 synthetic
    assert!(store.get_image("custom-img-00").is_some());
    assert!(store.get_image("custom-img-19").is_some());
    assert!(store.get_image("custom-img-20").is_none());
}

#[test]
fn test_t3_configuration_override_resolution() {
    let default_cfg = ImageBuildConfig::default();
    assert_eq!(default_cfg.default_target, "debian-12-minimal-raw");
    assert_eq!(default_cfg.compression_level, 3);

    let tmp = tempfile::tempdir().unwrap();
    let cfg_file = tmp.path().join("test_cfg.json");

    let mut custom = ImageBuildConfig::default();
    custom.default_target = "alpine-319-container-tarball".into();
    custom.compression_level = 9;
    custom.save_to_path(&cfg_file).unwrap();

    let loaded = ImageBuildConfig::from_file(&cfg_file).unwrap();
    assert_eq!(loaded.default_target, "alpine-319-container-tarball");
    assert_eq!(loaded.compression_level, 9);
}

#[test]
fn test_t4_end_to_end_pipeline_cohesion() {
    let tmp = tempfile::tempdir().unwrap();
    let store_file = tmp.path().join("store.json");

    let mut store = ImageStore::new();
    let custom_id = "test-e2e-image";
    let manifest = BaseImageManifest {
        id: custom_id.into(),
        version: "2.1.0".into(),
        format: ImageFormat::Iso,
        rootfs: RootfsSpec {
            distro_id: "ubuntu-2404-server-aarch64".into(),
            architecture: "aarch64".into(),
            filesystem_type: "ext4".into(),
            packages: vec!["curl".into(), "iproute2".into()],
            size_budget_bytes: 800 * 1024 * 1024,
            hostname: "e2e-host".into(),
        },
        kernel: KernelSpec {
            version: "6.8.0-generic".into(),
            cmdline: "earlycon quiet splash".into(),
            initramfs_generator: "initramfs-tools".into(),
        },
        created_at: "2026-09-03T12:00:00Z".into(),
        artifact_path: None,
        artifact_sha256: None,
        artifact_size_bytes: None,
    };

    store.register_image(manifest).expect("register");
    store.save_to_path(&store_file).expect("save store");

    let reloaded = ImageStore::load_from_path(&store_file).expect("reload store");
    assert_eq!(reloaded.list_images().len(), 5);

    let plan = reloaded.generate_build_plan(custom_id).expect("generate plan");
    assert_eq!(plan.stages.len(), 4);
    assert_eq!(plan.image_id, custom_id);
    assert!(plan.estimated_total_duration_secs > 0);
}

#[test]
fn test_t5_invalid_manifest_rejections() {
    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
    manifest.version = "invalid_version".into();
    assert!(manifest.validate().is_err());

    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
    manifest.rootfs.packages.push("ILLEGAL_PACKAGE!".into());
    assert!(manifest.validate().is_err());

    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
    manifest.rootfs.size_budget_bytes = 20 * 1024 * 1024 * 1024; // > 10 GiB
    assert!(manifest.validate().is_err());

    let mut manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
    manifest.kernel.cmdline = "bad\ncmdline".into();
    assert!(manifest.validate().is_err());
}

#[test]
fn test_t6_mcp_and_cli_parity() {
    let store = ImageStore::new();
    let plan = store.generate_build_plan("debian-12-minimal-raw").unwrap();
    let stage_names: Vec<String> = plan.stages.iter().map(|s| s.name.clone()).collect();
    assert_eq!(
        stage_names,
        vec![
            "bootstrap".to_string(),
            "kernel_and_boot".to_string(),
            "system_config".to_string(),
            "artifact_packaging".to_string(),
        ]
    );
}

#[test]
fn test_t7_tempdir_cleanup_and_poisoned_registration() {
    let dir_path;
    {
        let tmp = tempfile::tempdir().unwrap();
        dir_path = tmp.path().to_path_buf();
        assert!(dir_path.exists());
    }
    assert!(!dir_path.exists());

    let mut store = ImageStore::new();
    let mut bad_manifest = BaseImageManifest::debian_12_minimal(ImageFormat::Raw);
    bad_manifest.id = "bad\x07id".into();
    assert!(store.register_image(bad_manifest).is_err());
}

