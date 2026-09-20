//! Network Bootstrap data models and invariant validation (NET1..NET6).
//!
//! Strongly-typed representation of network interfaces, IP addresses,
//! routes, DNS configurations, and complete host network states.

use std::net::IpAddr;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

/// Maximum permissible length for an interface name (IFNAMSIZ - 1).
pub const MAX_IFACE_NAME_LEN: usize = 15;
/// Minimum MTU allowed (RFC 791).
pub const MIN_MTU: u32 = 68;
/// Maximum MTU allowed (theoretical IP maximum).
pub const MAX_MTU: u32 = 65535;
/// Maximum network interfaces per host state.
pub const MAX_INTERFACES: usize = 1024;
/// Maximum routes per host state.
pub const MAX_ROUTES: usize = 4096;

/// Functional classification of a network interface.
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

impl InterfaceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InterfaceType::Loopback => "loopback",
            InterfaceType::Ethernet => "ethernet",
            InterfaceType::Wireless => "wireless",
            InterfaceType::Bridge => "bridge",
            InterfaceType::Bond => "bond",
            InterfaceType::Vlan => "vlan",
            InterfaceType::TunTap => "tuntap",
            InterfaceType::Other => "other",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "loopback" | "lo" => InterfaceType::Loopback,
            "ethernet" | "eth" | "en" | "eno" | "ens" | "enp" => InterfaceType::Ethernet,
            "wireless" | "wlan" | "wifi" | "wl" | "wlp" | "wls" => InterfaceType::Wireless,
            "bridge" | "br" => InterfaceType::Bridge,
            "bond" => InterfaceType::Bond,
            "vlan" => InterfaceType::Vlan,
            "tuntap" | "tun" | "tap" => InterfaceType::TunTap,
            _ => InterfaceType::Other,
        }
    }
}

/// Operational state of a network link (RFC 2863).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperState {
    Up,
    Down,
    Dormant,
    LowerLayerDown,
    Unknown,
}

impl OperState {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperState::Up => "up",
            OperState::Down => "down",
            OperState::Dormant => "dormant",
            OperState::LowerLayerDown => "lowerlayerdown",
            OperState::Unknown => "unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "up" => OperState::Up,
            "down" => OperState::Down,
            "dormant" => OperState::Dormant,
            "lowerlayerdown" => OperState::LowerLayerDown,
            _ => OperState::Unknown,
        }
    }
}

/// IP address family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpFamily {
    V4,
    V6,
}

/// Individual IP address assigned to an interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpAddress {
    pub address: String,
    pub prefix_len: u8,
    pub family: IpFamily,
}

impl IpAddress {
    pub fn new_v4(address: impl Into<String>, prefix_len: u8) -> Self {
        IpAddress {
            address: address.into(),
            prefix_len,
            family: IpFamily::V4,
        }
    }

    pub fn new_v6(address: impl Into<String>, prefix_len: u8) -> Self {
        IpAddress {
            address: address.into(),
            prefix_len,
            family: IpFamily::V6,
        }
    }

    pub fn from_cidr(cidr: &str) -> Result<Self, String> {
        let clean = cidr.trim();
        let (addr_str, prefix_str) = clean
            .split_once('/')
            .ok_or_else(|| format!("missing CIDR slash delimiter in '{}'", clean))?;
        let parsed = IpAddr::from_str(addr_str)
            .map_err(|e| format!("invalid IP address '{}': {}", addr_str, e))?;
        let prefix_len = prefix_str
            .parse::<u8>()
            .map_err(|_| format!("invalid CIDR prefix '{}'", prefix_str))?;

        let ip = match parsed {
            IpAddr::V4(_) => IpAddress::new_v4(addr_str, prefix_len),
            IpAddr::V6(_) => IpAddress::new_v6(addr_str, prefix_len),
        };
        ip.validate()?;
        Ok(ip)
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_ip_address(self)
    }
}

