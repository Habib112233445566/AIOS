//! Health check, validation, and corruption recovery for Network Bootstrap (NVAL1..NVAL6).
//!
//! Provides automated non-destructive self-healing, timestamped quarantine of damaged
//! configuration files, dangling route pruning, loopback interface restoration, and
//! fallback DNS injection.

use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::network::{
    InterfaceType, IpAddress, IpFamily, NetworkInterface, NetworkState, OperState,
};

/// Maximum permissible size for a network store or configuration file on disk (1 MB).
pub const MAX_NETWORK_STORE_SIZE: u64 = 1_048_576;
pub const MAX_RECOVERY_ISSUES: usize = 100;

pub const NVAL_PATH_ERROR: &str = "NVAL_PATH_ERROR";
pub const NVAL_IO_ERROR: &str = "NVAL_IO_ERROR";
pub const NVAL_VALIDATION_ERROR: &str = "NVAL_VALIDATION_ERROR";
pub const NVAL_PARSE_ERROR: &str = "NVAL_PARSE_ERROR";

/// Validates that a store or configuration path is safe, bounded, free from directory traversal, and ends with .json.
pub fn validate_network_store_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_str().ok_or_else(|| format!("{}: path must be valid UTF-8", NVAL_PATH_ERROR))?;
    if path_str.trim().is_empty() {
        return Err(format!("{}: path cannot be empty", NVAL_PATH_ERROR));
    }
    if path_str.len() > 1024 {
        return Err(format!("{}: path exceeds maximum length of 1024 characters", NVAL_PATH_ERROR));
    }
    if path_str.chars().any(|c| c.is_control() || c == '\0') {
        return Err(format!("{}: path cannot contain control characters", NVAL_PATH_ERROR));
    }
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(format!("{}: path traversal ('..') is not permitted", NVAL_PATH_ERROR));
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => Ok(()),
        _ => Err(format!("{}: path must have a '.json' extension", NVAL_PATH_ERROR)),
    }
}

/// Validation report detailing the integrity of a network state or configuration file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkValidationReport {
    pub store_path: String,
    pub total_interfaces: usize,
    pub valid_interfaces: usize,
    pub invalid_interfaces: usize,
    pub dangling_routes: Vec<String>,
    pub missing_default_route: bool,
    pub missing_loopback: bool,
    pub dns_configured: bool,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

impl NetworkValidationReport {
    /// Validates report internal invariants (NVAL1..NVAL4).
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.valid_interfaces + self.invalid_interfaces != self.total_interfaces {
            return Err(format!(
                "NVAL1 violated: valid_interfaces ({}) + invalid_interfaces ({}) != total_interfaces ({})",
                self.valid_interfaces, self.invalid_interfaces, self.total_interfaces
            ));
        }

        let expected_healthy = self.errors.is_empty()
            && self.invalid_interfaces == 0
            && self.dangling_routes.is_empty()
            && !self.missing_loopback
            && self.dns_configured;

        if self.healthy != expected_healthy {
            return Err(format!(
                "NVAL4 violated: healthy ({}) != expected_healthy ({})",
                self.healthy, expected_healthy
            ));
        }

        Ok(())
    }
}

/// Actions performed during automated network recovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkRecoveryAction {
    NoneRequired,
    QuarantineCorruptedStore { backup_path: String },
    PruneDanglingRoutes { pruned_count: usize },
    RestoreLoopback,
    SetDefaultDnsFallback { fallback_servers: Vec<String> },
    RecreateEmptyConfig,
}

/// Comprehensive report on an automated recovery operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkRecoveryReport {
    pub store_path: String,
    pub initial_validation: NetworkValidationReport,
    pub actions_taken: Vec<NetworkRecoveryAction>,
    pub final_validation: NetworkValidationReport,
    pub backup_path: Option<String>,
    pub recovered: bool,
    pub completed_at: String,
}

