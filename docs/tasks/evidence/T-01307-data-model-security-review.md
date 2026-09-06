# T-01307: Init & Service Supervision - Data Model: Security Review

## Metadata
- **Task ID:** `T-01307`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service`
- **Component:** Init & Service Supervision Data Model Security Review
- **Status:** Complete

## 1. Threat Modeling & Abuse Scenarios

### Abuse Scenario 1: Shell Metacharacter & Path Traversal Injection in Service Names
- **Attack Vector:** An untrusted caller (agent or script) provides a malicious service name with directory traversal or shell control metacharacters (e.g., `../../etc/shadow`, `systemd-journald; reboot`, `foo$(id).service`, `service\0.socket`).
- **Mitigation:** Invariant `SS1` strictly enforced in `validate_service_name`:
  - Name length constrained to $[1 \dots 128]$ characters.
  - Allowed characters strictly limited to ASCII alphanumeric `[a-zA-Z0-9]`, underscores `_`, hyphens `-`, and dots `.`.
  - Leading character must be alphanumeric `[a-zA-Z0-9]`.
  - Slashes, backslashes, quotes, whitespace, control characters, null bytes, and shell metacharacters (`;`, `&`, `|`, `>`, `<`, `$`) are strictly rejected with validation errors.
  - Optional unit extension validated against recognized systemd/OpenRC unit types (`.service`, `.socket`, `.target`, `.timer`, etc.).
- **Verdict:** Secure. Injection attempts fail closed before reaching system execution.

### Abuse Scenario 2: Command Injection & Working Directory Traversal in Execution Paths
- **Attack Vector:** Specifying empty, malformed, or path-traversing execution commands (`exec_start`, `exec_stop`, `exec_reload`) or relative/traversing working directories (`working_dir = "../../../root"`).
- **Mitigation:** Invariant `SS2` enforced in `validate_service_spec`:
  - `exec_start` is mandatory, non-empty, non-whitespace, capped at 4,096 characters.
  - `exec_stop` and `exec_reload`, if provided, cannot be empty or whitespace-only, capped at 4,096 characters.
  - `working_dir`, if provided, must be an absolute path (Unix `/...` or Windows drive path `C:\...`) and is strictly checked for directory traversal sequences (`..`).
- **Verdict:** Secure. Prevents path traversal and execution escapes.

### Abuse Scenario 3: Dependency Graph Poisoning & Cyclic Resolution Loops
- **Attack Vector:** Constructing self-referential dependencies (`service_a -> service_a`) or flooding duplicate/excessive dependencies to cause infinite loops or resource exhaustion in service dependency resolution algorithms.
- **Mitigation:** Invariant `SS3` enforced in `validate_service_spec`:
  - Self-dependencies (`dep.name == spec.name`) are explicitly rejected.
  - Duplicate dependencies for the same target service are rejected.
  - Total dependency count is capped at 128 items.
  - Each dependency target name is recursively validated against `validate_service_name`.
- **Verdict:** Secure. Guarantees dependency graphs are acyclic and bounded.

### Abuse Scenario 4: Resource Exhaustion & Allocation DoS
- **Attack Vector:** Submitting specifications with unbounded environment variables, huge descriptions, or infinite timeouts ($t = 0$ or $t = \infty$) leading to unkillable or hanging service processes and memory exhaustion.
- **Mitigation:** Invariant `SS4` enforced in `validate_service_spec`:
  - Timeouts `timeout_start_secs` and `timeout_stop_secs` must be within $[1 \dots 86,400]$ seconds ($1\text{ s}$ to $24\text{ h}$). Zero and values exceeding 86,400 are rejected.
  - Description capped at 4,096 bytes.
  - Environment map limited to $\le 256$ entries; keys must be non-empty, cannot contain `=` or control/null characters, keys and values capped at 4,096 bytes.
  - User and group names validated against POSIX username bounds ($[1 \dots 32]$ chars, alphanumeric, `_`, `-`).
- **Verdict:** Secure. Prevents resource exhaustion and unbounded processing.

### Abuse Scenario 5: Contradictory Lifecycle & Status Corruption
- **Attack Vector:** Injecting telemetry indicating a `Failed` state while simultaneously asserting `healthy == true`, or asserting `startup_mode == Masked` for an `Active` running unit.
- **Mitigation:** Invariant `SS5` enforced in `validate_service_status`:
  - Enforces logical state consistency: `state == Failed` is incompatible with `healthy == true`.
  - `startup_mode == Masked` is incompatible with `state == Active` or `Activating`.
  - Process ID sanity checks ($pid > 0$).
- **Verdict:** Secure. Prevents state corruption and misleading telemetry.

## 2. Policy Gating & Audit Trail Conformance
- **PEP Enforcement:** Autonomous agent invocations of `aios.service.validate` pass through `dispatch::recorded_call`, supporting optional `grant_id` validation.
- **Immutable Audit Trail:** All operations via CLI (`aiosh service validate`) and MCP tool emit structured, SHA-256 hash-chained audit rows into the SQLite WAL ring (`audit.db`) on all branches (success and failure).
- **Policy Bypass Assessment:** Zero unauthenticated or unaudited execution paths exist. Fail-closed error handling is enforced on all input violations.
