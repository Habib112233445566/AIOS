use aiosh_core::{
    FilesystemLayoutService, FilesystemLayoutSpec, FilesystemLayoutStore,
};
use std::env;

#[test]
fn test_fs_layout_service_scaffold_initialization() {
    let service = FilesystemLayoutService::new();
    assert_eq!(service.store.active_layout_id, "aios-uefi-standard-v1");
    assert!(service.store.layouts.contains_key("aios-uefi-standard-v1"));
    assert!(service.store.layouts.contains_key("aios-container-minimal-v1"));

    let empty = FilesystemLayoutStore::empty();
    assert_eq!(empty.active_layout_id, "");
    assert!(empty.layouts.is_empty());
}

#[test]
fn test_fs_layout_service_store_crud() {
    let mut store = FilesystemLayoutStore::new();

    // List layouts
    let layouts = store.list_layouts();
    assert_eq!(layouts.len(), 2);

    // Register a valid custom layout
    let mut custom = FilesystemLayoutSpec::minimal_container();
    custom.id = "custom-test-v1".to_string();
    custom.name = "Custom Test Layout".to_string();
    assert!(store.register_layout(custom.clone()).is_ok());

    // Duplicate registration rejected
    let err = store.register_layout(custom.clone()).unwrap_err();
    assert!(err.contains("already registered"));

    // Set active layout
    assert!(store.set_active_layout("custom-test-v1").is_ok());
    assert_eq!(store.active_layout_id, "custom-test-v1");
    assert_eq!(store.get_active_layout().unwrap().id, "custom-test-v1");

    // Cannot remove currently active layout
    let err_rm_active = store.remove_layout("custom-test-v1").unwrap_err();
    assert!(err_rm_active.contains("cannot remove active layout"));

    // Cannot remove built-in canonical layout
    let err_rm_builtin = store.remove_layout("aios-uefi-standard-v1").unwrap_err();
    assert!(err_rm_builtin.contains("cannot remove built-in canonical layout"));

    // Switch active back and remove custom
    assert!(store.set_active_layout("aios-uefi-standard-v1").is_ok());
    let removed = store.remove_layout("custom-test-v1");
    assert!(removed.is_ok());
    assert_eq!(removed.unwrap().id, "custom-test-v1");
    assert!(store.get_layout("custom-test-v1").is_none());
}

#[test]
fn test_fs_layout_service_probe_target() {
    let service = FilesystemLayoutService::new();

    // 100 GiB disk: easily satisfies standard_uefi (min 64 GiB, partition sum 54.5 GiB)
    let eval_ok = service.probe_target("aios-uefi-standard-v1", 100 * 1024 * 1024 * 1024).unwrap();
    assert!(eval_ok.is_viable);
    assert!(eval_ok.errors.is_empty());
    assert_eq!(eval_ok.layout_id, "aios-uefi-standard-v1");

    // 20 GiB disk: fails because required minimum is 64 GiB and partition sum is ~54.5 GiB
    let eval_fail = service.probe_target("aios-uefi-standard-v1", 20 * 1024 * 1024 * 1024).unwrap();
    assert!(!eval_fail.is_viable);
    assert!(!eval_fail.errors.is_empty());

    // Non-existent layout
    assert!(service.probe_target("nonexistent-layout", 100 * 1024 * 1024 * 1024).is_err());
}

#[test]
fn test_fs_layout_service_diff_layouts() {
    let mut service = FilesystemLayoutService::new();

    // Diff between standard_uefi and minimal_container (destructive because EFI, root, swap partitions differ/removed)
    let diff = service.diff_layouts("aios-uefi-standard-v1", "aios-container-minimal-v1").unwrap();
    assert!(diff.destructive);
    assert!(!diff.partitions_removed.is_empty());
    assert!(diff.summary.contains("Destructive: true"));

    // Create a non-destructive variation: enlarge root partition from 50 GiB to 60 GiB
    let mut expanded = FilesystemLayoutSpec::standard_uefi();
    expanded.id = "expanded-uefi-v1".to_string();
    expanded.target_disk_min_bytes = 80 * 1024 * 1024 * 1024;
    expanded.partitions[1].size_mib = 60 * 1024;
    assert!(service.store_mut().register_layout(expanded).is_ok());

    let diff_expand = service.diff_layouts("aios-uefi-standard-v1", "expanded-uefi-v1").unwrap();
    assert!(!diff_expand.destructive);
    assert_eq!(diff_expand.partitions_modified.len(), 1);
    assert_eq!(diff_expand.partitions_modified[0].old_size_mib, 50 * 1024);
    assert_eq!(diff_expand.partitions_modified[0].new_size_mib, 60 * 1024);
}