/// A routing table entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    pub metric: u32,
}

impl Route {
    pub fn new(destination: impl Into<String>, metric: u32) -> Self {
        Route {
            destination: destination.into(),
            gateway: None,
            interface: None,
            metric,
        }
    }

    pub fn with_gateway(mut self, gateway: impl Into<String>) -> Self {
        self.gateway = Some(gateway.into());
        self
    }

    pub fn with_interface(mut self, interface: impl Into<String>) -> Self {
        self.interface = Some(interface.into());
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_route(self)
    }
}

/// Host DNS resolver configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DnsConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nameservers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search_domains: Vec<String>,
}

/// An individual network interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub iftype: InterfaceType,
    pub operstate: OperState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    pub mtu: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ip_addresses: Vec<IpAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<String>,
}

impl NetworkInterface {
    pub fn new(name: impl Into<String>, iftype: InterfaceType) -> Self {
        NetworkInterface {
            name: name.into(),
            iftype,
            operstate: OperState::Unknown,
            mac_address: None,
            mtu: 1500,
            ip_addresses: Vec::new(),
            flags: Vec::new(),
        }
    }

    pub fn with_operstate(mut self, operstate: OperState) -> Self {
        self.operstate = operstate;
        self
    }

    pub fn with_mac(mut self, mac: impl Into<String>) -> Self {
        self.mac_address = Some(mac.into());
        self
    }

    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = mtu;
        self
    }

    pub fn with_ip(mut self, ip: IpAddress) -> Self {
        self.ip_addresses.push(ip);
        self
    }

    pub fn with_flag(mut self, flag: impl Into<String>) -> Self {
        self.flags.push(flag.into());
        self
    }

    pub fn is_up(&self) -> bool {
        self.operstate == OperState::Up || self.flags.iter().any(|f| f.eq_ignore_ascii_case("up"))
    }

    pub fn is_loopback(&self) -> bool {
        self.iftype == InterfaceType::Loopback || self.name == "lo"
    }

    pub fn primary_ipv4(&self) -> Option<&str> {
        self.ip_addresses
            .iter()
            .find(|ip| ip.family == IpFamily::V4)
            .map(|ip| ip.address.as_str())
    }

    pub fn primary_ipv6(&self) -> Option<&str> {
        self.ip_addresses
            .iter()
            .find(|ip| ip.family == IpFamily::V6)
            .map(|ip| ip.address.as_str())
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_network_interface(self)
    }
}

/// Complete host network state snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkState {
    pub timestamp: String,
    pub hostname: String,
    pub interfaces: Vec<NetworkInterface>,
    pub routes: Vec<Route>,
    pub dns: DnsConfig,
}

impl NetworkState {
    pub fn new(hostname: impl Into<String>) -> Self {
        NetworkState {
            timestamp: chrono::Utc::now().to_rfc3339(),
            hostname: hostname.into(),
            interfaces: Vec::new(),
            routes: Vec::new(),
            dns: DnsConfig::default(),
        }
    }

    pub fn add_interface(&mut self, iface: NetworkInterface) -> Result<(), String> {
        iface.validate()?;
        if self.interfaces.len() >= MAX_INTERFACES {
            return Err(format!(
                "network interfaces count reached maximum permitted limit of {}",
                MAX_INTERFACES
            ));
        }
        if self.interfaces.iter().any(|i| i.name == iface.name) {
            return Err(format!("duplicate interface name: {}", iface.name));
        }
        self.interfaces.push(iface);
        // Sort deterministically by name (NET6)
        self.interfaces.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(())
    }

    pub fn add_route(&mut self, route: Route) -> Result<(), String> {
        route.validate()?;
        if self.routes.len() >= MAX_ROUTES {
            return Err(format!(
                "routes count reached maximum permitted limit of {}",
                MAX_ROUTES
            ));
        }
        self.routes.push(route);
        // Sort deterministically by metric, then destination (NET6)
        self.routes.sort_by(|a, b| a.metric.cmp(&b.metric).then_with(|| a.destination.cmp(&b.destination)));
        Ok(())
    }