/// Validates an in-memory NetworkState against structure, limits, and integrity rules.
pub fn validate_network_state(state: &NetworkState, store_path: &Path) -> NetworkValidationReport {
    let mut errors = Vec::new();
    let mut valid_interfaces = 0;
    let mut invalid_interfaces = 0;
    let mut dangling_routes = Vec::new();

    if state.hostname.trim().is_empty() {
        errors.push("hostname cannot be empty".into());
    }

    let mut iface_names = std::collections::HashSet::new();
    let mut has_loopback = false;

    for iface in &state.interfaces {
        let mut iface_err = false;
        if iface.name.trim().is_empty() {
            errors.push("interface name cannot be empty".into());
            iface_err = true;
        } else if !iface_names.insert(iface.name.clone()) {
            errors.push(format!("duplicate interface name '{}'", iface.name));
            iface_err = true;
        }

        if iface.iftype == InterfaceType::Loopback || iface.name == "lo" {
            has_loopback = true;
        }

        if iface_err {
            invalid_interfaces += 1;
        } else {
            valid_interfaces += 1;
        }
    }

    let missing_loopback = !has_loopback;
    if missing_loopback {
        errors.push("missing required loopback interface ('lo')".into());
    }

    let mut default_route_present = false;
    for route in &state.routes {
        if route.destination == "0.0.0.0/0" || route.destination == "default" || route.destination == "::/0" {
            default_route_present = true;
        }
        if let Some(dev) = &route.interface {
            if !dev.is_empty() && !iface_names.contains(dev) {
                dangling_routes.push(format!("route dst '{}' points to unknown dev '{}'", route.destination, dev));
            }
        }
    }

    let missing_default_route = !default_route_present;

    let dns_configured = !state.dns.nameservers.is_empty();
    if !dns_configured {
        errors.push("no DNS nameservers configured".into());
    }

    let healthy = errors.is_empty()
        && invalid_interfaces == 0
        && dangling_routes.is_empty()
        && !missing_loopback
        && dns_configured;

    NetworkValidationReport {
        store_path: store_path.to_string_lossy().to_string(),
        total_interfaces: state.interfaces.len(),
        valid_interfaces,
        invalid_interfaces,
        dangling_routes,
        missing_default_route,
        missing_loopback,
        dns_configured,
        errors,
        healthy,
        evaluated_at: Utc::now().to_rfc3339(),
    }
}

/// Checks the integrity and health of a network state or configuration file on disk.
pub fn check_network_file(path: &Path) -> NetworkValidationReport {
    let evaluated_at = Utc::now().to_rfc3339();
    let store_path = path.to_string_lossy().to_string();

    if let Err(e) = validate_network_store_path(path) {
        return NetworkValidationReport {
            store_path,
            total_interfaces: 0,
            valid_interfaces: 0,
            invalid_interfaces: 0,
            dangling_routes: Vec::new(),
            missing_default_route: true,
            missing_loopback: true,
            dns_configured: false,
            errors: vec![e],
            healthy: false,
            evaluated_at,
        };
    }

    if !path.exists() {
        return NetworkValidationReport {
            store_path,
            total_interfaces: 0,
            valid_interfaces: 0,
            invalid_interfaces: 0,
            dangling_routes: Vec::new(),
            missing_default_route: true,
            missing_loopback: true,
            dns_configured: false,
            errors: vec![format!("{}: file not found: {}", NVAL_IO_ERROR, path.display())],
            healthy: false,
            evaluated_at,
        };
    }

    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            return NetworkValidationReport {
                store_path,
                total_interfaces: 0,
                valid_interfaces: 0,
                invalid_interfaces: 0,
                dangling_routes: Vec::new(),
                missing_default_route: true,
                missing_loopback: true,
                dns_configured: false,
                errors: vec![format!("{}: failed to read metadata: {}", NVAL_IO_ERROR, e)],
                healthy: false,
                evaluated_at,
            };
        }
    };

    if metadata.len() > MAX_NETWORK_STORE_SIZE {
        return NetworkValidationReport {
            store_path,
            total_interfaces: 0,
            valid_interfaces: 0,
            invalid_interfaces: 0,
            dangling_routes: Vec::new(),
            missing_default_route: true,
            missing_loopback: true,
            dns_configured: false,
            errors: vec![format!(
                "{}: file size {} exceeds maximum permitted limit of {} bytes",
                NVAL_VALIDATION_ERROR,
                metadata.len(),
                MAX_NETWORK_STORE_SIZE
            )],
            healthy: false,
            evaluated_at,
        };
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return NetworkValidationReport {
                store_path,
                total_interfaces: 0,
                valid_interfaces: 0,
                invalid_interfaces: 0,
                dangling_routes: Vec::new(),
                missing_default_route: true,
                missing_loopback: true,
                dns_configured: false,
                errors: vec![format!("{}: failed to read file: {}", NVAL_IO_ERROR, e)],
                healthy: false,
                evaluated_at,
            };
        }
    };

    let state: NetworkState = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            return NetworkValidationReport {
                store_path,
                total_interfaces: 0,
                valid_interfaces: 0,
                invalid_interfaces: 0,
                dangling_routes: Vec::new(),
                missing_default_route: true,
                missing_loopback: true,
                dns_configured: false,
                errors: vec![format!("{}: JSON parse error: {}", NVAL_PARSE_ERROR, e)],
                healthy: false,
                evaluated_at,
            };
        }
    };

    validate_network_state(&state, path)
}

