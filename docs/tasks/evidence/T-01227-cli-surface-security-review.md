# T-01227: Package Management - CLI Surface: Security Review

## Metadata
- **Task ID:** `T-01227`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Package Management CLI Surface Security Review
- **Status:** Complete

## 1. Security Architecture & Threat Surface Overview
The Package Management CLI surface (`aiosh package`) handles package inspection, validation, planning, and transactional state application. All operations interact directly with the in-process `PackageStore` and `aiosh_core::package` domain logic, without spawning shell subshells.

Key security controls verified:
1. **No Process / Shell Execution**: No subshells or external binaries (`/bin/sh`, `apt`, `apk`) are invoked by `cmd_package`. Argument injection vulnerabilities (e.g., shell metacharacters `;&|`) cannot trigger arbitrary command execution.
2. **Strict Regex Validation**: Package names are strictly validated against `^[a-z0-9][a-z0-9+.-]{1,63}$`, precluding format string bugs, shell metacharacters, and path traversal strings.
3. **Payload Size Limits**: All incoming payloads (`--actions`, `--plan`, `--spec`) are limited to a maximum of 1 MiB (1,048,576 bytes) both for on-disk files and inline strings. Store files are limited to 10 MiB and 10,000 packages.
4. **Audit Emission Guarantees (ADR-0035)**: All state-changing operations and rejected requests consistently call `classify_and_emit(...)`, recording structured events to SQLite `audit.db`.

## 2. Abuse Scenarios & Mitigations

### Scenario 1: Shell Command Injection via Malicious Package Names or Search Query
- **Attack Vector**: An attacker supplies malicious metacharacters (`curl; id`, `$(reboot)`, or `` `rm -rf /` ``) as package names or search patterns.
- **Analysis & Defense**:
  - `aiosh package` operates entirely in-memory using Rust native types and standard library string comparisons.
  - Package names in `validate`, `show`, `plan`, and `apply` must conform to the POSIX/Debian package naming regex `^[a-z0-9][a-z0-9+.-]{1,63}$`. Non-conforming names are rejected with exit code 2 before any action.
  - Search patterns perform simple in-memory substring matching on names and descriptions without compiling unvetted regular expressions (precluding ReDoS).
- **Residual Risk**: None. Fully mitigated.

### Scenario 2: Resource Exhaustion (Memory/Disk DoS) via Massive Payloads
- **Attack Vector**: An attacker passes a multi-gigabyte file or deeply nested JSON payload via `--actions` or `--plan` to cause Out-Of-Memory (OOM) or high CPU usage.
- **Analysis & Defense**:
  - File metadata is checked prior to reading; files exceeding 1,048,576 bytes are immediately rejected with error code `PAYLOAD_TOO_LARGE` without reading into memory.
  - Inline string arguments are verified to be under 1,048,576 bytes before passing to `serde_json`.
  - Store files are bounded to 10 MiB in `PackageStore::load_from_path`, with a maximum of 10,000 package entries allowed.
- **Residual Risk**: None. Fully mitigated.

### Scenario 3: Path Traversal / Arbitrary File Overwrite via `--store`
- **Attack Vector**: An operator or compromised agent attempts to pass `--store /etc/shadow` or relative path traversal sequences `../../../../root/.bashrc` to overwrite critical system files.
- **Analysis & Defense**:
  - `PackageStore::save_to_path` performs atomic persistence by writing to an adjacent temporary file (`.tmp.<pid>`) with explicit `0o644` permissions and renaming.
  - Standard OS file system permissions apply. Unprivileged processes cannot overwrite root-owned system files.
  - Failures in file writing or renaming cleanly clean up the temporary file via RAII/drop logic and return a structured `PERSISTENCE_FAILED` error with an audit row.
- **Residual Risk**: Accepted administrative privilege boundaries for the operator role.

### Scenario 4: Inconsistent Dependency State & System Breakage
- **Attack Vector**: Submitting a transaction that installs a package without its required dependencies or removes a prerequisite package, corrupting system integrity.
- **Analysis & Defense**:
  - `PackageStore::plan_transaction` rigorously validates dependency closure (CS2) and builds a topologically sorted execution plan.
  - Unmet dependencies or cycle violations fail planning immediately (`PLAN_FAILED`, exit code 2).
  - Dry-run transactions (`--dry-run`) verify calculation delta (CS4) without mutating in-memory or on-disk state.
- **Residual Risk**: None. Fully mitigated.

### Scenario 5: Audit Log Bypass
- **Attack Vector**: Triggering an intentional error condition to execute code paths without leaving a trace in `audit.db`.
- **Analysis & Defense**:
  - Every error branch in `validate`, `list`, `show`, `search`, `plan`, and `apply` calls `classify_and_emit` to record the exact error code, message, and target package name.
  - State changes in `apply` emit high-fidelity audit records including transaction ID, counts of installed/removed/upgraded packages, and total size delta.
- **Residual Risk**: None. Fully mitigated.

## 3. Review Conclusion
Zero policy bypasses or unhandled security scenarios remain open. The CLI surface conforms to AIOS security requirements and ADR-0035 audit standards.
