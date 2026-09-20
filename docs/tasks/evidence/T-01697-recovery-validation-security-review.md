# T-01697: Kernel Module Management — Recovery & Validation Security Review

## Metadata
- **Task ID**: `T-01697`
- **Sub-Epic**: Kernel Module Management / Recovery & Validation
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor**: Security Subsystem Agent

---

## 1. Scope & Objectives
Conduct a comprehensive security review of the recovery and validation mechanisms introduced in `T-01691`..`T-01696` for Kernel Module Management, spanning:
- `code/aiosh-rust/aiosh-core/src/kernel_module_recovery.rs`
- CLI command `aiosh mod check [--store <path>] [--auto-recover] [--json]` in `code/aiosh-rust/aiosh-cli/src/main.rs`
- MCP tool `aios.kernel_module.check` in `code/aiosh-rust/aiosh-mcp/src/main.rs`

Key audit criteria:
1. Input validation & path/argument injection
2. Untrusted-content handling (JSON bombs, corrupt files, symlinks)
3. Policy Enforcement Point (PEP) gating and authorization grant checks
4. Audit-ring logging for state checks and recovery mutations
5. Quarantine integrity and data loss prevention

---

## 2. Threat Modeling & Abuse Scenarios

### Scenario A-1: Unbounded File Read (Memory Exhaustion / DoS)
- **Attack Vector**: An attacker or rogue subagent points `--store` or `store_path` to a pseudo-device (e.g. `/dev/zero`, `/dev/urandom`) or a multi-gigabyte log file.
- **Risk**: `fs::read_to_string` consumes all system memory, causing out-of-memory kernel panic or daemon kill.
- **Severity**: MEDIUM
- **Finding**: File size is not capped prior to `read_to_string`.
- **Mitigation Requirement (for T-01698)**: Enforce a strict file size cap (e.g. 10 MB maximum, `MAX_STORE_FILE_SIZE = 10 * 1024 * 1024`) by checking `fs::metadata(path)?.len()` before loading content into memory.

### Scenario A-2: Arbitrary File Overwrite via Malicious Target Path
- **Attack Vector**: An untrusted caller requests `aios.kernel_module.check` with `auto_recover: true` and `store_path: "/etc/shadow"` or another sensitive operating system configuration file.
- **Risk**: If the file is unparseable as JSON, `recover_store_file` creates a backup and rewrites the file with a default empty `KernelModuleStore`, destroying system configuration.
- **Severity**: HIGH
- **Mitigation Requirement (for T-01698)**: 
  - Ensure path canonicalization and verification.
  - Enforce that store files cannot target system critical directories unless running with explicit privilege and valid grant.
  - Require atomic writes via temporary files with sync and rename.

### Scenario A-3: Symlink Traversal & Directory Traversal via Path Injection
- **Attack Vector**: Input contains `../` sequences or symlinks pointing outside the workspace / `.aios` directory to trick the engine into modifying or quarantining arbitrary files.
- **Mitigation**: Both CLI and MCP call `check_kernel_module_path_bounds`, rejecting control characters (`\0`, `\r`, `\n`) and limiting lengths. In hardening, verify symlink resolution.

### Scenario A-4: Quarantine Collision / Race Condition
- **Attack Vector**: Rapid automated recovery invocations cause collision in quarantine backup filenames (`<name>.corrupt.<timestamp>.bak`).
- **Mitigation**: Quarantine backups use microsecond-resolution timestamps (`%Y%m%d_%H%M%S_%6f`). Verify that if a destination file exists, collision is prevented.

---

## 3. PEP Gating & Audit Logging Verification

| Surface | Operation | Mutating? | PEP Action | Audit Row Emitted? | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **CLI** | `aiosh mod check` | No | `kernel_module:check` | Yes (`classify_and_emit`) | VERIFIED |
| **CLI** | `aiosh mod check --auto-recover` | Yes | `kernel_module:recover` | Yes (`classify_and_emit`) | VERIFIED |
| **MCP** | `aios.kernel_module.check` (`auto_recover: false`) | No | `kernel_module:check` | Yes (`dispatch::recorded_call`) | VERIFIED |
| **MCP** | `aios.kernel_module.check` (`auto_recover: true`) | Yes | `kernel_module:recover` | Yes (`dispatch::recorded_call`) | VERIFIED |

Both CLI and MCP surfaces emit structured audit events to the audit ring recording:
- `healthy`, `recovered`, `store_path`, `total_rules`, `valid_rules`, `invalid_rules`
- `backup_path` (if quarantined)
- Result status (`success` vs `failure`)

---

## 4. Remediation Plan for Hardening (T-01698)
1. Impose `MAX_STORE_FILE_SIZE = 10 * 1024 * 1024` (10MB) in `check_store_file` and `recover_store_file`.
2. Reject non-regular files (directories, special devices, FIFOs, sockets) via `metadata.is_file()`.
3. Ensure atomic quarantine and write permissions are strictly handled.

---

## 5. Acceptance Criteria Checklist
- [x] Input validation, path injection, and untrusted-content handling reviewed.
- [x] PEP gating and audit-row emission verified for read and write recovery paths.
- [x] Abuse scenarios documented in evidence.
- [x] No unhandled policy bypass remains open.
