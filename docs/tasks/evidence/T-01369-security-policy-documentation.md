# T-01369: Init & Service Supervision / Security Policy - Documentation

## Metadata
- **Task ID:** `T-01369`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Security Policy
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Overview & Shipped Artifacts
This task establishes operational and architectural documentation for the Init & Service Supervision Security Policy subsystem across `docs/README.md` (§8.13), CLI command interfaces, and MCP tools.

### Shipped Features
- **Security Policy Invariants (`SP1..SP6`)**:
  - `SP1`: Policy configuration validation ($\le 1024$ prohibited services, $\le 128$ paths, $[1..86400]$s timeouts).
  - `SP2`: Prohibited services blocking (blocking `telnet.service`, `rsh.service`, `rlogin.service`, `rexec.service`, `tftp.service`, `xinetd.service`, `ypserv.service`, `ypbind.service`).
  - `SP3`: Executable path hygiene (prohibiting `/tmp`, `/var/tmp`, `/dev/shm`, relative paths, and directory traversal `..`).
  - `SP4`: User privilege & root hygiene (`require_service_user`, `disallow_root`, `allowed_root_services` exemption list).
  - `SP5`: Environment & parameter sanitization (blocking `LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`, bounding env count $\le 1024$, timeouts).
  - `SP6`: Tri-state policy modes (`Enforcing`, `Audit`, `Permissive`) with SQLite WAL audit logging.

---

## 2. Copy-Pasteable Invocation Examples

### 1. Operator CLI Usage

#### A. Inspecting Active Security Policy
```bash
aiosh service policy
```
*Output:*
```text
AIOS Init & Service Supervision Security Policy:
  Mode:                       Enforcing
  Require Service User:       false
  Disallow Root:              false
  Max Environment Variables:  256
  Max Timeout (s):            3600s
  Prohibited Services:        telnet.service, rsh.service, rlogin.service, rexec.service, tftp.service, xinetd.service, ypserv.service, ypbind.service
  Prohibited Exec Paths:      /tmp, /var/tmp, /dev/shm, /run/user
  Disallowed Env Vars:        LD_PRELOAD, LD_LIBRARY_PATH, IFS
  Allowed Root Services:      systemd-journald.service, aios-securityd.service
```

#### B. Evaluating a Specific Service Unit
```bash
aiosh service policy --service ssh.service
```
*Output:*
```text
Service Security Policy Verdict for 'ssh.service':
  Allowed:    true
  Mode:       Enforcing
  Violations: 0
```

#### C. Evaluating in Structured JSON
```bash
aiosh service policy --service telnet.service --json
```
*Output:*
```json
{
  "code": 1,
  "data": {
    "allowed": false,
    "evaluated_at": "2026-09-06T00:00:00Z",
    "mode": "enforcing",
    "service_name": "telnet.service",
    "violations": [
      {
        "description": "Service 'telnet.service' is prohibited by security policy",
        "fatal": true,
        "rule_id": "SP2-PROHIBITED-SERVICE",
        "service_name": "telnet.service"
      }
    ]
  },
  "error": null
}
```

### 2. Autonomous Agent MCP Tool Calls

#### Inspecting Policy
```json
{
  "tool": "aios.service.policy",
  "arguments": {}
}
```

#### Evaluating Target Service
```json
{
  "tool": "aios.service.policy",
  "arguments": {
    "service_name": "telnet.service"
  }
}
```

---

## 3. Constraints & Known Limitations
1. **Root Execution Default**: To maintain compatibility with standard base images, `disallow_root` and `require_service_user` are `false` by default, but can be enabled via policy file (`disallow_root: true`) or environment variable `AIOS_SERVICE_DISALLOW_ROOT=1`.
2. **Path Resolution**: Relative binaries in `exec_start` without leading `/` or Windows drive identifiers are treated as violations (`SP3-RELATIVE-PATH`). All service definitions must specify absolute binary paths.
3. **Configuration File Cap**: Custom policy configuration files are strictly capped at 64 KiB (`MAX_POLICY_FILE_BYTES = 65_536`). Larger files are rejected with an explicit error to prevent denial-of-service.