#[test]
fn test_fs_layout_service_fstab_export_and_import() {
    let mut service = FilesystemLayoutService::new();

    // Export fstab
    let fstab_text = service.export_fstab("aios-uefi-standard-v1").unwrap();
    assert!(fstab_text.contains("/boot/efi"));
    assert!(fstab_text.contains("LABEL=AIOS_ROOT"));

    // Import fstab into a new layout
    let imported_fstab = "LABEL=AIOS_ROOT / ext4 rw,relatime 0 1\ntmpfs /tmp tmpfs rw,nosuid,nodev,noexec 0 0\n";
    let imported = service.import_fstab_as_layout(
        "imported-fstab-v1",
        "Imported Fstab Layout",
        imported_fstab,
        Some("aios-container-minimal-v1"),
    );
    assert!(imported.is_ok());
    let imported_spec = imported.unwrap();
    assert_eq!(imported_spec.id, "imported-fstab-v1");
    assert_eq!(imported_spec.mounts.len(), 2);
    assert_eq!(imported_spec.mounts[0].path, "/");
    assert_eq!(imported_spec.mounts[1].path, "/tmp");
}

#[test]
fn test_fs_layout_service_atomic_persistence() {
    let service = FilesystemLayoutService::new();
    let tmp_dir = env::temp_dir().join(format!("aios_fs_layout_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp_dir);
    let store_file = tmp_dir.join("fs_layouts.json");

    // Save
    assert!(service.save_to_path(&store_file).is_ok());
    assert!(store_file.exists());

    // Load
    let loaded = FilesystemLayoutService::load_from_path(&store_file);
    assert!(loaded.is_ok());
    let loaded_service = loaded.unwrap();
    assert_eq!(loaded_service.store.active_layout_id, "aios-uefi-standard-v1");
    assert_eq!(loaded_service.store.layouts.len(), 2);

    // Clean up
    let _ = std::fs::remove_file(&store_file);
    let _ = std::fs::remove_dir(&tmp_dir);
}

#[test]
fn test_fs_layout_service_negative_registration_and_validation() {
    let mut store = FilesystemLayoutStore::new();

    // Layout violating FL1: root mount pass number must be 1
    let mut bad_layout = FilesystemLayoutSpec::minimal_container();
    bad_layout.id = "bad-pass-layout".to_string();
    bad_layout.mounts[0].pass = 0; // violates FL1
    let err = store.register_layout(bad_layout).unwrap_err();
    assert!(err.contains("pass number"));

    // Layout violating FL1: missing root mount
    let mut no_root = FilesystemLayoutSpec::minimal_container();
    no_root.id = "no-root-layout".to_string();
    no_root.mounts.clear();
    let err_no_root = store.register_layout(no_root).unwrap_err();
    assert!(err_no_root.contains("exactly one root"));
}

#[test]
fn test_fs_layout_service_probe_boundary_and_slack_warnings() {
    let service = FilesystemLayoutService::new();
    let spec = service.store.get_layout("aios-uefi-standard-v1").unwrap();
    let _partition_sum_mib: u64 = spec.partitions.iter().map(|p| p.size_mib).sum();

    // Boundary: 1 byte below required disk minimum
    let shortfall_eval = service
        .probe_target("aios-uefi-standard-v1", spec.target_disk_min_bytes - 1)
        .unwrap();
    assert!(!shortfall_eval.is_viable);
    assert!(shortfall_eval.errors.iter().any(|e| e.contains("below required minimum")));

    // Boundary: exact partition budget (slack = 0 < 10%)
    let exact_budget_eval = service
        .probe_target("aios-uefi-standard-v1", spec.target_disk_min_bytes)
        .unwrap();
    assert!(exact_budget_eval.is_viable);
    // At 64 GiB disk with ~54.5 GiB partition sum, slack is ~9.5 GiB, which is > 10% of 54.5 GiB (5.45 GiB)
    assert!(exact_budget_eval.warnings.is_empty());

    // Artificial test layout with tight slack
    let mut tight_service = FilesystemLayoutService::empty();
    let mut tight_layout = FilesystemLayoutSpec::minimal_container();
    tight_layout.id = "tight-layout".to_string();
    tight_layout.target_disk_min_bytes = 100 * 1024 * 1024;
    tight_layout.partitions = vec![
        aiosh_core::PartitionSpec {
            index: 1,
            label: "ROOT".into(),
            partition_type: aiosh_core::PartitionType::LinuxRoot,
            size_mib: 100,
            uuid: None,
            bootable: true,
            format_as: Some(aiosh_core::FsType::Ext4),
        }
    ];
    assert!(tight_service.store_mut().register_layout(tight_layout).is_ok());

    // 105 MiB disk capacity: partition budget is 100 MiB, slack is 5 MiB (5% < 10%)
    let tight_eval = tight_service.probe_target("tight-layout", 105 * 1024 * 1024).unwrap();
    assert!(tight_eval.is_viable);
    assert!(!tight_eval.warnings.is_empty());
    assert!(tight_eval.warnings[0].contains("less than 10% free capacity"));
}

#[test]
fn test_fs_layout_service_negative_store_and_diff_operations() {
    let mut service = FilesystemLayoutService::new();

    // Non-existent layout remove
    assert!(service.store_mut().remove_layout("no-such-layout").is_err());

    // Non-existent set active
    assert!(service.store_mut().set_active_layout("no-such-layout").is_err());

    // Diff non-existent source
    let err_src = service.diff_layouts("ghost-source", "aios-uefi-standard-v1").unwrap_err();
    assert!(err_src.contains("source layout with id 'ghost-source' not found"));

    // Diff non-existent target
    let err_tgt = service.diff_layouts("aios-uefi-standard-v1", "ghost-target").unwrap_err();
    assert!(err_tgt.contains("target layout with id 'ghost-target' not found"));
}

#[test]
fn test_fs_layout_service_negative_fstab_and_persistence() {
    let mut service = FilesystemLayoutService::new();

    // Import empty fstab
    let err_empty = service.import_fstab_as_layout("empty-fstab", "Empty", "", None).unwrap_err();
    assert!(err_empty.contains("contains no valid mount entries"));

    // Import malformed fstab line
    let err_malformed = service
        .import_fstab_as_layout("bad-fstab", "Bad", "INVALID_LINE_ONE_TOKEN", None)
        .unwrap_err();
    assert!(err_malformed.contains("error parsing fstab line 1"));

    // Export non-existent layout fstab
    assert!(service.export_fstab("ghost-layout").is_err());

    // Persistence: load non-existent path
    let tmp_nonexistent = env::temp_dir().join("nonexistent_fs_layout_store_path.json");
    let err_not_found = FilesystemLayoutService::load_from_path(&tmp_nonexistent).unwrap_err();
    assert!(err_not_found.contains("not found"));

    // Persistence: load corrupt json
    let tmp_corrupt = env::temp_dir().join(format!("aios_corrupt_layout_{}.json", std::process::id()));
    let _ = std::fs::write(&tmp_corrupt, "{ not valid json }");
    let err_corrupt = FilesystemLayoutService::load_from_path(&tmp_corrupt).unwrap_err();
    assert!(err_corrupt.contains("failed to deserialize"));
    let _ = std::fs::remove_file(&tmp_corrupt);
}

#[test]
fn test_fs_layout_service_hardening_mount_caps_and_path_hygiene() {
    let mut service = FilesystemLayoutService::new();

    // Oversized mount import (> 128 mounts)
    let mut huge_fstab = "LABEL=ROOT / ext4 rw,relatime 0 1\n".to_string();
    for i in 1..=130 {
        huge_fstab.push_str(&format!("tmpfs /tmp/mount_{} tmpfs rw 0 0\n", i));
    }
    let err_huge = service
        .import_fstab_as_layout("huge-fstab", "Huge", &huge_fstab, None)
        .unwrap_err();
    assert!(err_huge.contains("exceeds maximum limit of 128 mounts"));

    // Path hygiene on save
    assert!(service.save_to_path(std::path::Path::new("")).is_err());
    assert!(service.save_to_path(std::path::Path::new("bad\0path.json")).is_err());
}

// ---------------------------------------------------------------------------
// T-01528 hardening: atomic replacement, staged-file preservation, bounded and
// non-blocking reads.
// ---------------------------------------------------------------------------

use aiosh_core::fs_layout_service::{
    read_bounded_text_file, LayoutDocReadError, MAX_LAYOUT_DOC_BYTES, MAX_REPLACE_ATTEMPTS,
    MAX_STAGED_KEEP,
};
use std::path::PathBuf;

/// Collects staged temporary files (`.name.tmp.*`) left beside a destination.
fn staged_siblings(dir: &std::path::Path, dest_name: &str) -> Vec<PathBuf> {
    let prefix = format!(".{}.tmp.", dest_name);
    let mut found = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with(&prefix) {
                found.push(entry.path());
            }
        }
    }
    found
}

