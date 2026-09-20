# Task Evidence: T-01802 - Network Bootstrap / Data Model: Specification

## Metadata
- **Task ID:** `T-01802`
- **Sub-Epic:** Sub-Epic 1: Network Bootstrap / Data Model
- **Epic:** Phase 1 — Linux Base System & Bootable Target / Network Bootstrap
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Technical Specification

### 1. Data Types & Structs

#### `InterfaceType` & `OperState`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceType {
    Loopback,
    Ethernet,
    Wireless,
    Bridge,
    Bond,
    Vlan,
    TunTap,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperState {
    Up,
    Down,
    Dormant,
    LowerLayerDown,
    Unknown,
}
```

#### `IpAddress` & `IpFamily`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpFamily {
    V4,
    V6,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpAddress {
    pub address: String,
    pub prefix_len: u8,
    pub family: IpFamily,
}
```

#### `Route` & `DnsConfig`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: Option<String>,
    pub metric: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DnsConfig {
    pub nameservers: Vec<String>,
    pub search_domains: Vec<String>,
}
```

#### `NetworkInterface` & `NetworkState`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub iftype: InterfaceType,
    pub operstate: OperState,
    pub mac_address: Option<String>,
    pub mtu: u32,
    pub ip_addresses: Vec<IpAddress>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkState {
    pub timestamp: String,
    pub hostname: String,
    pub interfaces: Vec<NetworkInterface>,
    pub routes: Vec<Route>,
    pub dns: DnsConfig,
}
```

### 2. Invariants & Validation Rules (NET1..NET6)
- **NET1 (Interface Identification)**: Interface names must be 1 to 15 ASCII characters matching `^[a-zA-Z0-9_.-]+$`.
- **NET2 (MAC Address Hygiene)**: MAC address must be 6 hex bytes separated by colons (`^([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}$`) or `None`.
- **NET3 (IP Address & CIDR Hygiene)**: Address must parse into `std::net::IpAddr`. Prefix length must be $\le 32$ for IPv4 and $\le 128$ for IPv6.
- **NET4 (MTU Bounds)**: MTU must be between 68 and 65,535.
- **NET5 (Route Validity)**: Route must have non-empty destination CIDR, non-negative metric, and at least one of `gateway` or `interface` defined.
- **NET6 (Deterministic Canonical Output)**: Interfaces are sorted alphabetically by name; routes are sorted by metric then destination; JSON serialization uses sorted keys.

### 3. Core API Functions
- `pub fn validate_interface_name(name: &str) -> Result<(), String>`
- `pub fn validate_mac_address(mac: &str) -> Result<(), String>`
- `pub fn validate_ip_address(ip: &IpAddress) -> Result<(), String>`
- `pub fn validate_mtu(mtu: u32) -> Result<(), String>`
- `pub fn validate_route(route: &Route) -> Result<(), String>`
- `pub fn validate_network_interface(iface: &NetworkInterface) -> Result<(), String>`
- `pub fn validate_network_state(state: &NetworkState) -> Result<(), String>`
