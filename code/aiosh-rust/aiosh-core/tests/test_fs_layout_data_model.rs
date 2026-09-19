//! Automated Unit Tests for Filesystem Layout Data Model (FL1..FL6)

use aiosh_core::fs_layout::{
    validate_directory_spec, validate_filesystem_layout, validate_mount_point,
    validate_partition_spec, DirectorySpec, FilesystemLayoutSpec, FsType, MountPointSpec,
    PartitionSpec, PartitionType,
};

#[test]
fn test_standard_uefi_layout_validity() {
    let layout = FilesystemLayoutSpec::standard_uefi();
    let res = validate_filesystem_layout(&layout);
    assert!(res.is_ok(), "standard_uefi layout should be valid: {:?}", res);
}

#[test]
fn test_directory_spec_validation() {
    let valid_dir = DirectorySpec {
        path: "/var/lib/aios".into(),
        mode: 0o750,
        owner: "root".into(),
        group: "aios".into(),
        description: "AIOS daemon state".into(),
        symlink_target: None,
    };
    assert!(validate_directory_spec(&valid_dir).is_ok());

    let invalid_dir = DirectorySpec {
        path: "var/lib/aios".into(), // relative path fails FL2
        mode: 0o750,
        owner: "root".into(),
        group: "aios".into(),
        description: "AIOS daemon state".into(),
        symlink_target: None,
    };
    assert!(validate_directory_spec(&invalid_dir).is_err());
}

#[test]
fn test_minimal_container_layout_validity() {
    let layout = FilesystemLayoutSpec::minimal_container();
    let res = layout.validate();
    assert!(res.is_ok(), "minimal_container layout should be valid: {:?}", res);
}

#[test]
fn test_fl1_single_root_mount_enforcement() {
    let mut layout = FilesystemLayoutSpec::standard_uefi();
    
    // Removing root mount must fail FL1
    layout.mounts.retain(|m| m.path != "/");
    let res = layout.validate();
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("FL1 violation"));

    // Adding duplicate root mount must fail FL1
    let mut layout2 = FilesystemLayoutSpec::standard_uefi();
    let root = layout2.mounts[0].clone();
    layout2.mounts.push(root);
    let res2 = layout2.validate();
    assert!(res2.is_err());
    assert!(res2.unwrap_err().contains("FL1 violation"));
}

#[test]
fn test_fl2_path_hygiene() {
    // Relative path
    let m_rel = MountPointSpec {
        path: "etc/aios".into(),
        device: "LABEL=ROOT".into(),
        fs_type: FsType::Ext4,
        options: vec!["defaults".into()],
        dump: 0,
        pass: 2,
        required: false,
    };
    assert!(validate_mount_point(&m_rel).is_err());

    // Path traversal
    let m_trav = MountPointSpec {
        path: "/var/../etc".into(),
        device: "LABEL=ROOT".into(),
        fs_type: FsType::Ext4,
        options: vec!["defaults".into()],
        dump: 0,
        pass: 2,
        required: false,
    };
    assert!(validate_mount_point(&m_trav).is_err());

    // Trailing slash
    let m_trail = MountPointSpec {
        path: "/var/".into(),
        device: "LABEL=ROOT".into(),
        fs_type: FsType::Ext4,
        options: vec!["defaults".into()],
        dump: 0,
        pass: 2,
        required: false,
    };
    assert!(validate_mount_point(&m_trail).is_err());
}

#[test]
fn test_fl3_mount_order_hierarchy() {
    let mut layout = FilesystemLayoutSpec::standard_uefi();
    // Swap /boot/efi (child) to appear before / (parent)
    let efi_idx = layout.mounts.iter().position(|m| m.path == "/boot/efi").unwrap();
    let root_idx = layout.mounts.iter().position(|m| m.path == "/").unwrap();
    layout.mounts.swap(efi_idx, root_idx);
    let res = layout.validate();
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("FL3 violation"));
}

