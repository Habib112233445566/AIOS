//! Filesystem Layout Core Service (CS1..CS5).
//!
//! Provides the runtime `FilesystemLayoutService` coordinator, layout store management,
//! target disk feasibility probing, differential comparison, fstab synthesis, and atomic persistence.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::fs_layout::{
    validate_filesystem_layout, DirectorySpec, FilesystemLayoutSpec, FsType, MountPointSpec,
    PartitionSpec, PartitionType,
};

/// Hard ceiling for any layout store or layout spec document accepted from disk (10 MiB).
pub const MAX_LAYOUT_DOC_BYTES: u64 = 10 * 1024 * 1024;

/// Bounded number of attempts used when creating an exclusive temporary file.
const MAX_TEMP_ATTEMPTS: u32 = 8;

/// Why a layout document could not be read from disk.
///
/// Distinct variants exist so callers can map each failure onto its own explicit
/// audit classification instead of collapsing every disk problem into one opaque
/// "read failed" (see ADR-0035 §F-2: failures must be explicit and auditable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutDocReadError {
    /// No file exists at the requested path.
    NotFound(String),
    /// The path exists but is a directory, FIFO, device, or socket.
    NotRegularFile(String),
    /// The document is larger than the configured cap.
    TooLarge(String),
    /// The bytes are not valid UTF-8.
    NotUtf8(String),
    /// Any other underlying I/O failure.
    Io(String),
}

impl LayoutDocReadError {
    /// Human-readable diagnostic for this failure.
    pub fn message(&self) -> &str {
        match self {
            LayoutDocReadError::NotFound(m)
            | LayoutDocReadError::NotRegularFile(m)
            | LayoutDocReadError::TooLarge(m)
            | LayoutDocReadError::NotUtf8(m)
            | LayoutDocReadError::Io(m) => m.as_str(),
        }
    }
}

impl std::fmt::Display for LayoutDocReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

/// Reads a text document from a file that may be named by an untrusted caller.
///
/// `std::fs::read_to_string` is not safe on its own here. A path supplied through
/// `--spec`, `--store`, or `--fstab` can name something that is not a regular file:
///
/// * a **FIFO** reports a metadata length of `0`, so a length-only cap passes, and
///   the read then blocks forever waiting for a writer — an unauthenticated hang;
/// * a **character device** such as `/dev/zero` passes the same check and then never
///   reaches EOF, streaming until the process exhausts memory;
/// * a regular file can grow *after* the metadata check, so a cap applied only to
///   `metadata().len()` is a check-then-use race.
///
/// This helper therefore stats first and refuses anything that is not a regular
/// file, then enforces the cap *during* the read by capping the stream at
/// `max_bytes + 1` and rejecting any document that reaches the extra byte. Reads
/// are consequently both bounded and non-blocking no matter what the path names.
/// Symlinks to regular files are permitted: only reads happen here, and the write
/// path never follows a pre-existing link.
pub fn read_bounded_text_file(
    path: &Path,
    max_bytes: u64,
    label: &str,
) -> Result<String, LayoutDocReadError> {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(LayoutDocReadError::NotFound(format!(
                "{} not found: '{}'",
                label,
                path.display()
            )));
        }
        Err(e) => {
            return Err(LayoutDocReadError::Io(format!(
                "failed to inspect {} '{}': {}",
                label,
                path.display(),
                e
            )));
        }
    };

    if !meta.file_type().is_file() {
        let kind = if meta.file_type().is_dir() {
            "a directory".to_string()
        } else {
            // Not a directory and not a regular file: FIFO, device, or socket.
            "not a regular file (FIFO, device, or socket)".to_string()
        };
        return Err(LayoutDocReadError::NotRegularFile(format!(
            "{} '{}' is {} and cannot be read",
            label,
            path.display(),
            kind
        )));
    }

    if meta.len() > max_bytes {
        return Err(LayoutDocReadError::TooLarge(format!(
            "{} '{}' exceeds {} MiB security size limit",
            label,
            path.display(),
            max_bytes / (1024 * 1024)
        )));
    }

    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return Err(LayoutDocReadError::Io(format!(
                "failed to read {} '{}': {}",
                label,
                path.display(),
                e
            )));
        }
    };

    let mut buf = Vec::new();
    if let Err(e) = file.take(max_bytes + 1).read_to_end(&mut buf) {
        return Err(LayoutDocReadError::Io(format!(
            "failed to read {} '{}': {}",
            label,
            path.display(),
            e
        )));
    }

    if buf.len() as u64 > max_bytes {
        return Err(LayoutDocReadError::TooLarge(format!(
            "{} '{}' exceeds {} MiB security size limit",
            label,
            path.display(),
            max_bytes / (1024 * 1024)
        )));
    }

    String::from_utf8(buf).map_err(|_| {
        LayoutDocReadError::NotUtf8(format!("{} '{}' is not valid UTF-8", label, path.display()))
    })
}

