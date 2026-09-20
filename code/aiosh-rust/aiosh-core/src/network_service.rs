//! Network Bootstrap Core Service (NSERV1..NSERV6).
//!
//! Provides interface discovery, routing table parsing, DNS resolver inspection,
//! and host network state coordination.

use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use crate::network::{
    validate_interface_name, DnsConfig, InterfaceType, NetworkInterface, NetworkState, OperState,
    Route, MAX_DNS_NAMESERVERS, MAX_DNS_SEARCH_DOMAINS, MAX_INTERFACES, MAX_ROUTES,
};

/// Core network management and discovery service.
#[derive(Debug, Clone)]
pub struct NetworkService {
    sysfs_net_root: PathBuf,
    procfs_root: PathBuf,
    resolv_conf_path: PathBuf,
}

impl Default for NetworkService {
    fn default() -> Self {
        NetworkService {
            sysfs_net_root: PathBuf::from("/sys/class/net"),
            procfs_root: PathBuf::from("/proc/net"),
            resolv_conf_path: PathBuf::from("/etc/resolv.conf"),
        }
    }
}

impl NetworkService {
    /// Creates a new `NetworkService` using standard Linux filesystem paths.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `NetworkService` pointing to custom paths (NSERV1 for hermetic testing).
    pub fn with_paths(
        sysfs_net_root: impl Into<PathBuf>,
        procfs_root: impl Into<PathBuf>,
        resolv_conf_path: impl Into<PathBuf>,
    ) -> Self {
        NetworkService {
            sysfs_net_root: sysfs_net_root.into(),
            procfs_root: procfs_root.into(),
            resolv_conf_path: resolv_conf_path.into(),
        }
    }

    /// Returns the configured sysfs network root directory.
    pub fn sysfs_net_root(&self) -> &Path {
        &self.sysfs_net_root
    }

    /// Returns the configured procfs root directory.
    pub fn procfs_root(&self) -> &Path {
        &self.procfs_root
    }

    /// Returns the configured resolv.conf file path.
    pub fn resolv_conf_path(&self) -> &Path {
        &self.resolv_conf_path
    }

    /// Scans the sysfs network class directory and discovers all interfaces (NSERV1, NSERV2).
    pub fn scan_interfaces(&self) -> Result<Vec<NetworkInterface>, String> {
        if !self.sysfs_net_root.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.sysfs_net_root)
            .map_err(|e| format!("failed to read sysfs network directory '{:?}': {}", self.sysfs_net_root, e))?;

        let mut interfaces = Vec::new();

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().to_string();

            // Validate interface name (NET1)
            if validate_interface_name(&name).is_err() {
                continue;
            }

            let path = entry.path();
            if !path.is_dir() {
                // On Linux, /sys/class/net/* are symlinks to /sys/devices/...
                // In mock setups, they may be directories or symlinks.
                // Check metadata following symlinks.
                if let Ok(meta) = fs::metadata(&path) {
                    if !meta.is_dir() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if interfaces.len() >= MAX_INTERFACES {
                break;
            }

            let iface = self.parse_interface_dir(&path, &name);
            interfaces.push(iface);
        }

        // Sort deterministically by name (NET6)
        interfaces.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(interfaces)
    }