    pub fn get_interface(&self, name: &str) -> Option<&NetworkInterface> {
        self.interfaces.iter().find(|i| i.name == name)
    }

    pub fn interfaces_up(&self) -> Vec<&NetworkInterface> {
        self.interfaces.iter().filter(|i| i.is_up()).collect()
    }

    pub fn default_gateway(&self) -> Option<&str> {
        self.routes
            .iter()
            .find(|r| (r.destination == "0.0.0.0/0" || r.destination == "default") && r.gateway.is_some())
            .and_then(|r| r.gateway.as_deref())
    }

    pub fn default_gateway_v6(&self) -> Option<&str> {
        self.routes
            .iter()
            .find(|r| (r.destination == "::/0" || r.destination == "default") && r.gateway.is_some())
            .and_then(|r| r.gateway.as_deref())
    }

    pub fn find_interface_by_ip(&self, ip_str: &str) -> Option<&NetworkInterface> {
        self.interfaces.iter().find(|i| {
            i.ip_addresses.iter().any(|ip| ip.address == ip_str)
        })
    }

    pub fn validate_invariants(&self) -> Result<(), String> {
        validate_network_state(self)
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("failed to serialize network state: {}", e))
    }

    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("failed to serialize network state: {}", e))
    }

    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("failed to deserialize network state: {}", e))
    }
}

/// Validates interface name according to Linux IFNAMSIZ constraints (NET1).
pub fn validate_interface_name(name: &str) -> Result<(), String> {
    let clean = name.trim();
    if clean.is_empty() {
        return Err("interface name cannot be empty".into());
    }
    if clean.len() > MAX_IFACE_NAME_LEN {
        return Err(format!(
            "interface name '{}' exceeds maximum permitted length of {} characters (got {})",
            clean, MAX_IFACE_NAME_LEN, clean.len()
        ));
    }
    if !clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_') {
        return Err(format!(
            "interface name '{}' contains invalid characters (must be ASCII alphanumeric, '.', '-', or '_')",
            clean
        ));
    }
    Ok(())
}

/// Validates MAC address format (NET2).
pub fn validate_mac_address(mac: &str) -> Result<(), String> {
    let clean = mac.trim();
    if clean.is_empty() {
        return Ok(());
    }
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.len() != 6 {
        return Err(format!(
            "invalid MAC address format '{}' (must be 6 colon-separated octets)",
            clean
        ));
    }
    for part in parts {
        if part.len() != 2 || !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "invalid MAC address octet '{}' in '{}' (must be 2 hexadecimal digits)",
                part, clean
            ));
        }
    }
    Ok(())
}

/// Validates IP address and prefix length (NET3).
pub fn validate_ip_address(ip: &IpAddress) -> Result<(), String> {
    let parsed = IpAddr::from_str(ip.address.trim())
        .map_err(|e| format!("invalid IP address '{}': {}", ip.address, e))?;

    match (parsed, ip.family) {
        (IpAddr::V4(_), IpFamily::V4) => {
            if ip.prefix_len > 32 {
                return Err(format!(
                    "IPv4 prefix length {} exceeds maximum of 32",
                    ip.prefix_len
                ));
            }
        }
        (IpAddr::V6(_), IpFamily::V6) => {
            if ip.prefix_len > 128 {
                return Err(format!(
                    "IPv6 prefix length {} exceeds maximum of 128",
                    ip.prefix_len
                ));
            }
        }
        (IpAddr::V4(_), IpFamily::V6) => {
            return Err(format!(
                "IP address '{}' is IPv4 but family is specified as IPv6",
                ip.address
            ));
        }
        (IpAddr::V6(_), IpFamily::V4) => {
            return Err(format!(
                "IP address '{}' is IPv6 but family is specified as IPv4",
                ip.address
            ));
        }
    }
    Ok(())
}

