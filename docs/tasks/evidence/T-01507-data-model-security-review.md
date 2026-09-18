# T-01507: Filesystem Layout - Data Model: Security Review

## Metadata
- **Task ID:** `T-01507`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout (`aiosh-core::fs_layout`, `aiosh-cli`, `aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (7/10) — Security Review
- **Dependencies:** `T-01506` (Integration)
- **Next Task:** `T-01508` (Hardening)

---

## 1. Executive Summary & Security Assessment

A comprehensive security review of the Filesystem Layout data model and validation engine was conducted across `aiosh-core`, `aiosh-cli`, and `aiosh-mcp`. The review verified input validation, injection resistance, memory bounds, path sanitization, CIS benchmark compliance, and tamper-evident audit logging.

### Security Posture Summary
- **Input Sanitization**: All mount points, device specifiers, mount options, directory paths, and partition labels are strictly bounded and validated.
- **Injection Prevention**: Whitespace and control character injection (`\0`, `\n`, `\r`, `\t`, ANSI escapes) are rejected on all `device` and `options` fields, preventing `/etc/fstab` line or column tampering.
- **Path Hygiene (FL2)**: Absolute path enforcement (`/`), traversal prohibition (`.` and `..`), and empty segment elimination (`//`) prevent path confusion attacks.
- **CIS Hardening (FL4)**: Automated policy enforcement guarantees `nodev` and `nosuid` on shared and temporary directories (`/tmp`, `/dev/shm`).
- **Audit Non-Repudiation**: Every invocation across CLI and MCP surfaces writes an immutable audit record into the SHA-256 hash-chained SQLite WAL ring.

---

## 2. Abuse Scenario Analysis (AS-1 .. AS-6)

### AS-1: Fstab Line & Option Injection
- **Threat Vector**: An attacker supplies a crafted mount specification containing newline characters or spaces in `device` or `options` (e.g., `device = "UUID=xxx\n/dev/sda1 /mnt ext4 rw 0 0"`).
- **Impact**: Arbitrary `/etc/fstab` corruption allowing unauthorized mounts or root filesystem remounts.
- **Mitigation & Verification**:
  - `validate_mount_point` explicitly checks `spec.device.chars().any(|c| c.is_control() || c.is_whitespace())` and returns an immediate error if found.
  - Every mount option in `spec.options` is similarly checked for control characters and whitespace.
  - Device string length is bounded to 256 characters.

### AS-2: Directory Traversal via Malformed Mount/Directory Paths
- **Threat Vector**: An attacker attempts to break out of layout containment by supplying relative paths or traversal sequences (`/var/lib/aios/../../../etc/cron.d`).
- **Impact**: Overwriting or modifying sensitive system directories outside the intended layout scope.
- **Mitigation & Verification**:
  - `validate_path_hygiene` (invariant `FL2`) mandates paths start with `/`, enforces max length of 1,024 characters, and splits the path by `/` checking that no segment equals `.` or `..`.
  - Repeated slashes (`//`) are explicitly rejected.

### AS-3: Unbounded Partition Allocation (Denial of Service)
- **Threat Vector**: Supplying an excessive number of partitions or oversized partition capacity to exhaust system resources or trigger kernel GPT parsing faults.
- **Impact**: Disk partition table corruption, out-of-memory during layout processing, or exhaustion of available block storage.
- **Mitigation & Verification**:
  - Invariant `FL5` limits partition index to $1 \le \text{index} \le 128$.
  - Partition indices must be unique.
  - Total cumulative partition capacity is checked against `target_disk_min_bytes / (1024 * 1024)`; if partition sizes exceed the budget, validation fails immediately.

### AS-4: Execution from Insecure Temporary Filesystems (CIS Benchmark Evasion)
- **Threat Vector**: Malware placed into world-writable temporary directories (`/tmp` or `/dev/shm`) executed directly.
- **Impact**: Local privilege escalation or persistence.
- **Mitigation & Verification**:
  - Invariant `FL4` verifies that mount point specifications for `/tmp` and `/dev/shm` contain `nodev` and `nosuid`.
  - Canonical `standard_uefi()` layout additionally includes `noexec`.
  - Any configuration omitting these security flags fails validation.

### AS-5: ESP Filesystem Type Confusion
- **Threat Vector**: Creating an EFI System Partition formatted with non-FAT32 filesystems (e.g. `ext4`), rendering the system unbootable by UEFI firmware.
- **Impact**: Target system boot failure.
- **Mitigation & Verification**:
  - `validate_partition_spec` enforces `FsType::Vfat` on any partition marked `PartitionType::EfiSystem`.
  - `validate_filesystem_layout` checks that if an ESP partition exists, `/boot/efi` mount point is present and uses `vfat`.

### AS-6: Audit Trail Bypass & PEP Non-Repudiation
- **Threat Vector**: Invoking filesystem layout inspection or validation without recording operational telemetry.
- **Impact**: Covert reconnaissance by malicious agents.
- **Mitigation & Verification**:
  - All CLI subcommands (`aiosh layout <show|validate|fstab|check>`) call `classify_and_emit` emitting structured audit records with subsystem `"fs_layout"`.
  - All MCP tool calls (`aios.fs_layout.get`, `validate`, `fstab`) route through `dispatch::recorded_call`, preserving caller identity, timestamp, arguments, and outcome.

---

## 3. Residual Risk & Recommendations

1. **Partition Application Gate**: The current tasks implement the in-memory data model and validators. When physical disk partitioning and formatting tools (`parted`, `mkfs.ext4`, `mkfs.vfat`) are implemented in subsequent sub-epics, they MUST be PEP-gated with mandatory explicit grants (`require_grant: true`) due to irreversible data loss risks.
2. **Audit Parity**: All read-only queries are classified with appropriate rule pack tags, and no policy bypass remains open.