/// Creates a temporary file beside `dest` for atomic replacement.
///
/// The file is opened with `create_new` (`O_CREAT | O_EXCL`), which means the
/// kernel creates it or fails — it is never opened through a pre-existing path.
/// That closes the symlink attack on the old predictable `<store>.tmp.<pid>` name,
/// where an attacker with write access to the store directory could pre-plant a
/// symlink and have the CLI write attacker-chosen content with the CLI's own
/// privileges. A pre-planted name can now only cost one attempt, so collisions are
/// retried a bounded number of times with a fresh unpredictable suffix.
fn create_exclusive_temp(dest: &Path) -> Result<(PathBuf, fs::File), String> {
    let dir = match dest.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let stem = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "layout-store".to_string());
    let pid = std::process::id();

    let mut last_err = String::new();
    for attempt in 0..MAX_TEMP_ATTEMPTS {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp_path = dir.join(format!(".{}.tmp.{}.{}.{}", stem, pid, nanos, attempt));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
        {
            Ok(f) => return Ok((tmp_path, f)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                last_err = e.to_string();
                continue;
            }
            Err(e) => {
                return Err(format!(
                    "failed to create temporary store file in '{}': {}",
                    dir.display(),
                    e
                ));
            }
        }
    }

    Err(format!(
        "failed to create a unique temporary store file in '{}' after {} attempts: {}",
        dir.display(),
        MAX_TEMP_ATTEMPTS,
        last_err
    ))
}


/// Target block device feasibility evaluation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetEvaluation {
    /// Identifier of the evaluated layout.
    pub layout_id: String,
    /// Target disk capacity in bytes.
    pub target_disk_bytes: u64,
    /// Minimum disk capacity required by layout specification in bytes.
    pub required_disk_bytes: u64,
    /// Sum of all partition capacities in bytes.
    pub partition_budget_bytes: u64,
    /// Whether target device can safely accommodate this layout.
    pub is_viable: bool,
    /// Non-fatal diagnostic warnings.
    pub warnings: Vec<String>,
    /// Fatal constraint violation errors.
    pub errors: Vec<String>,
}

/// Itemized difference between two partition specifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionDiffItem {
    pub index: u32,
    pub old_label: String,
    pub new_label: String,
    pub old_size_mib: u64,
    pub new_size_mib: u64,
    pub old_type: PartitionType,
    pub new_type: PartitionType,
}

/// Itemized difference between two mount point specifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MountDiffItem {
    pub path: String,
    pub old_device: String,
    pub new_device: String,
    pub old_fs: FsType,
    pub new_fs: FsType,
    pub old_options: Vec<String>,
    pub new_options: Vec<String>,
}

/// Structural and safety differential comparison between two filesystem layouts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutDiff {
    pub source_layout_id: String,
    pub target_layout_id: String,
    pub partitions_added: Vec<PartitionSpec>,
    pub partitions_removed: Vec<PartitionSpec>,
    pub partitions_modified: Vec<PartitionDiffItem>,
    pub mounts_added: Vec<MountPointSpec>,
    pub mounts_removed: Vec<MountPointSpec>,
    pub mounts_modified: Vec<MountDiffItem>,
    pub directories_added: Vec<DirectorySpec>,
    pub directories_removed: Vec<DirectorySpec>,
    /// Whether the delta involves destructive changes (partition removal, shrinking, or fs reformat).
    pub destructive: bool,
    /// Human-readable summary of the diff.
    pub summary: String,
}

