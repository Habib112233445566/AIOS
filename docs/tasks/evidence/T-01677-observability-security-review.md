# T-01677: Kernel Module Management Observability Security Review

## Sub-Epic
Kernel Module Management / Observability (T-01677)

## Overview
Comprehensive security review and threat analysis of the Kernel Module Management Observability subsystem (`KernelModuleObservabilityReport`, `aiosh mod observability`, and `aios.kernel_module.observability`).

## Threat Modeling & Surface Analysis
- **Surfaces Evaluated:**
  1. Core Report Generator: `aiosh_core::kernel_module_observability::KernelModuleObservabilityReport::generate`
  2. CLI Operational Commands: `aiosh mod observability`, `aiosh mod status`
  3. MCP Agent Interface: `aios.kernel_module.observability`
  4. File System Inputs: `--proc-modules <path>`, `--store <path>`, `--policy <path>`

## Abuse Scenarios & Mitigations (AS-01..AS-06)

### AS-01: Information Disclosure via Kernel Memory Addresses
- **Threat:** `/proc/modules` exposes raw kernel load addresses (column 6) which could defeat KASLR (Kernel Address Space Layout Randomization) if leaked to unprivileged agents.
- **Analysis:** `KernelModuleObservabilityReport` discards raw memory load addresses. Only high-level aggregation metrics (`total_memory_bytes`, `state_breakdown`, `ref_count_distribution`) are included in the emitted report.
- **Verdict:** SAFE.

### AS-02: Path Traversal & Arbitrary File Ingestion
- **Threat:** Untrusted callers supplying `--proc-modules` or `--store` targeting system credentials or sensitive devices (`/dev/urandom`, `/dev/zero`, `/etc/shadow`).
- **Analysis:** Store parsing enforces strict JSON schema (`KernelModuleStore`). Non-JSON files fail immediately. In hardening (T-01678), a strict 1 MiB file size cap and regular-file verification will be enforced on input paths.
- **Verdict:** ACTION ITEM for T-01678.

### AS-03: Resource Exhaustion & Denial of Service
- **Threat:** Supplying arbitrarily large files causing high CPU or memory exhaustion during line parsing and policy evaluation.
- **Analysis:** Current procfs reader reads entire contents into memory. Needs bounded read limit (1 MiB cap) and bounded line length limit (512 bytes per line).
- **Verdict:** ACTION ITEM for T-01678.

### AS-04: Non-Root Execution & Privilege Separation
- **Threat:** Observability calls attempting to invoke privileged kernel facilities (`init_module`, `delete_module`).
- **Analysis:** `KernelModuleObservabilityReport` is strictly passive and read-only. It performs zero mutations and requires zero elevated capabilities (`CAP_SYS_MODULE` is not required).
- **Verdict:** SAFE.

### AS-05: State Invariant Consistency (KO1..KO6)
- **Threat:** Tampering or logic errors leading to contradictory status reports (e.g. loaded module count diverging from state totals).
- **Analysis:** Report generation derives all state breakdown and refcount distribution numbers directly from the loaded module set in a single linear pass. Total counts strictly match breakdown sums.
- **Verdict:** SAFE.

### AS-06: Audit Logging Integrity
- **Threat:** Sensitive configuration details or unescaped terminal control sequences injected into audit logs.
- **Analysis:** CLI integration uses structured audit logging (`classify_and_emit`) with JSON serialization and sanitization.
- **Verdict:** SAFE.

## Hardening Recommendations for T-01678
1. Implement 1 MiB reading ceiling on `proc_modules_path`.
2. Add maximum line length truncation (512 bytes per line) to prevent line-buffer attacks.
3. Verify that non-virtual procfs paths are regular files before reading.
4. Sanitize error messages in CLI and MCP to avoid path leakages.
