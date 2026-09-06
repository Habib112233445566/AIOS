# T-01327: Init & Service Supervision - CLI Surface: Security Review

## Metadata
- **Task ID:** `T-01327`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Security Review
- **Status:** Complete

## 1. Threat Modeling & Abuse Scenarios

### Abuse Scenario 1: Command Injection & Flag Smuggling via Service Names
- **Attack Vector:** An attacker supplies shell metacharacters, control bytes, or synthetic flag strings (e.g., `; reboot`, `bad\0name`, `--privileged`) as the service name argument to CLI commands.
- **Mitigation:**
  - `cmd_service` positional parsing explicitly separates options from target names.
  - Service names are validated to not exceed 128 characters and cannot contain control characters (`c.is_control()`).
  - Validation commands deep-audit names against invariant `SS1` (`^[a-zA-Z0-9][a-zA-Z0-9_.-]*$`), rejecting slashes, backslashes, and shell metacharacters.
- **Verdict:** Secure. Fail-closed rejection before invocation of any system subprocess or store lookups.

### Abuse Scenario 2: Path Traversal & Store Hijacking via `--store`
- **Attack Vector:** Supplying arbitrary relative paths or traversal sequences (`../../etc/shadow`) to overwrite sensitive host files or load untrusted registry profiles.
- **Mitigation:**
  - `store_path` parameter is length-capped at 1,024 bytes and checked for control characters.
  - Store persistence (`save_to_path`) uses atomic temporary file writes with process isolation (`.tmp.<pid>`) and un-link on failure, preserving original file integrity on error.
  - Permissions are constrained to `0o644` on Unix systems.
- **Verdict:** Secure. Filesystem operations respect process execution sandbox.

### Abuse Scenario 3: Memory Exhaustion via Malicious Spec Payloads
- **Attack Vector:** Submitting excessively large files or inline JSON strings to `aiosh service validate --spec` to trigger memory exhaustion and denial of service.
- **Mitigation:**
  - Files and inline payloads are subject to a strict 1 MiB (`1,048,576` bytes) ceiling before reading into memory.
  - Files exceeding the size limit are rejected immediately with exit code 2 and a `PAYLOAD_TOO_LARGE` audit event.
- **Verdict:** Secure. Heap allocation is bounded defensively.

### Abuse Scenario 4: FSM Bypass via Direct Action Shortcuts
- **Attack Vector:** Attempting to use direct action shortcuts (`start`, `restart`, `reload`) to force-activate an administratively masked service or bypass state transition validations.
- **Mitigation:**
  - Direct action shortcuts forward arguments into the core `action` execution engine.
  - State machine invariants are enforced by `ServiceStore::execute_action`: attempts to start or restart a masked unit return an explicit error and are blocked.
- **Verdict:** Secure. Direct action shortcuts maintain complete parity with core FSM rules.

### Abuse Scenario 5: Audit Evasion & Silent Execution
- **Attack Vector:** Attempting to execute state transitions or inspection commands without leaving an entry in the immutable audit trail.
- **Mitigation:**
  - Every subcommand path (`validate`, `list`, `show`, `status`, `action`, `start`, `stop`, `order`) invokes `classify_and_emit`.
  - Captures actor identity ("operator"), action verb, target service name, and outcome ("success" or "failure") into `audit.log` / SQLite WAL ring (`audit.db`).
- **Verdict:** Secure. Zero unaudited execution branches exist.

## 2. Policy & Enforcement Assessment
- **PEP Enforcement:** CLI surfaces enforce actor attribution and write immutable audit records on all execution paths.
- **Fail-Closed Semantics:** All syntax violations, payload ceiling breaches, and invariant failures exit with code 2 and emit failure audit records.
- **Zero Policy Bypass:** Full code review confirms no unauthenticated, unaudited, or unvalidated paths remain.