/// Registry and store for managing filesystem layout specifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemLayoutStore {
    /// Active layout identifier currently in effect.
    pub active_layout_id: String,
    /// Map of layout identifier to layout specification.
    pub layouts: BTreeMap<String, FilesystemLayoutSpec>,
}

impl Default for FilesystemLayoutStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemLayoutStore {
    /// Initializes a store pre-seeded with canonical UEFI and container layouts.
    pub fn new() -> Self {
        let mut layouts = BTreeMap::new();
        let uefi = FilesystemLayoutSpec::standard_uefi();
        let container = FilesystemLayoutSpec::minimal_container();
        layouts.insert(uefi.id.clone(), uefi);
        layouts.insert(container.id.clone(), container);
        Self {
            active_layout_id: "aios-uefi-standard-v1".to_string(),
            layouts,
        }
    }

    /// Initializes an empty store for testing.
    pub fn empty() -> Self {
        Self {
            active_layout_id: String::new(),
            layouts: BTreeMap::new(),
        }
    }

    /// Registers a new layout spec after validating invariants.
    pub fn register_layout(&mut self, spec: FilesystemLayoutSpec) -> Result<(), String> {
        validate_filesystem_layout(&spec)?;
        if self.layouts.contains_key(&spec.id) {
            return Err(format!("layout with id '{}' is already registered", spec.id));
        }
        if self.active_layout_id.is_empty() {
            self.active_layout_id = spec.id.clone();
        }
        self.layouts.insert(spec.id.clone(), spec);
        Ok(())
    }

    /// Retrieves an immutable reference to a layout by ID.
    pub fn get_layout(&self, id: &str) -> Option<&FilesystemLayoutSpec> {
        self.layouts.get(id)
    }

    /// Lists all registered layout specs sorted by ID.
    pub fn list_layouts(&self) -> Vec<&FilesystemLayoutSpec> {
        self.layouts.values().collect()
    }

    /// Removes a layout spec from the store by ID.
    pub fn remove_layout(&mut self, id: &str) -> Result<FilesystemLayoutSpec, String> {
        if id == self.active_layout_id {
            return Err(format!(
                "cannot remove active layout '{}'; switch active layout first",
                id
            ));
        }
        if id == "aios-uefi-standard-v1" || id == "aios-container-minimal-v1" {
            return Err(format!(
                "cannot remove built-in canonical layout '{}'",
                id
            ));
        }
        self.layouts
            .remove(id)
            .ok_or_else(|| format!("layout with id '{}' not found in store", id))
    }

    /// Returns the currently active layout spec.
    pub fn get_active_layout(&self) -> Result<&FilesystemLayoutSpec, String> {
        if self.active_layout_id.is_empty() {
            return Err("no active layout is currently set".into());
        }
        self.layouts
            .get(&self.active_layout_id)
            .ok_or_else(|| format!("active layout '{}' not found in store", self.active_layout_id))
    }

    /// Switches the active layout pointer to the given layout ID.
    pub fn set_active_layout(&mut self, id: &str) -> Result<(), String> {
        if !self.layouts.contains_key(id) {
            return Err(format!("layout with id '{}' not found in store", id));
        }
        self.active_layout_id = id.to_string();
        Ok(())
    }
}

/// Central coordinator for filesystem layout operations, target probing, diffing, and persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemLayoutService {
    pub store: FilesystemLayoutStore,
}

impl Default for FilesystemLayoutService {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemLayoutService {
    /// Initializes the service with default canonical presets.
    pub fn new() -> Self {
        Self {
            store: FilesystemLayoutStore::new(),
        }
    }

    /// Initializes an empty service.
    pub fn empty() -> Self {
        Self {
            store: FilesystemLayoutStore::empty(),
        }
    }

    pub fn store(&self) -> &FilesystemLayoutStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut FilesystemLayoutStore {
        &mut self.store
    }

