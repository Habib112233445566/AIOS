//! Network Bootstrap Observability Subsystem (NOBS1..NOBS6).
//!
//! Provides interface statistics collection, network link health evaluation,
//! historical metric snapshot ring buffering, and atomic telemetry persistence.

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::network::NetworkState;

/// Error constants for network observability operations.
pub const NOBS_IO_ERROR: &str = "NOBS_IO_ERROR";
pub const NOBS_PARSE_ERROR: &str = "NOBS_PARSE_ERROR";
pub const NOBS_PATH_ERROR: &str = "NOBS_PATH_ERROR";
pub const NOBS_VALIDATION_ERROR: &str = "NOBS_VALIDATION_ERROR";

/// Maximum allowed snapshot file size (1 MB) to prevent OOM / DoS (NOBS6).
pub const MAX_OBSERVABILITY_FILE_BYTES: u64 = 1_048_576;

/// Default capacity for the in-memory historical snapshot ring buffer (NOBS4).
pub const DEFAULT_HISTORY_CAPACITY: usize = 60;

/// Validates observability file path hygiene (no traversal, no control chars, max length 1024).
pub fn validate_observability_path(path: &Path) -> Result<(), String> {
    let path_str = path.to_str().ok_or_else(|| format!("{}: path must be valid UTF-8", NOBS_PATH_ERROR))?;
    if path_str.trim().is_empty() {
        return Err(format!("{}: path cannot be empty", NOBS_PATH_ERROR));
    }
    if path_str.len() > 1024 {
        return Err(format!("{}: path exceeds maximum length of 1024 characters", NOBS_PATH_ERROR));
    }
    if path_str.chars().any(|c| c.is_control() || c == '\0') {
        return Err(format!("{}: path cannot contain control characters", NOBS_PATH_ERROR));
    }
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(format!("{}: path traversal ('..') is not permitted", NOBS_PATH_ERROR));
    }
    Ok(())
}

/// Statistics and counters for an individual network interface (NOBS1, NOBS2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct InterfaceStatistics {
    pub interface_name: String,
    pub rx_bytes: u64,
    pub rx_packets: u64,
    pub rx_errors: u64,
    pub rx_dropped: u64,
    pub tx_bytes: u64,
    pub tx_packets: u64,
    pub tx_errors: u64,
    pub tx_dropped: u64,
    pub carrier: Option<bool>,
    pub collisions: u64,
}

/// High-level diagnostic verdict for network health (NOBS3).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkHealthVerdict {
    Healthy,
    Degraded,
    Critical,
}

impl Default for NetworkHealthVerdict {
    fn default() -> Self {
        Self::Healthy
    }
}

/// Diagnostic health assessment report (NOBS3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NetworkHealthReport {
    pub verdict: NetworkHealthVerdict,
    pub total_interfaces: usize,
    pub active_interfaces: usize,
    pub default_route_present: bool,
    pub dns_configured: bool,
    pub issues: Vec<String>,
}

/// Point-in-time observability snapshot including stats and health diagnosis (NOBS4, NOBS5).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkObservabilitySnapshot {
    pub timestamp: String,
    pub hostname: String,
    pub statistics: Vec<InterfaceStatistics>,
    pub health: NetworkHealthReport,
}

/// Service providing telemetry collection, health analysis, and history ring buffering.
pub struct NetworkObservabilityService {
    pub procfs_net_dev: PathBuf,
    pub sysfs_net_dir: PathBuf,
    pub history_capacity: usize,
    pub history: VecDeque<NetworkObservabilitySnapshot>,
}

impl NetworkObservabilityService {
    /// Creates a new service with standard host paths.
    pub fn new() -> Self {
        Self::with_paths(
            PathBuf::from("/proc/net/dev"),
            PathBuf::from("/sys/class/net"),
            DEFAULT_HISTORY_CAPACITY,
        )
    }

    /// Creates a service with custom paths for hermetic testing.
    pub fn with_paths(procfs_net_dev: PathBuf, sysfs_net_dir: PathBuf, history_capacity: usize) -> Self {
        let cap = history_capacity.clamp(1, 1000);
        Self {
            procfs_net_dev,
            sysfs_net_dir,
            history_capacity: cap,
            history: VecDeque::with_capacity(cap),
        }
    }