/// Recovers an in-memory NetworkState, healing missing loopback, pruning dangling routes, and restoring DNS.
pub fn recover_network_state_in_memory(
    state: &mut NetworkState,
    store_path: &Path,
) -> NetworkRecoveryReport {
    let initial_validation = validate_network_state(state, store_path);
    if initial_validation.healthy {
        return NetworkRecoveryReport {
            store_path: store_path.to_string_lossy().to_string(),
            initial_validation: initial_validation.clone(),
            actions_taken: vec![NetworkRecoveryAction::NoneRequired],
            final_validation: initial_validation,
            backup_path: None,
            recovered: true,
            completed_at: Utc::now().to_rfc3339(),
        };
    }

    let mut actions_taken = Vec::new();

    // 1. Restore loopback if missing
    if initial_validation.missing_loopback {
        let lo = NetworkInterface {
            name: "lo".into(),
            mac_address: None,
            flags: vec!["UP".into(), "LOOPBACK".into()],
            operstate: OperState::Up,
            mtu: 65536,
            iftype: InterfaceType::Loopback,
            ip_addresses: vec![
                IpAddress {
                    address: "127.0.0.1".into(),
                    prefix_len: 8,
                    family: IpFamily::V4,
                },
                IpAddress {
                    address: "::1".into(),
                    prefix_len: 128,
                    family: IpFamily::V6,
                },
            ],
        };
        state.interfaces.insert(0, lo);
        actions_taken.push(NetworkRecoveryAction::RestoreLoopback);
    }

    // 2. Prune dangling routes
    if !initial_validation.dangling_routes.is_empty() {
        let iface_names: std::collections::HashSet<String> =
            state.interfaces.iter().map(|i| i.name.clone()).collect();
        let before_count = state.routes.len();
        state.routes.retain(|r| match &r.interface {
            Some(dev) => iface_names.contains(dev) || dev.is_empty(),
            None => true,
        });
        let pruned = before_count.saturating_sub(state.routes.len());
        if pruned > 0 {
            actions_taken.push(NetworkRecoveryAction::PruneDanglingRoutes {
                pruned_count: pruned,
            });
        }
    }

    // 3. Set fallback DNS if empty
    if !initial_validation.dns_configured {
        let fallbacks = vec!["1.1.1.1".into(), "8.8.8.8".into()];
        state.dns.nameservers = fallbacks.clone();
        actions_taken.push(NetworkRecoveryAction::SetDefaultDnsFallback {
            fallback_servers: fallbacks,
        });
    }

    let final_validation = validate_network_state(state, store_path);
    let recovered = final_validation.healthy;

    NetworkRecoveryReport {
        store_path: store_path.to_string_lossy().to_string(),
        initial_validation,
        actions_taken,
        final_validation,
        backup_path: None,
        recovered,
        completed_at: Utc::now().to_rfc3339(),
    }
}