    /// Evaluates target disk capacity and feasibility against a registered layout.
    pub fn probe_target(&self, layout_id: &str, target_disk_bytes: u64) -> Result<TargetEvaluation, String> {
        let spec = self
            .store
            .get_layout(layout_id)
            .ok_or_else(|| format!("layout with id '{}' not found in store", layout_id))?;

        let required_disk_bytes = spec.target_disk_min_bytes;
        let partition_sum_mib: u64 = spec.partitions.iter().fold(0u64, |acc, p| acc.saturating_add(p.size_mib));
        let partition_budget_bytes = partition_sum_mib.saturating_mul(1024 * 1024);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if target_disk_bytes < required_disk_bytes {
            errors.push(format!(
                "target disk capacity ({} B) is below required minimum ({} B)",
                target_disk_bytes, required_disk_bytes
            ));
        }

        if target_disk_bytes < partition_budget_bytes {
            errors.push(format!(
                "target disk capacity ({} B) cannot fit total partition allocations ({} B)",
                target_disk_bytes, partition_budget_bytes
            ));
        }

        if target_disk_bytes >= partition_budget_bytes {
            let free_slack = target_disk_bytes - partition_budget_bytes;
            if free_slack < (partition_budget_bytes / 10) {
                warnings.push("target disk has less than 10% free capacity after partition allocation".to_string());
            }
        }

        let is_viable = errors.is_empty();

        Ok(TargetEvaluation {
            layout_id: layout_id.to_string(),
            target_disk_bytes,
            required_disk_bytes,
            partition_budget_bytes,
            is_viable,
            warnings,
            errors,
        })
    }

