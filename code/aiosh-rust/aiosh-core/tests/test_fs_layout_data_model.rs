//! Automated Unit Tests for Filesystem Layout Data Model (FL1..FL5)

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
    let dup = MountPointSpec {
        path: "/tmp".into(),
        device: "tmpfs".into(),
        fs_type: FsType::Tmpfs,
        options: vec!["nodev".into(), "nosuid".into()],
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