/// Atomically saves a NetworkState to disk with path validation and RAII temp file cleanup (NVAL6).
pub fn save_recovered_state_to_path(state: &NetworkState, path: &Path) -> Result<(), String> {
    validate_network_store_path(path)?;
    let serialized = serde_json::to_string_pretty(state)
        .map_err(|e| format!("{}: serialization error: {}", NVAL_VALIDATION_ERROR, e))?;

    if serialized.len() as u64 > MAX_NETWORK_STORE_SIZE {
        return Err(format!(
            "{}: state serialized size {} exceeds limit of {} bytes",
            NVAL_VALIDATION_ERROR,
            serialized.len(),
            MAX_NETWORK_STORE_SIZE
        ));
    }

    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    if !parent.as_os_str().is_empty() && !parent.exists() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("{}: failed to create dir {}: {}", NVAL_IO_ERROR, parent.display(), e))?;
    }

    let tmp_file_name = format!(
        ".{}.tmp.{}",
        path.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "store".into()),
        std::process::id()
    );
    let tmp_path = if parent.as_os_str().is_empty() {
        PathBuf::from(tmp_file_name)
    } else {
        parent.join(tmp_file_name)
    };

    struct TempFileGuard {
        path: PathBuf,
        active: bool,
    }
    impl Drop for TempFileGuard {
        fn drop(&mut self) {
            if self.active && self.path.exists() {
                let _ = fs::remove_file(&self.path);
            }
        }
    }

    let mut guard = TempFileGuard {
        path: tmp_path.clone(),
        active: true,
    };

    fs::write(&tmp_path, serialized)
        .map_err(|e| format!("{}: failed to write temporary file: {}", NVAL_IO_ERROR, e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
    }

    fs::rename(&tmp_path, path)
        .map_err(|e| format!("{}: failed to rename temporary file to {}: {}", NVAL_IO_ERROR, path.display(), e))?;

    guard.active = false;
    Ok(())
}

/// Recovers a network state or configuration file on disk, quarantining damaged files (NVAL5).
pub fn recover_network_file(path: &Path) -> Result<NetworkRecoveryReport, String> {
    validate_network_store_path(path)?;
    let initial_validation = check_network_file(path);

    if initial_validation.healthy {
        return Ok(NetworkRecoveryReport {
            store_path: path.to_string_lossy().to_string(),
            initial_validation: initial_validation.clone(),
            actions_taken: vec![NetworkRecoveryAction::NoneRequired],
            final_validation: initial_validation,
            backup_path: None,
            recovered: true,
            completed_at: Utc::now().to_rfc3339(),
        });
    }

    let mut backup_path = None;
    let mut actions_taken = Vec::new();

    // If file exists and is damaged/corrupted or oversized, quarantine it (NVAL5)
    if path.exists() {
        let ts = Utc::now().format("%Y%m%d_%H%M%S_%f").to_string();
        let bak = path.with_extension(format!("bak.{}_{}", ts, std::process::id()));
        fs::copy(path, &bak)
            .map_err(|e| format!("{}: failed to quarantine corrupted file to {}: {}", NVAL_IO_ERROR, bak.display(), e))?;
        let bak_str = bak.to_string_lossy().to_string();
        backup_path = Some(bak_str.clone());
        actions_taken.push(NetworkRecoveryAction::QuarantineCorruptedStore {
            backup_path: bak_str,
        });
    }

    // Try parsing existing file if readable and bounded
    let mut state = if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str::<NetworkState>(&content).unwrap_or_else(|_| {
                actions_taken.push(NetworkRecoveryAction::RecreateEmptyConfig);
                NetworkState::new("aiosh-recovered-node")
            })
        } else {
            actions_taken.push(NetworkRecoveryAction::RecreateEmptyConfig);
            NetworkState::new("aiosh-recovered-node")
        }
    } else {
        actions_taken.push(NetworkRecoveryAction::RecreateEmptyConfig);
        NetworkState::new("aiosh-recovered-node")
    };

    let report_in_mem = recover_network_state_in_memory(&mut state, path);
    for act in report_in_mem.actions_taken {
        if act != NetworkRecoveryAction::NoneRequired {
            actions_taken.push(act);
        }
    }

    save_recovered_state_to_path(&state, path)?;

    let final_validation = check_network_file(path);
    let recovered = final_validation.healthy;

    Ok(NetworkRecoveryReport {
        store_path: path.to_string_lossy().to_string(),
        initial_validation,
        actions_taken,
        final_validation,
        backup_path,
        recovered,
        completed_at: Utc::now().to_rfc3339(),
    })
}
