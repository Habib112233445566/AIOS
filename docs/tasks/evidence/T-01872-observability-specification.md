# Task Evidence: T-01872 - Network Bootstrap / observability: Specification

## 1. Overview
- **Task ID**: `T-01872`
- **Sub-Epic**: 8 (Network Bootstrap Observability)
- **Goal**: Formally specify the data structures, interfaces, health verdict rules, and persistence contract for Network Bootstrap observability.

---

## 2. Data Types & Contracts

### 2.1 `InterfaceStatistics`
```rust
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
```

### 2.2 `NetworkHealthVerdict` and `NetworkHealthReport`
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkHealthVerdict {
    Healthy,
    Degraded,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkHealthReport {
    pub verdict: NetworkHealthVerdict,
    pub total_interfaces: usize,
    pub active_interfaces: usize,
    pub default_route_present: bool,
    pub dns_configured: bool,
    pub issues: Vec<String>,
}
```

### 2.3 `NetworkObservabilitySnapshot`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkObservabilitySnapshot {
    pub timestamp: String,
    pub hostname: String,
    pub statistics: Vec<InterfaceStatistics>,
    pub health: NetworkHealthReport,
}
```

### 2.4 Service Contract: `NetworkObservabilityService`
- `new() -> Self`: Defaults to `/proc/net/dev` and `/sys/class/net/` with capacity 60.
- `with_paths(procfs_dev: PathBuf, sysfs_net: PathBuf) -> Self`: Hermetic testing constructor.
- `collect_statistics(&self) -> Vec<InterfaceStatistics>`: Non-blocking parser for procfs/sysfs.
- `evaluate_health(&self, state: &NetworkState, stats: &[InterfaceStatistics]) -> NetworkHealthReport`.
- `capture_snapshot(&mut self, state: &NetworkState) -> NetworkObservabilitySnapshot`: Captures snapshot and stores in ring buffer.
- `get_history(&self) -> Vec<NetworkObservabilitySnapshot>`: Returns chronological history.
- `save_snapshot_to_path(&self, snapshot: &NetworkObservabilitySnapshot, path: &Path) -> Result<(), String>`.
- `load_snapshot_from_path(path: &Path) -> Result<NetworkObservabilitySnapshot, String>`.

---

## 3. Error Handling & Standard Codes
- `NOBS_IO_ERROR`: Filesystem read/stat failure.
- `NOBS_PARSE_ERROR`: Parsing `/proc/net/dev` or JSON serialization failure.
- `NOBS_PATH_ERROR`: Path hygiene violation (length > 1024, traversal `..`, control characters).
- `NOBS_VALIDATION_ERROR`: Exceeding 1 MB size limits or invalid buffer capacity.