#[test]
fn test_fl4_cis_security_options() {
    let m_insecure_tmp = MountPointSpec {
        path: "/tmp".into(),
        device: "tmpfs".into(),
        fs_type: FsType::Tmpfs,
        options: vec!["rw".into()], // missing nodev, nosuid
        dump: 0,
        pass: 0,
        required: true,
    };
    let res = validate_mount_point(&m_insecure_tmp);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("FL4 violation"));
}

#[test]
fn test_fl5_partition_constraints() {
    // Partition index 0 is invalid
    let p_zero = PartitionSpec {
        index: 0,
        label: "TEST".into(),
        partition_type: PartitionType::LinuxRoot,
        size_mib: 1024,
        uuid: None,
        bootable: false,
        format_as: Some(FsType::Ext4),
    };
    assert!(validate_partition_spec(&p_zero).is_err());

    // ESP formatted as ext4 is invalid
    let p_bad_esp = PartitionSpec {
        index: 1,
        label: "EFI".into(),
        partition_type: PartitionType::EfiSystem,
        size_mib: 512,
        uuid: None,
        bootable: true,
        format_as: Some(FsType::Ext4),
    };
    assert!(validate_partition_spec(&p_bad_esp).is_err());
}

#[test]
fn test_fstab_serialization_and_parsing() {
    let layout = FilesystemLayoutSpec::standard_uefi();
    let fstab_content = layout.generate_fstab();
    assert!(fstab_content.contains("/boot/efi"));
    assert!(fstab_content.contains("LABEL=AIOS_ROOT"));

    // Parse valid line
    let line = "LABEL=AIOS_ROOT   /   ext4   rw,relatime,errors=remount-ro   0   1";
    let parsed = MountPointSpec::parse_fstab_line(line).unwrap().unwrap();
    assert_eq!(parsed.path, "/");
    assert_eq!(parsed.device, "LABEL=AIOS_ROOT");
    assert_eq!(parsed.fs_type, FsType::Ext4);
    assert_eq!(parsed.dump, 0);
    assert_eq!(parsed.pass, 1);

    // Comment and empty lines
    assert!(MountPointSpec::parse_fstab_line("# this is a comment").unwrap().is_none());
    assert!(MountPointSpec::parse_fstab_line("   ").unwrap().is_none());
}

#[test]
fn test_partition_type_gpt_guid_mapping() {
    let esp = PartitionType::EfiSystem;
    assert_eq!(esp.gpt_type_guid(), "c12a7328-f81f-11d2-ba4b-00a0c93ec93b");
    assert_eq!(
        PartitionType::from_gpt_guid("C12A7328-F81F-11D2-BA4B-00A0C93EC93B"),
        PartitionType::EfiSystem
    );

    let root = PartitionType::LinuxRoot;
    assert_eq!(root.gpt_type_guid(), "4f68bce3-e8cd-4db1-96e7-fbcaf984b709");
    assert_eq!(
        PartitionType::from_gpt_guid("4f68bce3-e8cd-4db1-96e7-fbcaf984b709"),
        PartitionType::LinuxRoot
    );

    let custom = PartitionType::from_gpt_guid("12345678-1234-1234-1234-123456789abc");
    assert_eq!(
        custom,
        PartitionType::Custom("12345678-1234-1234-1234-123456789abc".into())
    );
}

#[test]
fn test_layout_json_roundtrip() {
    let layout = FilesystemLayoutSpec::standard_uefi();
    let json = layout.to_json().expect("serialization should succeed");
    let parsed = FilesystemLayoutSpec::from_json(&json).expect("deserialization should succeed");
    assert_eq!(layout.id, parsed.id);
    assert_eq!(layout.mounts.len(), parsed.mounts.len());
    assert_eq!(layout.partitions.len(), parsed.partitions.len());
    assert_eq!(layout.directories.len(), parsed.directories.len());
}

#[test]
fn test_fl1_root_pass_number_validation() {
    let mut layout = FilesystemLayoutSpec::standard_uefi();
    // Root with pass 0 must fail FL1
    layout.mounts[0].pass = 0;
    assert!(layout.validate().unwrap_err().contains("FL1 violation"));

    // Non-root with pass 1 must fail FL1
    let mut layout2 = FilesystemLayoutSpec::standard_uefi();
    layout2.mounts[1].pass = 1;
    assert!(layout2.validate().unwrap_err().contains("FL1 violation"));
}