/// Validates MTU bounds (NET4).
pub fn validate_mtu(mtu: u32) -> Result<(), String> {
    if mtu < MIN_MTU || mtu > MAX_MTU {
        return Err(format!(
            "MTU {} is out of permissible range [{}, {}]",
            mtu, MIN_MTU, MAX_MTU
        ));
    }
    Ok(())
}

/// Validates routing entry (NET5).
pub fn validate_route(route: &Route) -> Result<(), String> {
    let dest = route.destination.trim();
    if dest.is_empty() {
        return Err("route destination cannot be empty".into());
    }

    // Check CIDR format
    if let Some((addr_str, prefix_str)) = dest.split_once('/') {
        let _ = IpAddr::from_str(addr_str)
            .map_err(|e| format!("invalid route destination address '{}': {}", addr_str, e))?;
        let prefix = prefix_str.parse::<u8>()
            .map_err(|_| format!("invalid route prefix length '{}'", prefix_str))?;
        if prefix > 128 {
            return Err(format!("route prefix length {} exceeds 128", prefix));
        }
    } else {
        let _ = IpAddr::from_str(dest)
            .map_err(|e| format!("invalid route destination '{}': must be IP or CIDR ({})", dest, e))?;
    }

    if let Some(ref gw) = route.gateway {
        let _ = IpAddr::from_str(gw.trim())
            .map_err(|e| format!("invalid route gateway '{}': {}", gw, e))?;
    }

    if let Some(ref iface) = route.interface {
        validate_interface_name(iface)?;
    }

    if route.gateway.is_none() && route.interface.is_none() {
        return Err("route must specify at least a gateway or an interface".into());
    }

    Ok(())
}

/// Validates individual network interface consistency (NET1, NET2, NET4).
pub fn validate_network_interface(iface: &NetworkInterface) -> Result<(), String> {
    validate_interface_name(&iface.name)?;
    if let Some(ref mac) = iface.mac_address {
        validate_mac_address(mac)?;
    }
    validate_mtu(iface.mtu)?;
    for ip in &iface.ip_addresses {
        validate_ip_address(ip)?;
    }
    for flag in &iface.flags {
        if flag.trim().is_empty() || flag.len() > 32 || flag.chars().any(|c| c.is_control()) {
            return Err(format!("invalid interface flag '{}'", flag));
        }
    }
    Ok(())
}

/// Validates complete network state invariants (NET1..NET6).
pub fn validate_network_state(state: &NetworkState) -> Result<(), String> {
    if state.hostname.trim().is_empty() {
        return Err("network state hostname cannot be empty".into());
    }

    if state.interfaces.len() > MAX_INTERFACES {
        return Err(format!(
            "interface count {} exceeds maximum permitted limit of {}",
            state.interfaces.len(),
            MAX_INTERFACES
        ));
    }

    let mut seen_names = std::collections::HashSet::new();
    for iface in &state.interfaces {
        iface.validate()?;
        if !seen_names.insert(&iface.name) {
            return Err(format!("duplicate interface name '{}'", iface.name));
        }
    }

    if state.routes.len() > MAX_ROUTES {
        return Err(format!(
            "route count {} exceeds maximum permitted limit of {}",
            state.routes.len(),
            MAX_ROUTES
        ));
    }

    for route in &state.routes {
        route.validate()?;
    }

    for ns in &state.dns.nameservers {
        IpAddr::from_str(ns.trim())
            .map_err(|e| format!("invalid DNS nameserver '{}': {}", ns, e))?;
    }

    for domain in &state.dns.search_domains {
        let clean = domain.trim();
        if clean.is_empty() || clean.len() > 255 || clean.chars().any(|c| c.is_control()) {
            return Err(format!("invalid DNS search domain '{}'", domain));
        }
    }

    Ok(())
}
