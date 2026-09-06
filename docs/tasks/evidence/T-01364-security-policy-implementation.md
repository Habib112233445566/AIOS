# T-01364: Init & Service Supervision - Security Policy: Implementation

## Metadata
- **Task ID:** `T-01364`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Security Policy
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Scope & Objective
Implement the complete core functionality for `ServiceSecurityPolicy` in `code/aiosh-rust/aiosh-core/src/service_policy.rs` adhering strictly to invariants `SP1..SP6`.

---

## 2. Changes Made
1. **Implemented Invariants (SP1..SP6)**:
   - `SP1`: Policy configuration bounds validation (`validate()`):
     - `prohibited_services.len() <= 1024`, no control characters or whitespace.
     - `prohibited_exec_paths.len() <= 128`, must be absolute paths.
     - `disallow_env_vars.len() <= 128`, no control characters or `=`.
     - `allowed_service_types` cannot be empty.
     - `allowed_root_services.len() <= 256`.
     - `max_env_vars` in $[1..1024]$, `max_timeout_secs` in $[1..86400]$.
   - `SP2`: Prohibited services blocking (`evaluate_spec()`):
     - Matches exact name and stripped `.service` suffix against default prohibited list (`telnet.service`, `rsh.service`, `rlogin.service`, `rexec.service`, `tftp.service`, `xinetd.service`, `ypserv.service`, `ypbind.service`).
     - Emits `SP2-PROHIBITED-SERVICE` (fatal = true).
   - `SP3`: Executable path & working dir hygiene:
     - Binary extraction for `exec_start`, `exec_stop`, `exec_reload`.
     - Validates absolute path, absence of `..` traversal, and rejects binaries or working dirs under `prohibited_exec_paths` (`/tmp`, `/var/tmp`, `/dev/shm`, `/run/user`).
     - Emits `SP3-RELATIVE-PATH`, `SP3-PATH-TRAVERSAL`, `SP3-PROHIBITED-PATH`.
   - `SP4`: User privilege & root hygiene:
     - Enforces `require_service_user` and `disallow_root` with exemptions for `allowed_root_services`.
     - Emits `SP4-UNPRIVILEGED-USER-REQUIRED`, `SP4-ROOT-DISALLOWED`.
   - `SP5`: Environment & timeout sanitization:
     - Evaluates prohibited environment variables (`LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`).
     - Verifies environment variable count $\le$ `max_env_vars` and timeouts $\le$ `max_timeout_secs`.
     - Emits `SP5-DANGEROUS-ENV-VAR`, `SP5-ENV-COUNT-EXCEEDED`, `SP5-TIMEOUT-EXCEEDED`, `SP5-DISALLOWED-TYPE`.
   - `SP6`: Tri-state policy modes:
     - `Enforcing`: blocks on any fatal violation.
     - `Audit`: passes all specs while recording violations.
     - `Permissive`: passes non-fatal violations but blocks prohibited services.
2. **Store Evaluation & Configuration Loading**:
   - Implemented `evaluate_store(&self, store: &ServiceStore) -> Vec<ServicePolicyVerdict>`.
   - Implemented `from_file`, `from_source`, `from_env`, and precedence resolver `resolve`.
3. **Automated Unit Tests**:
   - Added unit test suite in `service_policy.rs` covering default validation, prohibited services, path traversal/prohibited dirs, dangerous env vars, root restrictions, and audit mode.
   - Verified 6 tests passing cleanly via `cargo test -p aiosh-core --lib service_policy::tests`.