#[test]
fn test_fl2_path_control_chars_and_oversized() {
    // Control characters (null byte, newline)
    let m_null = MountPointSpec {
        path: "/etc\0bad".into(),
        device: "LABEL=ROOT".into(),
        fs_type: FsType::Ext4,
        options: vec!["defaults".into()],
        dump: 0,
        pass: 2,
        required: false,
    };
    assert!(validate_mount_point(&m_null).unwrap_err().contains("FL2 violation"));

    let m_nl = MountPointSpec {
        path: "/var\nbad".into(),
        device: "LABEL=ROOT".into(),
        fs_type: FsType::Ext4,
        options: vec!["defaults".into()],
        dump: 0,
        pass: 2,
        required: false,
    };
    assert!(validate_mount_point(&m_nl).unwrap_err().contains("FL2 violation"));

    // Empty path component (double slash)
    let m_double = MountPointSpec {
        path: "/var//log".into(),
        device: "LABEL=ROOT".into(),
        fs_type: FsType::Ext4,
        options: vec!["defaults".into()],
        dump: 0,
        pass: 2,
        required: false,
    };
    assert!(validate_mount_point(&m_double).unwrap_err().contains("FL2 violation"));

    // Oversized path (> 1024 chars)
    let long_path = format!("/{}", "a".repeat(1025));
    let m_long = MountPointSpec {
        path: long_path,
        device: "LABEL=ROOT".into(),
        fs_type: FsType::Ext4,
        options: vec!["defaults".into()],
        dump: 0,
        pass: 2,
        required: false,
    };
    assert!(validate_mount_point(&m_long).unwrap_err().contains("FL2 violation"));
}

#[test]
fn test_fl3_duplicate_mount_paths() {
    let mut layout = FilesystemLayoutSpec::standard_uefi();
    // T-01543: carries noexec too, so the FL4 rule (extended by T-01542 D3)
    // stays satisfied and this test still pins what it was written to pin.
    let dup = MountPointSpec {
        path: "/tmp".into(),
        device: "tmpfs".into(),
        fs_type: FsType::Tmpfs,
        options: vec!["nodev".into(), "nosuid".into(), "noexec".into()],
        dump: 0,
        pass: 0,
        required: false,
    };
    layout.mounts.push(dup);
    let res = layout.validate();
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("FL3 violation: duplicate mount path"));
}

#[test]
fn test_fl4_dev_shm_cis_options() {
    let m_bad_shm = MountPointSpec {
        path: "/dev/shm".into(),
        device: "tmpfs".into(),
        fs_type: FsType::Tmpfs,
        options: vec!["rw".into(), "nodev".into()], // missing nosuid
        dump: 0,
        pass: 0,
        required: true,
    };
    let res = validate_mount_point(&m_bad_shm);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("FL4 violation"));
}

#[test]
fn test_fl5_total_partition_budget_exceeded() {
    let mut layout = FilesystemLayoutSpec::standard_uefi();
    layout.target_disk_min_bytes = 10 * 1024 * 1024 * 1024; // 10 GiB budget
    // Standard partitions total ~54.5 GiB
    let res = layout.validate();
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("FL5 violation: total partition size"));
}

#[test]
fn test_fl5_esp_mount_filesystem_mismatch() {
    let mut layout = FilesystemLayoutSpec::standard_uefi();
    // Set /boot/efi mount fs_type to ext4 while EFI partition exists
    let efi_mount = layout.mounts.iter_mut().find(|m| m.path == "/boot/efi").unwrap();
    efi_mount.fs_type = FsType::Ext4;
    let res = layout.validate();
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("FL5 violation: mount point /boot/efi must use vfat"));
}

