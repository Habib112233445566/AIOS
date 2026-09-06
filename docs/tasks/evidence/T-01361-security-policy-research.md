# T-01361: Init & Service Supervision - Security Policy: Research

## Metadata
- **Task ID:** `T-01361`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Security Policy
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Research Overview & Prior Art
The AIOS system architecture requires that service supervision and daemon initialization be strictly governed by security policies to prevent the execution of untrusted, unconfined, or insecure services within the operating system target. Because services run persistently with elevated permissions and interact directly with IPC, filesystems, and network stacks, enforcing declarative security policies is vital to operating system integrity.

### Authoritative Upstream Standards
1. **systemd Security Directives (`systemd.exec(5)`, `systemd.service(5)`)**:
   - Upstream security hardening features:
     - `NoNewPrivileges=yes`: Disallows processes from acquiring additional privileges via setuid/setgid or file capabilities.
     - `ProtectSystem=strict|full|yes`: Mounts system paths (`/usr`, `/boot`, `/etc`) read-only for unit processes.
     - `ProtectHome=yes|read-only|tmpfs`: Isolates `/home`, `/root`, and `/run/user`.
     - `PrivateTmp=yes`: Creates private `/tmp` and `/var/tmp` mount namespaces per service instance.
     - `CapabilityBoundingSet=`: Enforces capability dropping, disallowing unneeded Linux capabilities (e.g. `CAP_SYS_ADMIN`, `CAP_NET_RAW`, `CAP_SYS_RAWIO`).
     - `ProtectKernelTunables=yes`, `ProtectControlGroups=yes`: Blocks unauthorized writes to `/proc/sys`, `/sys`, and cgroup hierarchies.
     - `RestrictAddressFamilies=`: Limits network socket domains (e.g., restricting raw sockets or obsolete protocol families).
2. **CIS Linux Benchmarks (Sections 2 & 3: Services, Daemons, and Network Configurations)**:
   - Prohibited / legacy services to be banned or masked by default:
     - `telnet.service`, `rsh.service`, `rlogin.service`, `rexec.service`, `tftp.service`, `vsftpd.service`, `xinetd.service`, `ypserv.service`, `ypbind.service`, `avahi-daemon.service`, `cups.service`.
   - Privilege minimization:
     - Daemon processes must not run as `root` unless explicitly authorized and documented.
     - Services must not execute binaries or scripts residing in world-writable or transient directories (`/tmp`, `/var/tmp`, `/dev/shm`).
     - Path hygiene: Working directories and binary paths must be absolute paths free of path traversal sequences (`..`).
3. **OpenRC Security Architecture (`openrc-run(8)`)**:
   - Daemon confinement via `--chuid` / `--user`, sandboxed root directories, and resource limits (`rc_ulimit`).
4. **NIST SP 800-53 (AC-6 Least Privilege, CM-7 Least Functionality) & NIST SP 800-123**:
   - Restricting background service privileges, disabling non-essential services, and isolating daemon environments.
5. **AIOS Security Architecture (ADR-0035 & AI Constitution)**:
   - Tri-state policy enforcement modes (`Enforcing`, `Audit`, `Permissive`).
   - Integration with PEP (Policy Enforcement Point) and cryptographic audit row emission.
   - Bounded sizes, safe parsing, and non-blocking audit logging.

---

## 2. Facts vs. Assumptions

### Established Facts
1. **Fact**: `ServiceSpec` in `code/aiosh-rust/aiosh-core/src/service.rs` defines complete metadata for services: `name`, `exec_start`, `exec_stop`, `exec_reload`, `service_type`, `restart_policy`, `startup_mode`, `user`, `group`, `working_dir`, `environment`, `dependencies`, `timeout_start_secs`, and `timeout_stop_secs`.
2. **Fact**: `ServiceStore` in `code/aiosh-rust/aiosh-core/src/service_service.rs` manages service registration, persistence, and transactional execution, but currently lacks a dedicated security policy engine.
3. **Fact**: `PackageSecurityPolicy` (`package_policy.rs`), `BaseImageSecurityPolicy` (`base_image_policy.rs`), and `DistroSecurityPolicy` (`distro_policy.rs`) establish a well-defined architectural convention:
   - A policy struct with a `mode: ServicePolicyMode` (`Enforcing`, `Audit`, `Permissive`).
   - Invariant validation (`validate()`).
   - Fine-grained violation tracking (`ServicePolicyViolation`).
   - Evaluation functions for single specs (`evaluate_spec`), lists, and stores (`evaluate_store`).
   - File loading with size caps (`from_file`), environment variable loading (`from_env`), and priority resolution (`resolve`).
4. **Fact**: No source code was modified during this research task.

### Assumptions
1. **Assumption**: `ServiceSecurityPolicy` will be implemented in `code/aiosh-rust/aiosh-core/src/service_policy.rs` and re-exported via `lib.rs`.
2. **Assumption**: Prohibited services by default will ban insecure legacy protocols (`telnet.service`, `rsh.service`, `rlogin.service`, `rexec.service`, `tftp.service`, `xinetd.service`, `ypserv.service`, `ypbind.service`).
3. **Assumption**: Security policy evaluation will inspect:
   - Service name against prohibited service patterns and control character rules.
   - Binary paths in `exec_start`, `exec_stop`, and `exec_reload` against prohibited directories (`/tmp`, `/var/tmp`, `/dev/shm`).
   - User identity (e.g., flag or rule disallowing unexempted root execution when `disallow_root = true`).
   - Environment variables (disallowing sensitive variables like `LD_PRELOAD`, `LD_LIBRARY_PATH` unless explicitly permitted).
   - Working directory paths (must be absolute, no traversals).
4. **Assumption**: The policy will provide configuration bounds (`SP1`), prohibited service blocking (`SP2`), execution binary path safety (`SP3`), privilege / user hygiene (`SP4`), environment & parameter hygiene (`SP5`), and tri-state enforcement semantics (`SP6`).

---

## 3. Unknowns & Decisions Needed

1. **Decision**: What policy invariants (`SP1..SP6`) should govern Service Supervision security policy?
   - `SP1`: Policy configuration validation (valid modes, bounds on lists $\le 1024$, string length bounds $\le 1024$).
   - `SP2`: Prohibited services blocking (blocking known insecure service names).
   - `SP3`: Executable path hygiene (banning execution from `/tmp/`, `/var/tmp/`, `/dev/shm/`, relative paths, or containing path traversal).
   - `SP4`: User privilege and root restriction (checking `user` field, enforcing unprivileged execution or explicit root allowlist).
   - `SP5`: Environment variable injection prevention (banning `LD_PRELOAD`, `LD_LIBRARY_PATH`, control chars in keys).
   - `SP6`: Tri-state policy modes (`Enforcing` rejects fatal violations; `Audit` logs violations without rejection; `Permissive` warns on non-prohibited violations).
2. **Decision**: CLI and MCP Surface:
   - CLI command: `aiosh service policy [--check] [--json] [--config <path>]`.
   - MCP tool: `aios.service.policy`.
   - Integration test criterion: `SS7` in `tools/test_service_suites.py`.
