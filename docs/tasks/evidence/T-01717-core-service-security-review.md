# T-01717: Hardware Detection — Core Service Security Review

## Metadata
- **Task ID**: `T-01717`
- **Sub-Epic**: Hardware Detection / Core Service
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Threat Modeling & Attack Surfaces

The Hardware Detection Core Service (`HardwareService` in `code/aiosh-rust/aiosh-core/src/hardware_service.rs`) interacts with the host or mock filesystem (`/sys` and `/proc`) to construct a system inventory. The following potential abuse vectors and threat scenarios were evaluated:

### Scenario CS-1: Unbounded In-Memory Buffering on sysfs Virtual Files
- **Hazard**: Linux sysfs and procfs files do not report accurate byte lengths via `stat.st_size` (often reporting 0 or 4096). Using `fs::read_to_string(path)` relies on EOF. If an adversary symlinks or mounts a streaming or infinite pseudo-file (e.g. `/dev/urandom` or circular procfs node), `read_to_string` could allocate memory without bound, triggering an out-of-memory (OOM) panic and denial of service.
- **Remediation**: Use `Read::take(MAX_READ_LEN)` to bound filesystem reads at the OS buffer level rather than reading the entire stream into memory before truncating.

### Scenario CS-2: Control Character & Null Byte Injection in sysfs Attributes
- **Hazard**: sysfs attributes (such as device names, serials, or vendor strings) could contain null bytes (`\0`) or ANSI control sequences (`\x1b`, `\x07`) designed to corrupt downstream terminal formatters, audit logs, or JSON parsers.
- **Remediation**: Sanitize read strings in `read_trimmed_file` by filtering out non-printable ASCII control characters.

### Scenario CS-3: Driver Identifier Spoofing & Traversal
- **Hazard**: Symlinks in `/sys/bus/*/devices/*/driver` point to driver directories. If a malicious symlink points outside the driver tree (e.g. `../../../../etc`), extracting `target.file_name()` could yield unintended values if not sanitized.
- **Remediation**: Enforce character whitelisting on driver names (`[a-zA-Z0-9_-]+`) and bound driver length to `MAX_ATTRIBUTE_VAL_LEN`.

### Scenario CS-4: Directory Traversal via Custom Mock Roots
- **Hazard**: When using `HardwareService::with_roots(sysfs, procfs)`, relative or untrusted paths containing `..` could allow scanning sensitive system directories outside intended roots.
- **Remediation**: Validate that custom roots are valid paths and reject paths containing null bytes or illegal characters.

---

## 2. Hardening Requirements for T-01718
1. **Bounded I/O in `read_trimmed_file`**:
   - Limit stream reading using `std::io::Read::take(1024)`.
   - Strip ASCII control characters (`c.is_ascii_control() && c != '\t' && c != '\n'`).
2. **Driver Identifier Validation**:
   - Enforce character whitelist on resolved driver names: only alphanumeric, underscore, and hyphen characters permitted.
3. **Safe File Opening & Symlink Boundary**:
   - Ensure non-regular or oversized files fail safely without panics.
4. **Verification Tests**:
   - Author negative unit tests covering bounded reads and control character sanitization in `test_hardware_service.rs`.
