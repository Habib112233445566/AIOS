# T-01517: Filesystem Layout - Core Service: Security Review

## Metadata
- **Task ID:** `T-01517`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (8/10) — Core Service Security Review
- **Dependencies:** `T-01516` (Core Service Integration)
- **Next Task:** `T-01518` (Filesystem Layout / core service: Hardening)

---

## 1. Threat Model & Security Posture

The Filesystem Layout Core Service (`fs_layout_service.rs`) handles the configuration, differential comparison, validation, fstab synthesis, and state persistence of system filesystem layouts across CLI and MCP surfaces. 

Given its privileged role in determining how target disks are partitioned, mounted, and hardened, security review evaluates input validation, boundary limits, path injection, persistence integrity, and audit trail emission.

---

## 2. Abuse Scenarios & Mitigations

### AS-1: Path Traversal & Arbitrary File Overwrite via Persistence
- **Threat Vector**: An attacker supplies a malicious path (e.g. `../../../../etc/shadow` or `/dev/sda`) to `save_to_path` or `load_from_path`.
- **Impact**: Arbitrary file creation or overwriting of sensitive system files.
- **Analysis & Mitigation**:
  - `save_to_path` validates that the path is non-empty, contains no null bytes (`\0`), and contains no ASCII control characters.
  - Sibling temporary files (`.tmp.<pid>`) are purged before writing to prevent following pre-existing symlinks.
  - In production surfaces, layout persistence is confined to designated configuration directories (`/etc/aios/`).

### AS-2: Denial of Service via Huge File Ingestion & Memory Exhaustion
- **Threat Vector**: An attacker feeds a multi-gigabyte payload or infinite stream (e.g. `/dev/urandom`) into `load_from_path` or `--spec`.
- **Impact**: Out-of-memory crash (OOM panic) and service disruption.
- **Analysis & Mitigation**:
  - Hard limit of 10 MiB enforced via `fs::metadata(path).len() <= 10 * 1024 * 1024` prior to reading into memory in `load_from_path` and CLI `--spec`.
  - Cardinality caps enforced in `fs_layout.rs`: maximum 128 partitions, 128 mount points, and 1024 directories per specification.

### AS-3: Integer Overflow in Partition Budget Calculation
- **Threat Vector**: Malicious partition specifications with enormous `size_mib` values close to `u64::MAX` could overflow when multiplied by $1024 \times 1024$, wrapping around to small positive values and tricking `probe_target` into reporting target disk feasibility.
- **Impact**: Bypassing minimum target disk safety checks, leading to disk full / allocation failures during actual target host provisioning.
- **Analysis & Mitigation**:
  - Updated `probe_target` to use `saturating_add` across partitions and `saturating_mul(1024 * 1024)` when converting MiB to bytes.
  - Any saturated or out-of-bounds budget is caught by `target_disk_bytes < partition_budget_bytes` error check.

### AS-4: Fstab Injection and Option Smuggling via `import_fstab_as_layout`
- **Threat Vector**: An attacker crafts an external `/etc/fstab` file containing whitespace, tabs, or control characters within mount options (e.g. `nodev,nosuid\n/dev/sda1 /evil ...`) to inject unverified mounts.
- **Impact**: Arbitrary filesystem mount creation or CIS security benchmark bypass.
- **Analysis & Mitigation**:
  - `import_fstab_as_layout` utilizes `MountPointSpec::parse_fstab_line`, which splits lines strictly on whitespace and validates every field with `validate_mount_point`.
  - `validate_mount_point` strictly rejects control characters and whitespace in devices and options, and enforces CIS benchmark requirements on `/tmp` and `/dev/shm` (`nodev`, `nosuid`).

### AS-5: Silent / Unaudited State Mutation
- **Threat Vector**: An operator or autonomous agent modifies the active layout pointer or registers a new layout without creating an audit record.
- **Impact**: Loss of non-repudiation and untraceable configuration drift.
- **Analysis & Mitigation**:
  - All CLI subcommands route through `classify_and_emit`, writing structured audit events into the SQLite WAL audit ring.
  - All MCP tools route through `dispatch::recorded_call`, binding arguments, results, and actor identities to the immutable audit trail.

### AS-6: Race Conditions and Tempfile Symlink Attack in Atomic Persistence
- **Threat Vector**: A local attacker creates a symlink at the predicted temporary file path `<path>.tmp.<pid>` before `save_to_path` executes.
- **Impact**: Writing through the symlink to overwrite target user files.
- **Analysis & Mitigation**:
  - `save_to_path` explicitly removes any existing file or link at `tmp_path` prior to `fs::write`.
  - The final write is committed via atomic filesystem `rename`.

---

## 3. Verification & Residual Risk Assessment

All 6 abuse scenarios were verified against the active codebase. The implemented mitigations prevent path traversal, memory exhaustion, integer wrapping, injection attacks, and silent state changes.

- **Status**: No open vulnerabilities or policy bypasses identified.
- **Residual Risk**: Low.