    /// Collects interface statistics from sysfs and procfs (NOBS1, NOBS2).
    pub fn collect_statistics(&self) -> Vec<InterfaceStatistics> {
        let mut stats_map = std::collections::BTreeMap::new();

        // 1. Try parsing procfs /proc/net/dev if exists
        if self.procfs_net_dev.exists() {
            if let Ok(meta) = fs::metadata(&self.procfs_net_dev) {
                if meta.len() <= 65536 {
                    if let Ok(content) = fs::read_to_string(&self.procfs_net_dev) {
                        for line in content.lines().take(1024) {
                            let line = line.trim();
                            if line.starts_with("Inter-") || line.starts_with("face") || !line.contains(':') {
                                continue;
                            }
                            if let Some((iface_part, stats_part)) = line.split_once(':') {
                                let iface_name = iface_part.trim().to_string();
                                if iface_name.is_empty() || iface_name.len() > 15 {
                                    continue;
                                }
                                let tokens: Vec<&str> = stats_part.split_whitespace().collect();
                                if tokens.len() >= 16 {
                                    let rx_bytes = tokens[0].parse::<u64>().unwrap_or(0);
                                    let rx_packets = tokens[1].parse::<u64>().unwrap_or(0);
                                    let rx_errors = tokens[2].parse::<u64>().unwrap_or(0);
                                    let rx_dropped = tokens[3].parse::<u64>().unwrap_or(0);
                                    let tx_bytes = tokens[8].parse::<u64>().unwrap_or(0);
                                    let tx_packets = tokens[9].parse::<u64>().unwrap_or(0);
                                    let tx_errors = tokens[10].parse::<u64>().unwrap_or(0);
                                    let tx_dropped = tokens[11].parse::<u64>().unwrap_or(0);
                                    let collisions = tokens[13].parse::<u64>().unwrap_or(0);

                                    stats_map.insert(
                                        iface_name.clone(),
                                        InterfaceStatistics {
                                            interface_name: iface_name,
                                            rx_bytes,
                                            rx_packets,
                                            rx_errors,
                                            rx_dropped,
                                            tx_bytes,
                                            tx_packets,
                                            tx_errors,
                                            tx_dropped,
                                            carrier: None,
                                            collisions,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Discover/enrich with sysfs statistics and carrier
        if self.sysfs_net_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.sysfs_net_dir) {
                for entry in entries.flatten() {
                    let iface_name = entry.file_name().to_string_lossy().to_string();
                    if iface_name.starts_with('.') || iface_name.len() > 15 {
                        continue;
                    }
                    let iface_path = entry.path();
                    let stat = stats_map.entry(iface_name.clone()).or_insert_with(|| InterfaceStatistics {
                        interface_name: iface_name.clone(),
                        ..Default::default()
                    });

                    // Check carrier file
                    let carrier_path = iface_path.join("carrier");
                    if carrier_path.exists() {
                        if let Ok(c) = fs::read_to_string(&carrier_path) {
                            match c.trim() {
                                "1" => stat.carrier = Some(true),
                                "0" => stat.carrier = Some(false),
                                _ => {}
                            }
                        }
                    }

                    // Check sysfs statistics directory if needed
                    let stats_dir = iface_path.join("statistics");
                    if stats_dir.exists() && stat.rx_bytes == 0 && stat.tx_bytes == 0 {
                        let read_u64 = |name: &str| -> u64 {
                            fs::read_to_string(stats_dir.join(name))
                                .ok()
                                .and_then(|s| s.trim().parse::<u64>().ok())
                                .unwrap_or(0)
                        };
                        stat.rx_bytes = read_u64("rx_bytes");
                        stat.rx_packets = read_u64("rx_packets");
                        stat.rx_errors = read_u64("rx_errors");
                        stat.rx_dropped = read_u64("rx_dropped");
                        stat.tx_bytes = read_u64("tx_bytes");
                        stat.tx_packets = read_u64("tx_packets");
                        stat.tx_errors = read_u64("tx_errors");
                        stat.tx_dropped = read_u64("tx_dropped");
                        stat.collisions = read_u64("collisions");
                    }
                }
            }
        }

        stats_map.into_values().collect()
    }

    /// Evaluates network health based on current state and statistics (NOBS3).
    pub fn evaluate_health(&self, state: &NetworkState, stats: &[InterfaceStatistics]) -> NetworkHealthReport {
        let mut issues = Vec::new();
        let total_interfaces = state.interfaces.len();
        let mut active_non_loopback = 0;
        let mut total_non_loopback = 0;

        for iface in &state.interfaces {
            let is_loopback = iface.iftype == crate::network::InterfaceType::Loopback || iface.name == "lo";
            if !is_loopback {
                total_non_loopback += 1;
                let is_up = iface.operstate == crate::network::OperState::Up;
                let carrier_ok = stats
                    .iter()
                    .find(|s| s.interface_name == iface.name)
                    .and_then(|s| s.carrier)
                    .unwrap_or(true);

                if is_up && carrier_ok {
                    active_non_loopback += 1;
                } else if !is_up {
                    issues.push(format!("Interface '{}' is down", iface.name));
                } else if !carrier_ok {
                    issues.push(format!("Interface '{}' has lost carrier/link", iface.name));
                }
            }
        }

        // Default route check
        let default_route_present = state.routes.iter().any(|r| {
            r.destination == "0.0.0.0/0" || r.destination == "::/0" || r.destination == "default"
        });
        if !default_route_present {
            issues.push("No default gateway route configured".into());
        }

        // DNS check
        let dns_configured = !state.dns.nameservers.is_empty();
        if !dns_configured {
            issues.push("No DNS nameservers configured".into());
        }

        // Check packet drop or error rate (> 5% drops when rx_packets > 100)
        for stat in stats {
            if stat.rx_packets > 100 && (stat.rx_dropped.saturating_mul(20) > stat.rx_packets || stat.rx_errors.saturating_mul(20) > stat.rx_packets) {
                issues.push(format!(
                    "Elevated error/drop rate on '{}' (rx_packets={}, rx_dropped={}, rx_errors={})",
                    stat.interface_name, stat.rx_packets, stat.rx_dropped, stat.rx_errors
                ));
            }
        }

        // Determine verdict (NOBS3)
        let verdict = if total_non_loopback > 0 && active_non_loopback == 0 {
            NetworkHealthVerdict::Critical
        } else if !default_route_present && !dns_configured {
            NetworkHealthVerdict::Critical
        } else if !issues.is_empty() {
            NetworkHealthVerdict::Degraded
        } else {
            NetworkHealthVerdict::Healthy
        };

        NetworkHealthReport {
            verdict,
            total_interfaces,
            active_interfaces: active_non_loopback,
            default_route_present,
            dns_configured,
            issues,
        }
    }


    /// Captures a point-in-time snapshot, appends to the history ring, and returns it (NOBS4).
    pub fn capture_snapshot(&mut self, state: &NetworkState) -> NetworkObservabilitySnapshot {
        let stats = self.collect_statistics();
        let health = self.evaluate_health(state, &stats);
        let snapshot = NetworkObservabilitySnapshot {
            timestamp: state.timestamp.clone(),
            hostname: state.hostname.clone(),
            statistics: stats,
            health,
        };
        if self.history.len() >= self.history_capacity {
            self.history.pop_front();
        }
        self.history.push_back(snapshot.clone());
        snapshot
    }

    /// Returns historical snapshots recorded so far.
    pub fn get_history(&self) -> Vec<NetworkObservabilitySnapshot> {
        self.history.iter().cloned().collect()
    }

    /// Saves snapshot atomically to disk (NOBS6).
    pub fn save_snapshot_to_path(&self, snapshot: &NetworkObservabilitySnapshot, path: &Path) -> Result<(), String> {
        validate_observability_path(path)?;
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| format!("{}: failed to serialize snapshot: {}", NOBS_PARSE_ERROR, e))?;
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: failed to create directory {}: {}", NOBS_IO_ERROR, parent.display(), e))?;
        }

        let tmp_file_name = format!(
            ".{}.tmp.{}",
            path.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "obs".into()),
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

        if let Err(e) = fs::write(&tmp_path, &json) {
            return Err(format!("{}: failed to write temp snapshot {}: {}", NOBS_IO_ERROR, tmp_path.display(), e));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
        }

        if let Err(e) = fs::rename(&tmp_path, path) {
            return Err(format!("{}: failed to atomically rename {} to {}: {}", NOBS_IO_ERROR, tmp_path.display(), path.display(), e));
        }

        guard.active = false;
        Ok(())
    }

    /// Loads snapshot from disk (NOBS6).
    pub fn load_snapshot_from_path(path: &Path) -> Result<NetworkObservabilitySnapshot, String> {
        validate_observability_path(path)?;
        if !path.exists() {
            return Err(format!("{}: file {} not found", NOBS_IO_ERROR, path.display()));
        }
        let metadata = fs::metadata(path)
            .map_err(|e| format!("{}: failed to read metadata for {}: {}", NOBS_IO_ERROR, path.display(), e))?;
        if metadata.len() > MAX_OBSERVABILITY_FILE_BYTES {
            return Err(format!(
                "{}: snapshot file {} size {} exceeds maximum ({} bytes)",
                NOBS_VALIDATION_ERROR,
                path.display(),
                metadata.len(),
                MAX_OBSERVABILITY_FILE_BYTES
            ));
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("{}: failed to read snapshot {}: {}", NOBS_IO_ERROR, path.display(), e))?;
        let snapshot: NetworkObservabilitySnapshot = serde_json::from_str(&content)
            .map_err(|e| format!("{}: failed to parse snapshot JSON: {}", NOBS_PARSE_ERROR, e))?;
        Ok(snapshot)
    }
}