    /// Computes differential comparison between source and target layouts.
    pub fn diff_layouts(&self, source_id: &str, target_id: &str) -> Result<LayoutDiff, String> {
        let source = self
            .store
            .get_layout(source_id)
            .ok_or_else(|| format!("source layout with id '{}' not found in store", source_id))?;
        let target = self
            .store
            .get_layout(target_id)
            .ok_or_else(|| format!("target layout with id '{}' not found in store", target_id))?;

        let mut destructive = false;

        // Partition diffs
        let mut partitions_added = Vec::new();
        let mut partitions_removed = Vec::new();
        let mut partitions_modified = Vec::new();

        let source_parts_by_idx: BTreeMap<u32, &PartitionSpec> =
            source.partitions.iter().map(|p| (p.index, p)).collect();
        let target_parts_by_idx: BTreeMap<u32, &PartitionSpec> =
            target.partitions.iter().map(|p| (p.index, p)).collect();

        for (idx, target_part) in &target_parts_by_idx {
            match source_parts_by_idx.get(idx) {
                None => {
                    partitions_added.push((*target_part).clone());
                }
                Some(source_part) => {
                    if source_part.label != target_part.label
                        || source_part.size_mib != target_part.size_mib
                        || source_part.partition_type != target_part.partition_type
                    {
                        if target_part.size_mib < source_part.size_mib {
                            destructive = true;
                        }
                        partitions_modified.push(PartitionDiffItem {
                            index: *idx,
                            old_label: source_part.label.clone(),
                            new_label: target_part.label.clone(),
                            old_size_mib: source_part.size_mib,
                            new_size_mib: target_part.size_mib,
                            old_type: source_part.partition_type.clone(),
                            new_type: target_part.partition_type.clone(),
                        });
                    }
                }
            }
        }

        for (idx, source_part) in &source_parts_by_idx {
            if !target_parts_by_idx.contains_key(idx) {
                destructive = true;
                partitions_removed.push((*source_part).clone());
            }
        }

        // Mount diffs
        let mut mounts_added = Vec::new();
        let mut mounts_removed = Vec::new();
        let mut mounts_modified = Vec::new();

        let source_mounts_by_path: BTreeMap<&str, &MountPointSpec> =
            source.mounts.iter().map(|m| (m.path.as_str(), m)).collect();
        let target_mounts_by_path: BTreeMap<&str, &MountPointSpec> =
            target.mounts.iter().map(|m| (m.path.as_str(), m)).collect();

        for (path, target_m) in &target_mounts_by_path {
            match source_mounts_by_path.get(path) {
                None => {
                    mounts_added.push((*target_m).clone());
                }
                Some(source_m) => {
                    if source_m.device != target_m.device
                        || source_m.fs_type != target_m.fs_type
                        || source_m.options != target_m.options
                    {
                        if source_m.fs_type != target_m.fs_type {
                            destructive = true;
                        }
                        if *path == "/" && source_m.device != target_m.device {
                            destructive = true;
                        }
                        mounts_modified.push(MountDiffItem {
                            path: path.to_string(),
                            old_device: source_m.device.clone(),
                            new_device: target_m.device.clone(),
                            old_fs: source_m.fs_type.clone(),
                            new_fs: target_m.fs_type.clone(),
                            old_options: source_m.options.clone(),
                            new_options: target_m.options.clone(),
                        });
                    }
                }
            }
        }

        for (path, source_m) in &source_mounts_by_path {
            if !target_mounts_by_path.contains_key(path) {
                if *path == "/" || *path == "/boot/efi" {
                    destructive = true;
                }
                mounts_removed.push((*source_m).clone());
            }
        }

        // Directory diffs
        let mut directories_added = Vec::new();
        let mut directories_removed = Vec::new();

        let source_dirs_by_path: BTreeMap<&str, &DirectorySpec> =
            source.directories.iter().map(|d| (d.path.as_str(), d)).collect();
        let target_dirs_by_path: BTreeMap<&str, &DirectorySpec> =
            target.directories.iter().map(|d| (d.path.as_str(), d)).collect();

        for (path, target_d) in &target_dirs_by_path {
            if !source_dirs_by_path.contains_key(path) {
                directories_added.push((*target_d).clone());
            }
        }

        for (path, source_d) in &source_dirs_by_path {
            if !target_dirs_by_path.contains_key(path) {
                directories_removed.push((*source_d).clone());
            }
        }

        let summary = format!(
            "Delta from '{}' to '{}': +{} -{} ~{} partitions; +{} -{} ~{} mounts; +{} -{} directories. Destructive: {}",
            source_id,
            target_id,
            partitions_added.len(),
            partitions_removed.len(),
            partitions_modified.len(),
            mounts_added.len(),
            mounts_removed.len(),
            mounts_modified.len(),
            directories_added.len(),
            directories_removed.len(),
            destructive
        );

        Ok(LayoutDiff {
            source_layout_id: source_id.to_string(),
            target_layout_id: target_id.to_string(),
            partitions_added,
            partitions_removed,
            partitions_modified,
            mounts_added,
            mounts_removed,
            mounts_modified,
            directories_added,
            directories_removed,
            destructive,
            summary,
        })
    }

    /// Synthesizes `/etc/fstab` text for the specified layout.
    pub fn export_fstab(&self, layout_id: &str) -> Result<String, String> {
        let spec = self
            .store
            .get_layout(layout_id)
            .ok_or_else(|| format!("layout with id '{}' not found in store", layout_id))?;
        Ok(spec.generate_fstab())
    }

    /// Imports external `/etc/fstab` content into a new layout profile.
    pub fn import_fstab_as_layout(
        &mut self,
        id: &str,
        name: &str,
        fstab_content: &str,
        base_layout_id: Option<&str>,
    ) -> Result<FilesystemLayoutSpec, String> {
        let mut parsed_mounts = Vec::new();
        for (line_no, line) in fstab_content.lines().enumerate() {
            match MountPointSpec::parse_fstab_line(line) {
                Ok(Some(mount)) => parsed_mounts.push(mount),
                Ok(None) => {}
                Err(err) => {
                    return Err(format!("error parsing fstab line {}: {}", line_no + 1, err));
                }
            }
        }

        if parsed_mounts.is_empty() {
            return Err("fstab content contains no valid mount entries".into());
        }
        if parsed_mounts.len() > 128 {
            return Err(format!("imported fstab exceeds maximum limit of 128 mounts (found {})", parsed_mounts.len()));
        }

        let base_spec = if let Some(base_id) = base_layout_id {
            self.store
                .get_layout(base_id)
                .ok_or_else(|| format!("base layout with id '{}' not found in store", base_id))?
                .clone()
        } else {
            self.store.get_active_layout()?.clone()
        };

        let new_spec = FilesystemLayoutSpec {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("Layout imported from fstab content based on {}", base_spec.id),
            target_disk_min_bytes: base_spec.target_disk_min_bytes,
            partitions: base_spec.partitions,
            mounts: parsed_mounts,
            directories: base_spec.directories,
            created_at: "2026-09-16T00:00:00Z".to_string(),
        };

        self.store.register_layout(new_spec.clone())?;
        Ok(new_spec)
    }

