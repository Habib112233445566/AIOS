# Task Evidence: T-01817 - Network Bootstrap / Core Service: Security Review

## Metadata
- **Task ID:** `T-01817`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `code/aiosh-rust/aiosh-core/src/network_service.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Security Review & Threat Modeling

The `NetworkService` is the operational core that reads kernel network metadata and executes link state modifications. A thorough security analysis was conducted covering filesystem access, parser robustness, and state mutation safety.

### Threat Matrix

| Threat ID | Threat Vector | Impact | Severity | Mitigation & Invariant |
|:---|:---|:---|:---|:---|
| **THREAT-NSERV-01** | Sysfs Symlink Traversal & Host Arbitrary Reads | Symlink following in sysfs tree exposing out-of-tree sensitive files (`/etc/shadow`, `id_rsa`). | **HIGH** | Invariant **`NSERV5`**: Validate interface directory names (`NET1`); only open standard attribute filenames (`operstate`, `address`, `mtu`, `type`, `flags`) directly under the interface directory without following external paths. |
| **THREAT-NSERV-02** | Unbounded File Read / FIFO Hang (DoS) | A crafted FIFO or multi-GB file in sysfs/procfs causing process hang or heap exhaustion. | **HIGH** | Invariant **`NSERV6`**: Enforce strict read bounds (`MAX_SYSFS_FILE_BYTES = 64 KB`, `MAX_ROUTE_FILE_BYTES = 1 MB`, `MAX_RESOLV_FILE_BYTES = 64 KB`). |
| **THREAT-NSERV-03** | Route Table Hex Decoding Poisoning | Malformed hexadecimal lines in `/proc/net/route` causing integer overflow or parser crash. | **MEDIUM** | Invariant **`NSERV3`**: Use checked `u32::from_str_radix(..., 16)`, validate exact column structure, enforce `MAX_ROUTES = 4096`. |
| **THREAT-NSERV-04** | Resolv.conf Injection & DNS Spoofing | Control characters or invalid IP addresses in `resolv.conf` injected into network state. | **MEDIUM** | Invariant **`NSERV4`**: Validate each nameserver with `IpAddr::from_str`, sanitize search domains, cap at `MAX_DNS_NAMESERVERS = 32`. |
| **THREAT-NSERV-05** | Link Mutation Injection | Metacharacters in interface names executing commands or mutating arbitrary files. | **HIGH** | Invariant **`NSERV5`**: Enforce `validate_interface_name` on `bring_up` and `bring_down` calls, rejecting any path traversal or metacharacters. |

---

## 2. Hardening Recommendations for T-01818

1. **Add Bounded File Reading Helper**:
   - Implement `read_bounded_string(path: &Path, max_bytes: u64) -> Result<String, String>` using `File::open` and `take(max_bytes)`.
2. **Add Constants**:
   - `pub const MAX_SYSFS_FILE_BYTES: u64 = 64 * 1024;`
   - `pub const MAX_ROUTE_FILE_BYTES: u64 = 1024 * 1024;`
   - `pub const MAX_RESOLV_FILE_BYTES: u64 = 64 * 1024;`
3. **Verify with Automated Unit Tests**:
   - Add unit tests verifying oversized file rejection or truncation in `test_network_service.rs`.