    /// Retrieves an individual network interface by name.
    pub fn get_interface(&self, name: &str) -> Result<Option<NetworkInterface>, String> {
        validate_interface_name(name)?;
        let path = self.sysfs_net_root.join(name);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(self.parse_interface_dir(&path, name)))
    }

    /// Parses an interface sysfs directory with graceful fallbacks (NSERV2).
    fn parse_interface_dir(&self, dir: &Path, name: &str) -> NetworkInterface {
        // 1. Operational state
        let operstate = fs::read_to_string(dir.join("operstate"))
            .map(|s| OperState::from_str_loose(&s))
            .unwrap_or(OperState::Unknown);

        // 2. Hardware type
        let iftype = fs::read_to_string(dir.join("type"))
            .ok()
            .and_then(|s| s.trim().parse::<u16>().ok())
            .map(|t| match t {
                1 => InterfaceType::Ethernet,
                772 => InterfaceType::Loopback,
                801..=803 => InterfaceType::Wireless,
                _ => InterfaceType::from_str_loose(name),
            })
            .unwrap_or_else(|| InterfaceType::from_str_loose(name));

        let mut iface = NetworkInterface::new(name, iftype).with_operstate(operstate);

        // 3. MAC address
        if let Ok(addr_raw) = fs::read_to_string(dir.join("address")) {
            let clean = addr_raw.trim().to_ascii_lowercase();
            if !clean.is_empty() && clean != "00:00:00:00:00:00" && !iface.is_loopback() {
                iface = iface.with_mac(clean);
            }
        }

        // 4. MTU
        if let Ok(mtu_raw) = fs::read_to_string(dir.join("mtu")) {
            if let Ok(mtu) = mtu_raw.trim().parse::<u32>() {
                iface = iface.with_mtu(mtu);
            }
        }

        // 5. Flags
        if let Ok(flags_raw) = fs::read_to_string(dir.join("flags")) {
            let clean = flags_raw.trim();
            let hex_str = clean.strip_prefix("0x").unwrap_or(clean);
            if let Ok(mask) = u32::from_str_radix(hex_str, 16) {
                if mask & 0x1 != 0 {
                    iface = iface.with_flag("UP");
                }
                if mask & 0x2 != 0 {
                    iface = iface.with_flag("BROADCAST");
                }
                if mask & 0x8 != 0 {
                    iface = iface.with_flag("LOOPBACK");
                }
                if mask & 0x40 != 0 {
                    iface = iface.with_flag("RUNNING");
                }
                if mask & 0x1000 != 0 {
                    iface = iface.with_flag("MULTICAST");
                }
            }
        }

        iface
    }

    /// Parses `/proc/net/route` IPv4 routing table (NSERV3).
    pub fn scan_routes(&self) -> Result<Vec<Route>, String> {
        let route_file = self.procfs_root.join("route");
        if !route_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&route_file)
            .map_err(|e| format!("failed to read route file '{:?}': {}", route_file, e))?;

        let mut routes = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            if idx == 0 {
                // Header: Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 8 {
                continue;
            }

            let iface_name = parts[0];
            let dest_hex = parts[1];
            let gw_hex = parts[2];
            let metric_str = parts[6];
            let mask_hex = parts[7];

            let dest_val = match u32::from_str_radix(dest_hex, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let gw_val = match u32::from_str_radix(gw_hex, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let mask_val = match u32::from_str_radix(mask_hex, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let metric = metric_str.parse::<u32>().unwrap_or(0);

            // Convert little-endian u32 to Ipv4Addr
            let dest_bytes = dest_val.to_le_bytes();
            let dest_ip = Ipv4Addr::new(dest_bytes[0], dest_bytes[1], dest_bytes[2], dest_bytes[3]);

            let prefix_len = mask_val.count_ones() as u8;

            let dest_cidr = if dest_val == 0 && mask_val == 0 {
                "0.0.0.0/0".to_string()
            } else {
                format!("{}/{}", dest_ip, prefix_len)
            };

            let mut route = Route::new(dest_cidr, metric).with_interface(iface_name);

            if gw_val != 0 {
                let gw_bytes = gw_val.to_le_bytes();
                let gw_ip = Ipv4Addr::new(gw_bytes[0], gw_bytes[1], gw_bytes[2], gw_bytes[3]);
                route = route.with_gateway(gw_ip.to_string());
            }

            if routes.len() >= MAX_ROUTES {
                break;
            }

            routes.push(route);
        }

        // Sort deterministically by metric ascending, then destination ascending (NET6)
        routes.sort_by(|a, b| a.metric.cmp(&b.metric).then_with(|| a.destination.cmp(&b.destination)));
        Ok(routes)
    }

    /// Parses `/etc/resolv.conf` DNS configuration (NSERV4).
    pub fn get_dns_config(&self) -> Result<DnsConfig, String> {
        if !self.resolv_conf_path.exists() {
            return Ok(DnsConfig::default());
        }

        let content = fs::read_to_string(&self.resolv_conf_path)
            .map_err(|e| format!("failed to read resolv.conf '{:?}': {}", self.resolv_conf_path, e))?;

        let mut dns = DnsConfig::new();

        for line in content.lines() {
            let clean = line.trim();
            if clean.is_empty() || clean.starts_with('#') || clean.starts_with(';') {
                continue;
            }

            let parts: Vec<&str> = clean.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0].to_ascii_lowercase().as_str() {
                "nameserver" => {
                    if parts.len() >= 2 && dns.nameservers.len() < MAX_DNS_NAMESERVERS {
                        dns = dns.with_nameserver(parts[1]);
                    }
                }
                "search" | "domain" => {
                    for domain in parts.iter().skip(1) {
                        if dns.search_domains.len() < MAX_DNS_SEARCH_DOMAINS {
                            dns = dns.with_search_domain(*domain);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(dns)
    }

    /// Gathers full host network state snapshot.
    pub fn get_network_state(&self) -> Result<NetworkState, String> {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "aios-host".into());

        // Sanitize hostname for NetworkState
        let clean_hostname: String = hostname
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
            .collect();
        let final_hostname = if clean_hostname.is_empty() {
            "aios-host".to_string()
        } else {
            clean_hostname
        };

        let mut state = NetworkState::new(final_hostname);
        let ifaces = self.scan_interfaces()?;
        for iface in ifaces {
            state.add_interface(iface)?;
        }

        let routes = self.scan_routes()?;
        for route in routes {
            state.add_route(route)?;
        }

        state.dns = self.get_dns_config()?;

        // Validate state invariants (NET1..NET6)
        state.validate_invariants()?;
        Ok(state)
    }

    /// Requests to bring an interface up (NSERV5).
    pub fn bring_up(&self, iface: &str) -> Result<(), String> {
        validate_interface_name(iface)?;
        let oper_path = self.sysfs_net_root.join(iface).join("operstate");
        if oper_path.exists() {
            fs::write(&oper_path, "up\n")
                .map_err(|e| format!("failed to set operstate up for '{}': {}", iface, e))?;
        }
        Ok(())
    }

    /// Requests to bring an interface down (NSERV5).
    pub fn bring_down(&self, iface: &str) -> Result<(), String> {
        validate_interface_name(iface)?;
        let oper_path = self.sysfs_net_root.join(iface).join("operstate");
        if oper_path.exists() {
            fs::write(&oper_path, "down\n")
                .map_err(|e| format!("failed to set operstate down for '{}': {}", iface, e))?;
        }
        Ok(())
    }
}
