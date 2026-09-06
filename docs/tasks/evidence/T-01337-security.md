# T-01337: Init & Service Supervision - MCP/API Surface: Security Review

## Metadata
- **Task ID:** `T-01337`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface Security Review
- **Status:** Complete (Security Review)

---

## 1. Threat Model & Security Posture
The Model Context Protocol (MCP) server is the primary programmatic interface between AI agents/orchestrators and host OS management functions. In Init & Service Supervision, unauthorized or unvalidated actions could destabilize system services, terminate essential security daemons, or execute unauthorized binaries.

---

## 2. Abuse Scenarios & Defensive Mitigations

### Abuse Scenario 1: Path Traversal via `store_path`
- **Threat**: An agent or adversary specifies a path like `../../../../etc/aios/critical.json` or `/dev/null` in the `store_path` parameter to overwrite system state or cause corruption.
- **Analysis & Mitigation**:
  - `store_path` is constrained: max length 1024 bytes, checked for control characters (`c.is_control()`).
  - Read operations strictly parse valid JSON matching `ServiceStore` schema (`services` and `statuses`).
  - Write operations (`save_to_path`) write to a PID-tagged temporary file in the target directory, fsync, and atomically rename.
  - No shell interpolation or unbounded file permissions.

### Abuse Scenario 2: Log Injection & Terminal Hijacking via Control Characters
- **Threat**: An adversary provides a service name or search pattern with embedded control characters (e.g., `\r\n`, `\x00`, ANSI escape codes) to manipulate audit logs or corrupt operator terminals.
- **Analysis & Mitigation**:
  - All string inputs (`name`, `pattern`, `store_path`) validate `!c.is_control()`.
  - Service names must additionally satisfy the strict `SS1` charset: ASCII alphanumeric, underscores, hyphens, dots, and must start with an alphanumeric character.
  - Injection attempts return immediate validation errors before execution or audit classification.

### Abuse Scenario 3: Administrative State Machine Bypass (Masking Invariant)
- **Threat**: An attacker attempts to execute `start`, `restart`, or `enable` on a service masked by security policy to activate a vulnerable or decommissioned service.
- **Analysis & Mitigation**:
  - `ServiceStore::execute_action` enforces that `ServiceStartupMode::Masked` rejects `Start`, `Restart`, `Reload`, and `Enable` actions with explicit errors.
  - Services can only be masked when inactive (`status.state != Active`), preventing ungraceful orphan processes.
  - Masking status cannot be circumvented without an explicit `unmask` action.

### Abuse Scenario 4: Dependency Graph Poisoning & DoS via Cycles
- **Threat**: Submitting an invalid or malicious dependency topology (cycles, self-dependencies, massive fan-out) to trigger infinite loops, stack overflows, or deadlock during startup ordering.
- **Analysis & Mitigation**:
  - `validate_service_spec` enforces `SS5` (no self-dependencies) and rejects duplicate dependencies.
  - `plan_service_order` implements Kahn's algorithm (topological sorting using in-degree counts).
  - Graph traversal runs in bounded linear time `O(V + E)`. If a cycle is detected, traversal immediately halts and returns a descriptive error rather than looping.

### Abuse Scenario 5: Unauthorized Action & Audit Evasion
- **Threat**: An unprivileged agent triggers service state transitions without an authorized capability token or bypasses audit logging.
- **Analysis & Mitigation**:
  - All tools pass through `dispatch::recorded_call`.
  - PEP gating evaluates the actor's `grant_id`.
  - Every tool execution (both permitted and rejected) writes an immutable record to `AuditRing` with tool name, arguments, actor, and result status, guaranteeing ADR-0035 compliance and non-repudiation.

---

## 3. Vulnerability Findings Summary
- **Critical Issues:** 0
- **High Issues:** 0
- **Medium Issues:** 0
- **Low Issues:** 0
- **Policy Bypasses:** None found.
- **Verdict:** PASS.
