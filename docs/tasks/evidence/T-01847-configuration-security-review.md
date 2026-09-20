# Task Evidence: T-01847 - Network Bootstrap / configuration: Security Review

## 1. Overview
- **Task ID**: `T-01847`
- **Sub-Epic**: 5 (Network Bootstrap Configuration)
- **Goal**: Perform comprehensive security review and threat analysis of the `NetworkConfig` subsystem in `aiosh-core`.

---

## 2. Threat Modeling & Abuse Scenarios

### `THREAT-NCONF-01`: Path Traversal & Arbitrary File Overwrite
- **Scenario**: An attacker provides malicious paths containing `../` or absolute symlink paths in `default_store_path`, `sysfs_net_path`, `procfs_path`, or `resolv_conf_path`.
- **Impact**: Reading arbitrary system files or overwriting sensitive system configuration files (`/etc/shadow`, `/etc/sudoers`) during save operations.
- **Evaluation**:
  - Invariant `NCONF1` enforces path validation:
    - Must be non-empty and valid UTF-8.
    - Path length capped at 1024 characters.
    - Rejects ASCII control characters (0..31 and 127) and null bytes (`\0`).
    - Explicitly checks `path.components().any(|c| matches!(c, std::path::Component::ParentDir))` and rejects parent directory traversal (`..`).

### `THREAT-NCONF-02`: Resource Exhaustion via Unbounded Collections
- **Scenario**: An attacker configures enormous collection limits (`max_interfaces = 1_000_000`, `max_routes = 10_000_000`, `max_dns_servers = 100_000`).
- **Impact**: Out-of-memory (OOM) crash or excessive CPU consumption during interface discovery and route table parsing.
- **Evaluation**:
  - Invariant `NCONF2` caps capacities:
    - `1 <= max_interfaces <= 10,000`
    - `1 <= max_routes <= 50,000`
    - `1 <= max_dns_servers <= 64`

### `THREAT-NCONF-03`: Disk & Memory Exhaustion via Oversized Configuration Documents
- **Scenario**: An attacker places a multi-gigabyte corrupted JSON file at the configuration path, causing high memory allocation during `load_from_path`.
- **Impact**: DoS through memory allocation failure.
- **Evaluation**:
  - `load_from_path` inspects `fs::metadata(path).len()` prior to reading, enforcing `MAX_CONFIG_FILE_BYTES = 1,048,576` (1 MB limit). Files exceeding this threshold are immediately rejected before reading into memory.
  - Invariant `NCONF3` constrains `max_payload_bytes` to `1024..=104_857_600` (1 KB to 100 MB).

### `THREAT-NCONF-04`: DNS Injection / Hijacking via Malformed Nameservers
- **Scenario**: Fallback DNS servers contain malicious hostnames, domain names with shell escape sequences, or unroutable addresses.
- **Impact**: DNS resolution poisoning, command injection if passed unsanitized to external scripts.
- **Evaluation**:
  - Invariant `NCONF4` strictly parses all fallback DNS servers using `std::net::IpAddr`.
  - Non-IP strings or empty entries are rejected at validation time.

### `THREAT-NCONF-05`: State Corruption via Partial Writes
- **Scenario**: Process crash or system power loss during configuration write leaves truncated or malformed JSON on disk.
- **Impact**: Irrecoverable configuration state on subsequent boots.
- **Evaluation**:
  - Invariant `NCONF6` uses an atomic write pattern:
    - Writes to sibling temporary file `.{name}.tmp.{pid}` in the parent directory.
    - Renames temporary file to destination path atomically using filesystem `rename`.

### `THREAT-NCONF-06`: Insecure Environment Variable Ingestion
- **Scenario**: Host environment variables contain malicious inputs to bypass validation.
- **Impact**: Unvalidated configuration loaded into runtime.
- **Evaluation**:
  - Invariant `NCONF5` implements safe bounds parsing in `from_env()`.
  - A post-validation guard checks `cfg.validate()`. If any overridden property causes validation failure, `from_env()` safely reverts to `NetworkConfig::default()`.

---

## 3. Security Findings & Recommendations for Hardening (`T-01848`)
1. **Temporary File Cleanup on Write Failure**:
   - In `save_to_path()`, if `fs::write()` fails, ensure the temporary file is explicitly deleted so no orphan `.tmp.*` files remain on disk.
2. **Atomic Permissions (Unix)**:
   - On Unix systems, ensure temporary files are created with restricted permissions (`0600`) to prevent local unauthorized access to intermediate configuration data.
3. **Explicit Error Categorization**:
   - Wrap filesystem errors with contextual prefixes (`NCONF_IO_ERROR`, `NCONF_VALIDATION_ERROR`) for clear audit logs.