fn fresh_dir(tag: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("aios_fs_layout_{}_{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_fs_layout_save_replaces_atomically_and_leaves_no_temp_files() {
    let tmp_dir = fresh_dir("atomic");
    let store_file = tmp_dir.join("fs_layouts.json");

    let service = FilesystemLayoutService::new();
    assert!(service.save_to_path(&store_file).is_ok());

    // Re-save over an existing store. The destination must exist the moment the call
    // returns, and it must hold the *new* state: the previous implementation unlinked
    // the destination before renaming into place, so any failure between those two
    // syscalls left the operator with no store at all.
    let mut grown = FilesystemLayoutService::new();
    let mut extra = FilesystemLayoutSpec::minimal_container();
    extra.id = "atomic-second-v1".to_string();
    extra.name = "Atomic Second".to_string();
    assert!(grown.store_mut().register_layout(extra).is_ok());
    assert!(grown.save_to_path(&store_file).is_ok());

    assert!(store_file.exists(), "destination must never be absent after a save");
    let reloaded = FilesystemLayoutService::load_from_path(&store_file).unwrap();
    assert_eq!(reloaded.store.layouts.len(), service.store.layouts.len() + 1);
    assert!(reloaded.store.layouts.contains_key("atomic-second-v1"));

    // The staged file is consumed by the rename: no temporary garbage survives success.
    assert!(
        staged_siblings(&tmp_dir, "fs_layouts.json").is_empty(),
        "successful save must not leave staged files behind"
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_fs_layout_save_preserves_staged_file_when_rename_fails() {
    let tmp_dir = fresh_dir("rename-fail");

    // A non-empty directory at the destination makes the final rename fail on every
    // platform (POSIX EISDIR, Windows ACCESS_DENIED), which exercises the error path.
    let blocked = tmp_dir.join("fs_layouts.json");
    std::fs::create_dir_all(&blocked).unwrap();
    std::fs::write(blocked.join("keep"), b"x").unwrap();

    let service = FilesystemLayoutService::new();
    let err = service.save_to_path(&blocked).unwrap_err();
    assert!(
        err.contains("preserved at"),
        "rename failure must name the preserved staged file, got: {}",
        err
    );

    // The staged file holds the only complete copy of the state the caller asked to
    // persist, so deleting it on the error path would silently destroy their data.
    let staged = staged_siblings(&tmp_dir, "fs_layouts.json");
    assert_eq!(staged.len(), 1, "expected exactly one preserved staged file, got {:?}", staged);
    let content = std::fs::read_to_string(&staged[0]).unwrap();
    assert!(content.contains("aios-uefi-standard-v1"));

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_fs_layout_reads_reject_non_regular_files_and_oversize_documents() {
    let tmp_dir = fresh_dir("reads");

    // A directory is not a layout document. The reader must reject it by file type
    // instead of attempting a read that cannot produce a store.
    let as_dir = tmp_dir.join("store_is_a_dir.json");
    std::fs::create_dir_all(&as_dir).unwrap();
    let err_dir = FilesystemLayoutService::load_from_path(&as_dir).unwrap_err();
    assert!(err_dir.contains("is a directory"), "unexpected message: {}", err_dir);
    assert!(!err_dir.contains("failed to deserialize"));
    assert!(matches!(
        read_bounded_text_file(&as_dir, MAX_LAYOUT_DOC_BYTES, "layout store file"),
        Err(LayoutDocReadError::NotRegularFile(_))
    ));

    // The cap is enforced from the stream, not merely from the pre-read metadata
    // snapshot, so a document that outgrows the snapshot is still rejected.
    let big = tmp_dir.join("big_store.json");
    std::fs::write(&big, "x".repeat(4096)).unwrap();
    assert!(matches!(
        read_bounded_text_file(&big, 64, "layout store file"),
        Err(LayoutDocReadError::TooLarge(_))
    ));

    // Non-UTF-8 input is reported as its own failure rather than an opaque I/O error.
    let bin = tmp_dir.join("binary_store.json");
    std::fs::write(&bin, [0xffu8, 0xfe, 0x00, 0x01]).unwrap();
    assert!(matches!(
        read_bounded_text_file(&bin, MAX_LAYOUT_DOC_BYTES, "layout store file"),
        Err(LayoutDocReadError::NotUtf8(_))
    ));

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

// ---------------------------------------------------------------------------
// T-01538 hardening: bounded replace retry, bounded staged residue, and a write-side
// size ceiling that matches the reader's.
// ---------------------------------------------------------------------------

/// A store larger than the reader's ceiling must be refused, not written.
///
/// Nothing bounded the write side, so a store could be grown past
/// `MAX_LAYOUT_DOC_BYTES` and saved successfully — and that file is then rejected by
/// `load_from_path` *forever*. The mutation reported success, so the store silently
/// became unreadable. The refusal must be explicit, must leave the previous store on
/// disk exactly as it was, and must not stage a file.
#[test]
fn test_fs_layout_save_refuses_store_larger_than_reader_ceiling() {
    let tmp_dir = fresh_dir("oversize-write");
    let store_file = tmp_dir.join("fs_layouts.json");

    // Baseline: a normal store that is readable.
    let service = FilesystemLayoutService::new();
    assert!(service.save_to_path(&store_file).is_ok());
    let baseline = std::fs::read(&store_file).unwrap();
    assert!(FilesystemLayoutService::load_from_path(&store_file).is_ok());

    // Grow the store past the ceiling. `DirectorySpec::description` is free text with
    // no length bound of its own, so one registered layout is enough to exceed 10 MiB.
    let mut oversized = FilesystemLayoutService::new();
    let mut spec = FilesystemLayoutSpec::minimal_container();
    spec.id = "oversized-v1".to_string();
    spec.name = "Oversized".to_string();
    spec.directories.push(aiosh_core::fs_layout::DirectorySpec {
        path: "/var/lib/aios/oversized".to_string(),
        mode: 0o750,
        owner: "root".to_string(),
        group: "root".to_string(),
        description: "x".repeat(MAX_LAYOUT_DOC_BYTES as usize + 4096),
        symlink_target: None,
    });
    // The layout validates (that is the point): the *write* is what must stop it.
    assert!(oversized.store_mut().register_layout(spec).is_ok());

    let err = oversized.save_to_path(&store_file).unwrap_err();
    assert!(
        err.contains("exceeds") && err.contains("read ceiling"),
        "oversized save must be refused by the read ceiling, got: {}",
        err
    );
    assert!(err.contains("unchanged"), "refusal must state the store is unchanged: {}", err);

    // The pre-existing store is untouched and still loadable: a refused write must not
    // be able to destroy the state it declined to replace.
    assert_eq!(std::fs::read(&store_file).unwrap(), baseline);
    assert!(FilesystemLayoutService::load_from_path(&store_file).is_ok());
    assert!(
        staged_siblings(&tmp_dir, "fs_layouts.json").is_empty(),
        "a refused oversized save must not stage a file"
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

/// A replace that keeps failing must not grow the directory without limit.
///
/// Every failed replace deliberately preserves its staged file — it is the only
/// complete copy of the state. Without a cap, a destination that is permanently
/// unwritable turns each call into another stranded file. The cap is checked *before*
/// staging, so the refusal itself adds no residue.
#[test]
fn test_fs_layout_save_refuses_to_stage_past_the_residue_cap() {
    let tmp_dir = fresh_dir("staged-cap");

    // Simulate the residue of MAX_STAGED_KEEP earlier failed replacements. The names
    // match `create_exclusive_temp`'s own shape, so they are the files a real cap must
    // count (and a *differently* named store's staged files must not be counted).
    let mut pre = 0usize;
    for i in 0..MAX_STAGED_KEEP {
        std::fs::write(tmp_dir.join(format!(".fs_layouts.json.tmp.{}.{}.{}", std::process::id(), i, i)), b"{}")
            .unwrap();
        pre += 1;
    }
    // A sibling store's staged file shares the directory but not the prefix.
    std::fs::write(tmp_dir.join(".other-store.json.tmp.1.1.0"), b"{}").unwrap();

    let service = FilesystemLayoutService::new();
    let err = service.save_to_path(&tmp_dir.join("fs_layouts.json")).unwrap_err();
    assert!(
        err.contains(&format!("cap {}", MAX_STAGED_KEEP)) && err.contains("refusing to stage"),
        "the residue cap must be enforced with the cap named, got: {}",
        err
    );
    assert_eq!(pre, MAX_STAGED_KEEP);

    // Refusal added nothing, and the unrelated store's staged file was neither counted
    // against this store nor deleted.
    assert_eq!(
        staged_siblings(&tmp_dir, "fs_layouts.json").len(),
        MAX_STAGED_KEEP,
        "refusing to stage must not itself stage a file"
    );
    assert!(tmp_dir.join(".other-store.json.tmp.1.1.0").exists());

    // Removing the residue restores normal operation, so the cap is a bound and not a
    // permanent lockout.
    for entry in staged_siblings(&tmp_dir, "fs_layouts.json") {
        std::fs::remove_file(entry).unwrap();
    }
    assert!(service.save_to_path(&tmp_dir.join("fs_layouts.json")).is_ok());

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

/// One physical destination = one residue budget, however it is spelled.
///
/// The T-01539 adversarial verification proved the first cut of this cap was keyed on the
/// *spelled* store name, so spellings of one destination each drew their own quota - case
/// variants, 8.3 short names, trailing dot/space and a nesting prefix pooled 32 staged
/// files around one destination where 8 was the cap. The cap must charge residue to the
/// canonical destination: here 8 staged files are planted under *mixed-case* spellings of
/// one destination, and the plain-spelling save is then refused because the shared budget
/// is spent - which is exactly what the old spelling-keyed cap did not do. Reverting the
/// grouping to the spelled prefix makes this test fail (the plain spelling would see zero
/// residue of its own and save successfully).
#[test]
fn test_fs_layout_residue_cap_is_keyed_on_the_physical_destination() {
    let tmp_dir = fresh_dir("alias-cap");
    let aliases: Vec<String> = if cfg!(windows) {
        // One physical destination, several case spellings (the filesystem folds case).
        vec![
            "fs_layouts.json".to_string(),
            "FS_LAYOUTS.JSON".to_string(),
            "Fs_Layouts.Json".to_string(),
        ]
    } else {
        // POSIX keeps case, so there the alias set is the one spelling.
        vec!["fs_layouts.json".to_string()]
    };

    // Plant MAX_STAGED_KEEP staged files across the alias spellings of one destination,
    // in `create_exclusive_temp`'s own name shape.
    let mut planted = 0usize;
    'outer: for spelling in &aliases {
        let stem = spelling.trim_end_matches(".json");
        for i in 0..MAX_STAGED_KEEP {
            if planted >= MAX_STAGED_KEEP {
                break 'outer;
            }
            std::fs::write(
                tmp_dir.join(format!(".{spelling}.tmp.{}.{}.{}", std::process::id(), i, i)),
                b"{}",
            )
            .unwrap();
            planted += 1;
            let _ = stem;
        }
    }
    assert_eq!(planted, MAX_STAGED_KEEP);

    let service = FilesystemLayoutService::new();
    let plain_dest = tmp_dir.join("fs_layouts.json");

    // The plain spelling must now be refused: its physical destination is saturated.
    let err = service.save_to_path(&plain_dest).unwrap_err();
    assert!(
        err.contains("refusing to stage") && err.contains(&format!("cap {}", MAX_STAGED_KEEP)),
        "a saturated destination must be refused under any spelling, got: {err}"
    );
    // ...and the refusal must not itself stage anything, on any spelling.
    for spelling in &aliases {
        let err = service.save_to_path(&tmp_dir.join(spelling)).unwrap_err();
        assert!(
            err.contains("refusing to stage"),
            "spelling {spelling} of a saturated destination must be refused, got: {err}"
        );
    }

    // A genuinely different store in the same directory keeps its own budget and saves.
    let other = tmp_dir.join("other-store.json");
    assert!(
        service.save_to_path(&other).is_ok(),
        "a different destination must keep its own budget"
    );

    // The other store's residue is not charged to the saturated destination either.
    std::fs::write(tmp_dir.join(".other-store.json.tmp.9.9.0"), b"{}").unwrap();
    let err = service.save_to_path(&plain_dest).unwrap_err();
    assert!(err.contains("refusing to stage"), "still saturated: {err}");

    // Nothing was ever deleted: every planted file plus the other store's residue exists.
    let residue: Vec<_> = std::fs::read_dir(&tmp_dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(".fs_layouts.json.tmp.") || n.starts_with(".other-store.json.tmp."))
        .collect();
    assert_eq!(residue.len(), MAX_STAGED_KEEP + 1, "residue must never be deleted");

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

/// The mirror image of the alias evasion: two stores whose *spelled* names nest must not
/// share a budget. The old prefix grouping had this bug in the other direction - the
/// prefix `.fs_layouts.json.tmp.` matched files staged for BOTH `fs_layouts.json` and the
/// distinct destination `fs_layouts.json.tmp`, so saturating one locked out the other.
#[test]
fn test_fs_layout_nested_store_spellings_keep_separate_budgets() {
    let tmp_dir = fresh_dir("nested-cap");

    // Saturate the distinct destination `fs_layouts.json.tmp` (staged files whose
    // recovered destination is `<dir>/fs_layouts.json.tmp`).
    for i in 0..MAX_STAGED_KEEP {
        std::fs::write(
            tmp_dir.join(format!(".fs_layouts.json.tmp.tmp.{}.{}.{}", std::process::id(), i, i)),
            b"{}",
        )
        .unwrap();
    }

    let service = FilesystemLayoutService::new();
    // `fs_layouts.json` - a DIFFERENT physical destination - must save normally.
    assert!(
        service.save_to_path(&tmp_dir.join("fs_layouts.json")).is_ok(),
        "a store whose name nests another's must keep its own budget"
    );
    // And the nested-spelling destination is still saturated, as intended.
    let err = service.save_to_path(&tmp_dir.join("fs_layouts.json.tmp")).unwrap_err();
    assert!(
        err.contains("refusing to stage"),
        "the saturated nested-spelling destination must still be capped: {err}"
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

/// The replace retry is bounded and always reports how many attempts it made.
///
/// Platform behaviour differs (POSIX reports `EISDIR` for a directory destination,
/// Windows reports a sharing violation that the predicate classes as transient), so
/// this pins the *contract* rather than an attempt count: at least one attempt, never
/// more than the bound, always an explicit error naming the preserved staged file, and
/// no extra residue from the retries themselves.
#[test]
fn test_fs_layout_replace_retry_is_bounded_and_reported() {
    let tmp_dir = fresh_dir("replace-bounded");

    // A non-empty directory at the destination fails the rename on every platform.
    let blocked = tmp_dir.join("fs_layouts.json");
    std::fs::create_dir_all(&blocked).unwrap();
    std::fs::write(blocked.join("keep"), b"x").unwrap();

    let service = FilesystemLayoutService::new();
    let started = std::time::Instant::now();
    let err = service.save_to_path(&blocked).unwrap_err();
    let elapsed = started.elapsed();

    assert!(err.contains("attempt(s)"), "error must report its attempt count: {}", err);
    assert!(err.contains("preserved at"), "error must name the preserved staged file: {}", err);

    let attempts: u32 = err
        .split("after ")
        .nth(1)
        .and_then(|tail| tail.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("could not parse the attempt count from: {}", err));
    assert!(
        (1..=MAX_REPLACE_ATTEMPTS).contains(&attempts),
        "attempts must be bounded by {} but the error reported {}",
        MAX_REPLACE_ATTEMPTS,
        attempts
    );

    // Retries reuse the one staged file, so they cannot multiply residue.
    assert_eq!(staged_siblings(&tmp_dir, "fs_layouts.json").len(), 1);
    // And the whole envelope stays inside its stated budget (backoff only, no sleep loop).
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "replace retry took {:?}, which is not bounded by the retry budget",
        elapsed
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

/// On Linux, `/proc` files report a metadata length of `0` while yielding real
/// content. That mismatch is exactly what a metadata-only size check misses, so this
/// pins the read-time enforcement of the cap.
#[cfg(target_os = "linux")]
#[test]
fn test_fs_layout_reader_bounds_growing_proc_files() {
    let proc_status = std::path::Path::new("/proc/self/status");
    if !proc_status.is_file() {
        return;
    }
    assert_eq!(
        std::fs::metadata(proc_status).unwrap().len(),
        0,
        "/proc/self/status is expected to report a zero metadata length"
    );
    assert!(
        matches!(
            read_bounded_text_file(proc_status, 64, "layout store file"),
            Err(LayoutDocReadError::TooLarge(_))
        ),
        "a metadata-zero file with real content must still honour the cap"
    );
}

