//! AIOS core library — the Rust rewrite of the aiosh userspace stack.
//!
//! This crate replaces the previous dual-substrate implementation
//! (`code/aiosh-cli` in TypeScript + `code/aiosh-mcp` in Python) with a
//! single Rust implementation. All invariants that used to be enforced
//! by a TS↔Python cross-substrate test now live here as Rust unit and
//! integration tests:
//!
//!   - canonical JSON (byte-identical to the old Python `json.dumps(
//!     sort_keys=True, separators=(",",":"))` and TS `canonicalJson`)
//!   - SHA-256 hash-chained append-only audit ring (SQLite WAL)
//!   - Constitution rule-pack classifier (R-01..R-12)
//!   - PEP grant store and gate
//!   - Sprint-3 retention (checkpointed segment rotation + bloom filter)
//!   - Pillar-A pentest wrappers with safe defaults
//!   - Landlock + seccomp-bpf sandbox for `aiosh run`
//!   - Sprint-2 agent loop (Ollama + deterministic stub)
//!   - Task Ledger Control data model (T-00014 port of
//!     `tools/task_ledger.py`: atomic state, event log, no-skip law)

pub mod agent;
pub mod audit;
pub mod base_image;
pub mod base_image_config;
pub mod base_image_observability;
pub mod base_image_policy;
pub mod base_image_recovery;
pub mod base_image_service;
pub mod canonical;
pub mod ci;
pub mod ci_config;
pub mod classifier;
pub mod dispatch;
pub mod distro;
pub mod distro_config;
pub mod distro_observability;
pub mod distro_policy;
pub mod distro_recovery;
pub mod distro_service;
pub mod doc_index;
pub mod doc_index_config;
pub mod doc_index_service;
pub mod evidence;
pub mod evidence_config;
pub mod evidence_service;
pub mod fs_layout;
pub mod fs_layout_service;
pub mod fs_layout_service_key {
    //! Residue-grouping keys shared with `fs_layout_service` (kept beside `pep` so both
    //! the policy matcher and the residue cap resolve spelling aliases through one
    //! implementation of "what is this path's physical identity").
    pub use super::pep::{canonical_store_key, staged_file_destination};
}

pub mod handoff;

pub mod handoff_config;
pub mod handoff_service;
pub mod hardware;
pub mod hardware_config;
pub mod hardware_doc;
pub mod hardware_observability;
pub mod hardware_policy;
pub mod hardware_service;
pub mod kernel_module;
pub mod kernel_module_config;
pub mod kernel_module_doc;
pub mod kernel_module_observability;
pub mod kernel_module_policy;
pub mod kernel_module_recovery;
pub mod kernel_module_service;
pub mod ledger;
pub mod ledger_config;
pub mod package;
pub mod package_config;
pub mod package_observability;
pub mod package_policy;
pub mod package_recovery;
pub mod package_service;
pub mod pentest;
pub mod pep;
pub mod release;
pub mod release_config;
pub mod repo_health;
pub mod repo_health_config;
pub mod repo_health_service;
pub mod retention;
#[allow(dead_code)]
pub mod sandbox;
pub mod secrets;
pub mod secrets_config;
pub mod secrets_service;
pub mod service;
pub mod service_config;
pub mod service_observability;
pub mod service_policy;
pub mod service_recovery;
pub mod service_service;
pub mod session;
pub mod session_config;
pub mod session_observability;
pub mod session_policy;
pub mod session_recovery;
pub mod session_service;
pub mod task_service;
pub mod toolchain_config;
pub mod toolchain_service;
pub mod triage;
pub mod triage_config;
pub mod triage_service;
pub mod types;

pub use audit::{AuditRing, OpenOptions};
pub use classifier::classify;
pub use distro::{ArchTarget, CLibrary, DistroEvaluation, DistroFamily, DistroProfile, InitSystem};
pub use distro_service::DistroStore;
pub use fs_layout::{
    validate_directory_spec, validate_filesystem_layout, validate_mount_point,
    validate_partition_spec, DirectorySpec, FilesystemLayoutSpec, FsType, MountPointSpec,
    PartitionSpec, PartitionType,
};
pub use fs_layout_service::{
    FilesystemLayoutService, FilesystemLayoutStore, LayoutDiff, MountDiffItem, PartitionDiffItem,
    TargetEvaluation,
};
pub use handoff::{HandoffPriority, HandoffRecord, HandoffReport, HandoffStatus};
pub use handoff_service::HandoffStore;
pub use package::{
    PackageAction, PackageActionType, PackageDependency, PackageFormat, PackageQuery, PackageSpec,
    PackageState, PackageTransaction,
};
pub use package_recovery::{
    load_or_recover, recover_package_store_with_backup, validate_package_store,
    PackageValidationReport,
};
pub use package_service::{PackageStore, TransactionReport};
pub use pep::PepStore;
pub use service::{
    validate_service_name, validate_service_spec, validate_service_status, ServiceAction,
    ServiceDependency, ServiceDependencyType, ServiceHealth, ServiceQuery, ServiceRestartPolicy,
    ServiceSpec, ServiceStartupMode, ServiceState, ServiceStatus, ServiceType,
};
pub use service_config::ServiceConfig;
pub use service_observability::ServiceObservabilityReport;
pub use service_policy::{
    ServicePolicyMode, ServicePolicyVerdict, ServicePolicyViolation, ServiceSecurityPolicy,
};
pub use service_recovery::{ServiceRecoveryAction, ServiceValidationReport};
pub use service_service::{ServiceActionReport, ServiceStore};
pub use session::{
    transition_session_state, validate_session_id, validate_user_session_spec,
    validate_user_session_status, validate_username, SessionClass, SessionScope, SessionState,
    SessionType, UserSessionAction, UserSessionQuery, UserSessionSpec, UserSessionStatus,
    UserSessionStore,
};
pub use session_config::SessionConfig;
pub use session_observability::SessionObservabilityReport;
pub use session_policy::{
    SessionPolicyMode, SessionPolicyVerdict, SessionPolicyViolation, UserSessionSecurityPolicy,
};
pub use session_recovery::{SessionRecoveryAction, SessionValidationReport};
pub use session_service::{UserSessionActionReport, UserSessionService};
pub use hardware::{
    validate_hardware_inventory, DeviceClass, HardwareDevice, HardwareInventory,
    MAX_ATTRIBUTES_PER_DEVICE, MAX_ATTRIBUTE_KEY_LEN, MAX_ATTRIBUTE_VAL_LEN, MAX_DEVICES,
    MAX_DEVICE_ID_LEN, MAX_DEVICE_NAME_LEN, MAX_JSON_PAYLOAD_SIZE, MAX_PATH_LEN,
};
pub use hardware_config::HardwareConfig;
pub use hardware_policy::{
    HardwarePolicyMode, HardwarePolicyReport, HardwarePolicyViolation, HardwareSecurityPolicy,
};
pub use hardware_service::{HardwareScanOptions, HardwareService};
pub use types::GENESIS_HASH;



