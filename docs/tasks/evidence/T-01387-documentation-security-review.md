# T-01387: Init & Service Supervision Documentation Security Review

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Documentation  
**Task ID:** T-01387  

---

## 1. Security Review & Threat Modeling

### A. Secret & Credential Leakage Audit
- Scanned `docs/service_supervision.md` for exposed credentials, passwords, cryptographic keys, and internal secrets.
- **Finding**: Zero private credentials or sensitive tokens present. All examples use standard reference services (`aios-securityd.service`, `auditd.service`, `dbus.service`, `ssh.service`) or canonical prohibited test fixtures (`telnet.service`).

### B. Command & Argument Injection Surface
- Reviewed all documented CLI commands (`aiosh service *`) and MCP tools (`aios.service.*`).
- Verified that syntax examples adhere strictly to safe syntax:
  - Positional arguments (`name`, `action`) and option flags (`--state`, `--mode`, `--store`, `--config`, `--policy`, `--spec`) do not employ unsafe shell interpolation.
  - Rust implementations in `aiosh-cli` and `aiosh-mcp` enforce length bounds (128 chars on service names, 1024 on paths) and immediately reject ASCII control characters with structured `INVALID_ARGUMENT` envelopes.

---

## 2. Abuse Scenarios & Mitigations

### Scenario 1: Prohibited Daemon Activation & Evasion
- **Threat**: An adversary attempts to activate prohibited insecure legacy network services (`telnet.service`, `rsh.service`, `rlogin.service`, `rexec.service`, `tftp.service`, `xinetd.service`, `ypserv.service`, `ypbind.service`) by alternating casing or stripping suffixes.
- **Mitigation**: Security policy invariant `SP2` strips suffixes and converts identifiers to lowercase before checking against `prohibited_services`. In `Enforcing` mode, activation is unconditionally denied and recorded in the audit trail.

### Scenario 2: Binary Path Traversal & Temporary Directory Execution
- **Threat**: An attacker supplies relative execution paths (e.g. `bin/daemon`), directory traversal sequences (`..`), or binaries residing within world-writable directories (`/tmp`, `/var/tmp`, `/dev/shm`, `/run/user`) in `exec_start` or `working_dir`.
- **Mitigation**: Invariants `SS2` and `SP3` enforce absolute paths, reject `..`, and actively disallow prohibited path prefixes, emitting fatal violation `SP3-PROHIBITED-PATH`.

### Scenario 3: Unauthorized Root Privilege Escalation
- **Threat**: A service specification omits a user account or specifies `user = "root"` to escalate privileges upon service activation.
- **Mitigation**: Security policy invariant `SP4` enforces `require_service_user` and `disallow_root`. Only explicitly whitelisted system daemons in `allowed_root_services` (e.g. `systemd-journald.service`, `aios-securityd.service`) are permitted root execution.

### Scenario 4: Dynamic Linker & Environment Variable Hijacking
- **Threat**: Attackers inject environment variables such as `LD_PRELOAD`, `LD_LIBRARY_PATH`, or `IFS` into service definitions to hijack execution flow.
- **Mitigation**: Invariant `SP5` scans all keys in `spec.environment` against `disallow_env_vars` and enforces bounds $\le 1024$ variables, blocking linker hijacking.

### Scenario 5: Cyclic Dependency Deadlocks in Startup Ordering
- **Threat**: An adversary defines circular service dependencies (e.g. A requires B, B requires A) to hang system boot sequencing.
- **Mitigation**: Invariant `CS3` executes Kahn's algorithm over the dependency Directed Acyclic Graph (DAG) with linear cycle detection, rejecting circular definitions with explicit error `ORDER_FAILED`.

### Scenario 6: Unlogged State Mutations & Stealth Actions
- **Threat**: A rogue agent attempts to modify service definitions or execute lifecycle actions without forensic traceability.
- **Mitigation**: All CLI mutations write audit events to `audit.db` via `classify_and_emit`. All autonomous agent MCP tool calls are gated through `dispatch::recorded_call`, enforcing PEP capability verification and writing immutable SHA-256 hash-chained entries to the SQLite WAL ring.

---

## 3. Review Conclusion
Zero policy bypasses remain open. The documentation in `docs/service_supervision.md` accurately describes implemented security controls, boundary invariants, and audit guarantees.
