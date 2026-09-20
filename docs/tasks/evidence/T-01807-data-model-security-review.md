# Task Evidence: T-01807 - Network Bootstrap / Data Model: Security Review

## Metadata
- **Task ID:** `T-01807`
- **Sub-Epic:** Sub-Epic 1: Network Bootstrap / Data Model
- **Component:** `code/aiosh-rust/aiosh-core/src/network.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Security Review & Threat Modeling

The Network Bootstrap Data Model represents low-level Linux networking abstractions (interfaces, MAC addresses, IP addresses, MTU, routes, DNS resolvers, and network state). Since network metadata is sourced from sysfs, procfs, and user configurations, strict data validation is essential to prevent vulnerabilities.

### Threat Matrix

| Threat ID | Threat Vector | Impact | Severity | Mitigation & Invariant |
|:---|:---|:---|:---|:---|
| **THREAT-NET-01** | Interface Name Path Traversal & Injection | Path traversal in `/sys/class/net/{name}` or command injection in `ip link` / `ifconfig` invocations. | **HIGH** | Invariant **`NET1`**: Strict validation $\le 15$ chars (Linux `IFNAMSIZ - 1`), regex `^[a-zA-Z0-9_.-]+$`, explicit rejection of `/`, `\`, `..`, null bytes, and shell metacharacters. |
| **THREAT-NET-02** | MAC Address Spoofing & Malformed Octets | Buffer overflow or parser crashes in ARP/NDP handling. | **MEDIUM** | Invariant **`NET2`**: Strict regex `^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$` or empty/None for interfaces without hardware addresses (e.g., loopback). |
| **THREAT-NET-03** | IP Prefix & Family Mismatch | Out-of-bounds prefix lengths ($>32$ for IPv4, $>128$ for IPv6) leading to panic or routing table corruption. | **HIGH** | Invariant **`NET3`**: Strict validation of IP parsing and prefix bounds ($0 \le \text{prefix} \le 32$ for IPv4, $0 \le \text{prefix} \le 128$ for IPv6). |
| **THREAT-NET-04** | MTU Out-of-Bounds Exploitation | MTU $< 68$ breaks IPv4 packet handling; MTU $> 65535$ overflows 16-bit hardware buffer registers. | **MEDIUM** | Invariant **`NET4`**: Enforce strict range $68 \le \text{MTU} \le 65535$. |
| **THREAT-NET-05** | Route Poisoning & Missing Destinations | Empty destination CIDR, negative metrics, or routes lacking both gateway and interface causing kernel routing errors. | **HIGH** | Invariant **`NET5`**: Route destination must be valid CIDR, metric $\ge 0$, and at least gateway or interface must be specified. |
| **THREAT-NET-06** | Unbounded Collection Memory Exhaustion (DoS) | A crafted or corrupted state file with millions of interfaces or routes exhausting heap memory. | **HIGH** | Invariant **`NET6`**: Impose hard caps: `MAX_INTERFACES = 1024`, `MAX_ROUTES = 4096`, `MAX_ADDRESSES_PER_IFACE = 64`, `MAX_DNS_SERVERS = 32`. |

---

## 2. Hardening Recommendations for T-01808

1. **Add Hard Caps**:
   - Define constants in `network.rs`:
     - `pub const MAX_INTERFACES: usize = 1024;`
     - `pub const MAX_ROUTES: usize = 4096;`
     - `pub const MAX_ADDRESSES_PER_IFACE: usize = 64;`
     - `pub const MAX_DNS_SERVERS: usize = 32;`
2. **Add Comprehensive Validation Methods**:
   - Implement `NetworkInterface::validate(&self) -> Result<(), NetworkError>`
   - Implement `NetworkState::validate(&self) -> Result<(), NetworkError>`
   - Implement `Route::validate(&self) -> Result<(), NetworkError>`
   - Implement `DnsConfig::validate(&self) -> Result<(), NetworkError>`
3. **Automated Test Coverage**:
   - Verify all bounds and error paths in `code/aiosh-rust/aiosh-core/tests/test_network.rs`.
