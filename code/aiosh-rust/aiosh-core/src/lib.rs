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
pub mod capability;
pub mod capability_config;
pub mod capability_service;
pub mod capability_policy;
pub mod capability_observability;
pub mod capability_doc;
pub mod capability_recovery;
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
pub mod hardware_recovery;
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
pub mod network;
pub mod network_config;
pub mod network_doc;
pub mod network_observability;
pub mod network_policy;
pub mod network_recovery;
pub mod network_service;
pub mod package;
pub mod package_config;
pub mod package_observability;
pub mod package_policy;
pub mod package_recovery;
pub mod package_service;
pub mod pentest;
pub mod pep;
pub mod pep_decision;
pub mod pep_decision_service;
pub mod pep_config;
pub mod pep_security_policy;
pub mod pep_observability;
pub mod pep_doc;
pub mod pep_recovery;
pub mod pep_grant;
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
pub mod system_update;
pub mod system_update_config;
pub mod system_update_doc;
pub mod system_update_observability;
pub mod system_update_policy;
pub mod system_update_recovery;
pub mod system_update_service;
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
pub use hardware_recovery::{
    check_inventory_file, recover_inventory_file, recover_inventory_in_memory, validate_inventory,
    HardwareRecoveryAction, HardwareRecoveryReport, HardwareValidationReport,
};
pub use hardware_service::{HardwareScanOptions, HardwareService};
pub use network::{
    validate_interface_name, validate_ip_address, validate_mac_address, validate_mtu,
    validate_network_interface, validate_network_state, validate_route, DnsConfig, InterfaceType,
    IpAddress, IpFamily, NetworkInterface, NetworkState, OperState, Route, MAX_IFACE_NAME_LEN,
    MAX_INTERFACES, MAX_MTU, MAX_ROUTES, MIN_MTU,
};
pub use network_service::NetworkService;
pub use network_config::NetworkConfig;
pub use network_policy::{
    validate_policy_path as validate_network_policy_path, NetworkPolicyMode, NetworkPolicyReport,
    NetworkPolicyViolation, NetworkSecurityPolicy, MAX_POLICY_FILE_BYTES as MAX_NETWORK_POLICY_FILE_BYTES,
};
pub use network_recovery::{
    check_network_file, recover_network_file, recover_network_state_in_memory,
    save_recovered_state_to_path, validate_network_state as validate_network_state_integrity,
    validate_network_store_path, NetworkRecoveryAction, NetworkRecoveryReport,
    NetworkValidationReport, MAX_NETWORK_STORE_SIZE,
};
pub use types::GENESIS_HASH;
pub use system_update::{
    PartitionTarget, SystemSlotStatus, SystemUpdateStatus, UpdateArtifact,
    UpdateChannel, UpdateManifest, UpdateSlot, UpdateState,
    MAX_ARTIFACTS_PER_MANIFEST, MAX_ARTIFACT_FILENAME_LEN,
    MAX_UPDATE_ID_LEN, MAX_UPDATE_PAYLOAD_SIZE, MAX_UPDATE_VERSION_LEN,
    UPD_DIGEST_ERROR, UPD_SLOT_ERROR, UPD_STATE_ERROR, UPD_VALIDATION_ERROR,
};
pub use system_update_service::{SystemUpdateService, SystemUpdateServiceConfig};
pub use system_update_config::{
    SystemUpdateConfig, DEFAULT_UPDATE_CONFIG_PATH, DEFAULT_UPDATE_STAGING_DIR,
    DEFAULT_UPDATE_STATE_DIR, MAX_UPDATE_CONFIG_FILE_BYTES, UCONF_VALIDATION_ERROR,
};
pub use system_update_policy::{
    SystemUpdateSecurityPolicy, UpdatePolicyMode, UpdatePolicyReport, UpdatePolicyViolation,
    MAX_POLICY_FILE_BYTES, UPOL_IO_ERROR, UPOL_PARSE_ERROR, UPOL_PATH_ERROR, UPOL_VALIDATION_ERROR,
    validate_policy_path,
};
pub use system_update_observability::{
    SystemUpdateObservabilityReport, sanitize_telemetry_text,
};
pub use system_update_doc::{
    SystemUpdateDocCategory, SystemUpdateDocIndex, SystemUpdateDocSearchResult,
    SystemUpdateDocTopic,
};
pub use system_update_recovery::{
    check_update_files, recover_update_files_with_backup, recover_update_state_in_memory,
    validate_update_state, validate_update_store_path, SystemUpdateRecoveryAction,
    SystemUpdateRecoveryReport, SystemUpdateValidationReport, MAX_UPDATE_STORE_SIZE,
};
pub use capability::{
    Capability, CapabilityConstraints, CapabilityError, CapabilityRight, CapabilityScope,
    MAX_ACTIONS_PER_SCOPE, MAX_CAPABILITY_ID_LEN, MAX_ISSUER_LEN, MAX_RESOURCE_URI_LEN,
    MAX_SUBJECT_LEN, CAP_ERROR_ATTENUATION, CAP_ERROR_EXPIRED, CAP_ERROR_NOT_YET_VALID,
    CAP_ERROR_QUOTA_BYTES, CAP_ERROR_QUOTA_INVOCATIONS, CAP_ERROR_REVOKED, CAP_ERROR_RIGHT,
    CAP_ERROR_SCOPE, CAP_ERROR_VALIDATION,
};
pub use capability_service::{
    CapabilityService, CSERV_IO_ERROR, CSERV_NOT_FOUND, CSERV_VALIDATION_ERROR,
    MAX_CAPABILITY_STORE_SIZE,
};
pub use pep_config::{
    PepConfig, DEFAULT_PEP_STORE_PATH, MAX_CONFIG_BYTES as MAX_PEP_CONFIG_BYTES,
    MAX_STORE_BYTES as MAX_PEP_STORE_BYTES, MIN_STORE_BYTES as MIN_PEP_STORE_BYTES,
    DEFAULT_MAX_STORE_BYTES as DEFAULT_PEP_MAX_STORE_BYTES,
    DEFAULT_MAX_RULES as DEFAULT_PEP_MAX_RULES, MAX_RULES_COUNT as MAX_PEP_RULES_COUNT,
    MIN_RULES_COUNT as MIN_PEP_RULES_COUNT,
    PEPCONF_ERR_BOUNDS, PEPCONF_ERR_IO, PEPCONF_ERR_PARSE, PEPCONF_ERR_VALIDATION,
};
pub use pep_security_policy::{
    PepEnforcementMode, PepObligationCriticality, PepSecurityPolicy,
    MAX_PEP_POLICY_DESC_LEN, MAX_PEP_POLICY_VERSION_LEN, MAX_PEP_SECURITY_POLICY_BYTES,
    MAX_PREFIX_LEN, MAX_RESTRICTED_PREFIXES,
    PEPPOL_ERR_IO, PEPPOL_ERR_PRIVILEGE, PEPPOL_ERR_TEMPORAL, PEPPOL_ERR_VALIDATION,
    validate_policy_path as validate_pep_security_policy_path,
};
pub use pep_observability::{
    PepObservabilityReport, PEPOBS_ERR_VALIDATION, PEP_HEALTH_UTILIZATION_THRESHOLD,
    sanitize_telemetry_text as sanitize_pep_telemetry_text,
};
pub use pep_doc::{
    extract_utf8_snippet as extract_pep_doc_utf8_snippet, PepDocCategory, PepDocIndex,
    PepDocSearchResult, PepDocSection, PepDocTopic, MAX_DOC_QUERY_LEN as MAX_PEP_DOC_QUERY_LEN,
    MAX_DOC_SEARCH_RESULTS as MAX_PEP_DOC_SEARCH_RESULTS,
    MAX_SNIPPET_LEN as MAX_PEP_DOC_SNIPPET_LEN, MAX_TOPIC_ID_LEN as MAX_PEP_DOC_TOPIC_ID_LEN,
};
pub use pep_recovery::{
    PepIssueSeverity, PepRecoveryManager, PepRecoveryResult, PepRecoveryStrategy,
    PepStoreValidator, PepValidationIssue, PepValidationReport,
    PEPRECV_ERR_CAPACITY, PEPRECV_ERR_CHECKSUM, PEPRECV_ERR_DUPLICATE_ID,
    PEPRECV_ERR_FILE_SIZE, PEPRECV_ERR_IO, PEPRECV_ERR_PARSE,
    PEPRECV_ERR_PATH_TRAVERSAL, PEPRECV_ERR_RULE_SYNTAX,
};