    /// Atomically persists store state to a JSON file.
    ///
    /// The write is staged into an exclusively-created temporary file in the
    /// destination directory, flushed to stable storage, and then moved over the
    /// destination with a single rename. Three properties follow, all of which the
    /// previous implementation lacked:
    ///
    /// 1. the destination is **never unlinked first**, so there is no window in
    ///    which the store does not exist — a crash mid-save leaves either the old
    ///    complete store or the new one, never nothing;
    /// 2. the staged bytes are **fsync'd before the rename**, so a power loss after
    ///    success cannot leave a renamed-but-empty file;
    /// 3. a failed rename **preserves** the staged file and names it in the error,
    ///    because that file is the only complete copy of the state we were asked to
    ///    persist — silently deleting it would destroy the caller's data.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let path_str = path.to_string_lossy();
        if path_str.is_empty() || path_str.contains('\0') || path_str.chars().any(|c| c.is_control()) {
            return Err("store path cannot be empty or contain control characters".into());
        }

        let json_str = serde_json::to_string_pretty(&self.store)
            .map_err(|e| format!("failed to serialize filesystem layout store: {}", e))?;

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("failed to create parent directories for '{}': {}", path.display(), e))?;
            }
        }

        let (tmp_path, mut file) = create_exclusive_temp(path)?;

        if let Err(e) = file.write_all(json_str.as_bytes()).and_then(|()| file.sync_all()) {
            // Nothing usable was staged: drop the partial temporary file so a later
            // run cannot pick up a truncated store.
            drop(file);
            let _ = fs::remove_file(&tmp_path);
            return Err(format!(
                "failed to write temporary store file '{}': {}",
                tmp_path.display(),
                e
            ));
        }
        drop(file);

        // `fs::rename` already replaces an existing destination on both POSIX and
        // Windows, so the destination must not be removed beforehand: doing so would
        // open exactly the window this atomicity is supposed to prevent.
        if let Err(e) = fs::rename(&tmp_path, path) {
            return Err(format!(
                "failed to atomically replace store file '{}' from '{}': {} (the fully-written new store was preserved at '{}' for manual recovery)",
                path.display(),
                tmp_path.display(),
                e,
                tmp_path.display()
            ));
        }

        Ok(())
    }

    /// Loads and validates store state from a JSON file.
    ///
    /// Size and file-type enforcement is delegated to [`read_bounded_text_file`],
    /// which rejects directories, FIFOs, devices, and sockets, and applies the size
    /// cap during the read rather than only to the pre-read metadata snapshot.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = read_bounded_text_file(path, MAX_LAYOUT_DOC_BYTES, "layout store file")
            .map_err(|e| e.message().to_string())?;

        let store: FilesystemLayoutStore = serde_json::from_str(&content)
            .map_err(|e| format!("failed to deserialize layout store from '{}': {}", path.display(), e))?;

        for (id, spec) in &store.layouts {
            if &spec.id != id {
                return Err(format!(
                    "layout key '{}' does not match spec id '{}'",
                    id, spec.id
                ));
            }
            spec.validate()?;
        }

        if !store.active_layout_id.is_empty() && !store.layouts.contains_key(&store.active_layout_id) {
            return Err(format!(
                "active layout '{}' is not present in registered layouts",
                store.active_layout_id
            ));
        }

        Ok(Self { store })
    }
}