#[test]
fn test_fstab_malformed_lines() {
    // Fewer than 4 fields
    assert!(MountPointSpec::parse_fstab_line("LABEL=ROOT / ext4").is_err());

    // More than 6 fields
    assert!(MountPointSpec::parse_fstab_line("LABEL=ROOT / ext4 rw 0 1 extra_field").is_err());

    // Non-integer dump
    assert!(MountPointSpec::parse_fstab_line("LABEL=ROOT / ext4 rw not_a_num 1").is_err());

    // Non-integer pass
    assert!(MountPointSpec::parse_fstab_line("LABEL=ROOT / ext4 rw 0 not_a_num").is_err());
}

// ---- T-01542 configuration contract (V-1..V-7) ----
// These tests are revert-controlled: against the pre-T-01543 validator every
// `*_rejects_*` test below fails, because the rule it pins did not exist.

fn base_spec() -> FilesystemLayoutSpec {
    let mut l = FilesystemLayoutSpec::standard_uefi();
    l.id = "cfg-contract-v1".into();
    l
}

#[test]
fn test_both_builtins_remain_valid_under_new_rules() {
    // Spec §9: every NEW rule is additive — the shipped presets must still pass.
    assert!(FilesystemLayoutSpec::standard_uefi().validate().is_ok());
    assert!(FilesystemLayoutSpec::minimal_container().validate().is_ok());
}

#[test]
fn test_unknown_json_field_rejected_top_level() {
    let mut l = base_spec();
    l.created_at = "2026-09-16T00:00:00Z".into();
    let mut v = serde_json::to_value(&l).unwrap();
    v.as_object_mut().unwrap().insert("dry_run".into(), serde_json::json!(true));
    let err = serde_json::from_value::<FilesystemLayoutSpec>(v).unwrap_err().to_string();
    assert!(err.contains("unknown field"), "E-1 wording, got: {err}");
    assert!(err.contains("dry_run"), "first offender named, got: {err}");
}

#[test]
fn test_unknown_json_field_rejected_nested() {
    let mut l = base_spec();
    l.created_at = "2026-09-16T00:00:00Z".into();
    let mut v = serde_json::to_value(&l).unwrap();
    v["mounts"][0].as_object_mut().unwrap().insert("no_such_mount_field".into(), serde_json::json!(1));
    let err = serde_json::from_value::<FilesystemLayoutSpec>(v).unwrap_err().to_string();
    assert!(err.contains("unknown field"), "E-1 wording, got: {err}");
    assert!(err.contains("no_such_mount_field"), "got: {err}");
}

#[test]
fn test_directory_mode_range_enforced() {
    let mut d = DirectorySpec {
        path: "/var/lib/aios".into(),
        mode: 0o750,
        owner: "root".into(),
        group: "aios".into(),
        description: "d".into(),
        symlink_target: None,
    };
    assert!(validate_directory_spec(&d).is_ok());
    d.mode = 0;
    let e = validate_directory_spec(&d).unwrap_err();
    assert!(e.contains("mode must be in 1..=0o7777 (octal), found 0"), "E-2 wording: {e}");
    d.mode = 0o10000; // first value beyond the 12 permission bits
    let e = validate_directory_spec(&d).unwrap_err();
    assert!(e.contains("found 4096"), "E-2 wording: {e}");
    d.mode = 0o7777; // sticky+setuid+setgid+rwx all round: legal
    assert!(validate_directory_spec(&d).is_ok());
}

#[test]
fn test_fl4_noexec_required_on_tmp_and_dev_shm() {
    for path in ["/tmp", "/dev/shm"] {
        let m = MountPointSpec {
            path: path.into(),
            device: "tmpfs".into(),
            fs_type: FsType::Tmpfs,
            options: vec!["rw".into(), "nodev".into(), "nosuid".into()], // no noexec
            dump: 0,
            pass: 0,
            required: true,
        };
        let e = validate_mount_point(&m).unwrap_err();
        assert!(e.contains("FL4 violation"), "got: {e}");
        assert!(e.contains("'noexec'"), "E-3 wording: {e}");
    }
}

