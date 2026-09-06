# T-01367: Init & Service Supervision / Security Policy - Security Review

## Executive Summary
This document provides a thorough security review and threat modeling analysis of the AIOS Init & Service Supervision Security Policy subsystem (`ServiceSecurityPolicy`, `ServicePolicyMode`, `ServicePolicyVerdict`, `ServicePolicyViolation`, criteria `SP1..SP6`). The review examines potential attack vectors, path injection vulnerabilities, root privilege escalation, environment variable tampering, denial-of-service protections, and audit logging compliance against NIST SP 800-53 (AC-6, CM-7), CIS Linux Benchmarks (Sections 2 & 3), and systemd execution security directives.

---

## Threat Model & Abuse Scenario Analysis

### 1. Prohibited Service Evasion & Suffix Stripping Attacks (SP2)
- **Threat**: An adversary attempts to bypass prohibited daemon blocking by varying case (e.g. `Telnet.service`, `TELNET`), alternating suffixes (e.g. `telnet` instead of `telnet.service`), or injecting whitespace/control characters.
- **Mitigation & Evaluation**:
  - `ServiceSecurityPolicy::evaluate_spec` converts service identifiers to lowercase and strips the canonical `.service` suffix before comparison:
    ```rust
    let name_lower = spec.name.to_lowercase();
    let name_no_suffix = name_lower.strip_suffix(".service").unwrap_or(&name_lower);
    ```
  - Exact matches and stripped suffix matches against `prohibited_services` emit fatal violation `SP2-PROHIBITED-SERVICE`.
  - Service naming syntax validation (`validate_service_name` / `SP1-MALFORMED-NAME`) rejects control characters and non-alphanumeric leading characters.
- **Verdict**: SECURE. Suffix variation and casing cannot circumvent prohibited service blocking.

### 2. Binary Path Injection & Directory Traversal (SP3)
- **Threat**: An adversary attempts to execute untrusted binaries by referencing relative paths (e.g. `bin/daemon`), world-writable temporary directories (e.g. `/tmp/exploit`, `/dev/shm/agent`), or path traversal sequences (e.g. `/usr/bin/../../tmp/exploit`).
- **Mitigation & Evaluation**:
  - `evaluate_spec` inspects the executable token for `exec_start`, `exec_stop`, and `exec_reload`:
    - Enforces absolute paths (must start with `/` on Unix or drive letter on Windows); emits `SP3-RELATIVE-PATH`.
    - Prohibits directory traversal sequences (`..`); emits `SP3-PATH-TRAVERSAL`.
    - Rejects binary locations and `working_dir` residing within `prohibited_exec_paths` prefixes (`/tmp`, `/var/tmp`, `/dev/shm`, `/run/user`); emits `SP3-PROHIBITED-PATH`.
- **Verdict**: SECURE. Execution from untrusted temporary directories and directory traversal attacks are deterministically blocked.

### 3. Root Privilege Escalation & User Omission (SP4)
- **Threat**: In traditional init systems, unspecified service users default to running with full UID 0 (`root`) privileges, allowing compromised services to compromise the entire operating system.
- **Mitigation & Evaluation**:
  - When `require_service_user = true`, services without an explicitly defined `user` generate fatal violation `SP4-UNPRIVILEGED-USER-REQUIRED`.
  - When `disallow_root = true`, services specifying `user = "root"` or `user = "0"` generate fatal violation `SP4-ROOT-DISALLOWED`.
  - Core system services requiring root permissions are strictly governed by `allowed_root_services` (e.g. `systemd-journald.service`, `aios-securityd.service`).
- **Verdict**: SECURE. Principle of least privilege is strictly enforced; root execution requires explicit architectural exemption.

### 4. Dynamic Linker & Environment Variable Hijacking (SP5)
- **Threat**: Attackers inject environment variables such as `LD_PRELOAD`, `LD_LIBRARY_PATH`, or `IFS` into service definitions to hijack execution flow and execute arbitrary shared objects.
- **Mitigation & Evaluation**:
  - `evaluate_spec` checks all keys in `spec.environment` against `disallow_env_vars` (defaults include `LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`).
  - Presence of dangerous environment variables triggers fatal violation `SP5-DANGEROUS-ENV-VAR`.
  - Service spec validation ensures keys cannot contain `=` or control characters, preventing argument/env injection.
- **Verdict**: SECURE. Linker hijacking and environment variable injection vectors are prevented.

### 5. Denial-of-Service via Configuration Exhaustion (SP1)
- **Threat**: Attackers supply huge configuration files or unbounded timeout values to exhaust system memory or block shutdown sequences.
- **Mitigation & Evaluation**:
  - Policy file reading strictly enforces a 64 KiB ceiling (`MAX_POLICY_FILE_BYTES = 65_536`).
  - Limits enforced by `validate()`:
    - Maximum prohibited services: 1024.
    - Maximum prohibited paths: 128.
    - Maximum disallowed env vars: 128.
    - Maximum timeout ceiling: 86,400s (24 hours).
    - Maximum env vars per spec: 1024.
- **Verdict**: SECURE. Memory and timeout limits prevent resource exhaustion and hanging processes.

### 6. Audit Logging & Non-Repudiation (SP6 / PEP Integration)
- **Threat**: Administrative actions or policy overrides occur without forensic audit trails.
- **Mitigation & Evaluation**:
  - Both CLI (`aiosh service policy`) and MCP (`aios.service.policy`) emit immutable SHA-256 hash-chained audit records to the SQLite WAL ring buffer.
  - MCP requests are protected by PEP gating via `grant_id` and dispatched through `dispatch::recorded_call`.
  - Audit mode (`ServicePolicyMode::Audit`) produces full violation reporting without mutative blocking.
- **Verdict**: SECURE. Complete forensic non-repudiation is maintained.

---

## Conclusion
The Init & Service Supervision Security Policy subsystem satisfies all security invariants `SP1..SP6`. No known policy bypasses or architectural vulnerabilities remain open.