#[test]
fn test_symlink_target_usrmerge_shape_enforced() {
    let mut d = DirectorySpec {
        path: "/bin".into(),
        mode: 0o777,
        owner: "root".into(),
        group: "root".into(),
        description: "usrmerge".into(),
        symlink_target: Some("usr/bin".into()),
    };
    assert!(validate_directory_spec(&d).is_ok());
    for bad in ["/usr/bin", "bin/usr", "usr/../etc", "usr/./bin"] {
        d.symlink_target = Some(bad.into());
        let e = validate_directory_spec(&d).unwrap_err();
        assert!(
            e.contains("must be a relative path under 'usr' (UsrMerge)"),
            "E-4 wording for '{bad}': {e}"
        );
    }
}

#[test]
fn test_created_at_must_be_rfc3339_utc() {
    let mut l = base_spec();
    for bad in ["yesterday", "2026-09-16T00:00:00", "2026-09-16T00:00:00+02:00"] {
        l.created_at = bad.into();
        let e = validate_filesystem_layout(&l).unwrap_err();
        assert!(
            e.contains("must be an RFC 3339 UTC timestamp"),
            "E-5 wording for '{bad}': {e}"
        );
    }
    l.created_at = "2026-09-16T00:00:00Z".into();
    assert!(validate_filesystem_layout(&l).is_ok());
}

#[test]
fn test_dump_bounded_to_zero_or_one() {
    let mut l = base_spec();
    l.mounts[0].dump = 2;
    let e = validate_filesystem_layout(&l).unwrap_err();
    assert!(e.contains("dump must be 0 or 1, found 2"), "E-6 wording: {e}");
    l.mounts[0].dump = 1;
    assert!(validate_filesystem_layout(&l).is_ok());
}

#[test]
fn test_fl6_at_least_one_required_mount() {
    let mut l = base_spec();
    for m in l.mounts.iter_mut() {
        m.required = false;
    }
    let e = validate_filesystem_layout(&l).unwrap_err();
    assert!(e.contains("FL6 violation: at least one mount must be marked required"), "E-7 wording: {e}");
    l.mounts[0].required = true;
    assert!(validate_filesystem_layout(&l).is_ok());
}

#[test]
fn test_error_ordering_matches_spec_section_4() {
    // Ordering rule: E-5 (top-level shape) precedes FL1; E-7 sits with FL1,
    // before the per-mount loop; E-2/E-4 (directories) come last.
    let mut l = base_spec();
    l.created_at = "not-a-date".into();
    for m in l.mounts.iter_mut() { m.required = false; }
    l.mounts[0].options.push("nodev".into()); // harmless
    let e = validate_filesystem_layout(&l).unwrap_err();
    assert!(e.contains("RFC 3339 UTC"), "E-5 must win over FL6: got {e}");

    let mut l = base_spec();
    for m in l.mounts.iter_mut() { m.required = false; }
    l.directories[0].mode = 0;
    let e = validate_filesystem_layout(&l).unwrap_err();
    assert!(e.contains("FL6 violation"), "FL6 must precede directory checks: got {e}");
}

#[test]
fn test_fs_type_roundtrip_and_custom() {
    use std::str::FromStr;

    assert_eq!(FsType::from_str("ext4").unwrap(), FsType::Ext4);
    assert_eq!(FsType::from_str("btrfs").unwrap(), FsType::Btrfs);
    assert_eq!(FsType::from_str("xfs").unwrap(), FsType::Xfs);
    assert_eq!(FsType::from_str("vfat").unwrap(), FsType::Vfat);
    assert_eq!(FsType::from_str("fat32").unwrap(), FsType::Vfat);
    assert_eq!(FsType::from_str("swap").unwrap(), FsType::Swap);
    assert_eq!(
        FsType::from_str("zfs").unwrap(),
        FsType::Custom("zfs".into())
    );

    assert_eq!(FsType::Ext4.to_string(), "ext4");
    assert_eq!(FsType::Vfat.to_string(), "vfat");
    assert_eq!(FsType::Custom("myfs".into()).to_string(), "myfs");
}


