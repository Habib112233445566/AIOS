# T-01747: Hardware Detection — Configuration Security Review

## Metadata
- **Task ID**: `T-01747`
- **Sub-Epic**: Sub-Epic 5: Hardware Detection Configuration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Security Review Scope
Reviewed `code/aiosh-rust/aiosh-core/src/hardware_config.rs` and its interactions with `HardwareService` in `code/aiosh-rust/aiosh-core/src/hardware_service.rs` and the broader userspace execution boundary.

---

## 2. Threat Analysis & Vulnerability Findings

### Finding CFG-SEC-1: Path Traversal and Relative Path Escaping
- **Vulnerability**: `HardwareConfig::validate()` validates path length ($\le 1024$ characters) and checks for control/NUL characters, but does not detect or reject directory traversal components (`..`) in `default_store_path`.
- **Threat Vector**: A malicious actor or compromised configuration file could configure `default_store_path: "../../../etc/evil.json"` and trigger `save_to_path()`, attempting to write JSON into sensitive directory trees.
- **Severity**: **MEDIUM** (CWE-22)
- **Remediation**: In `T-01748`, enhance `validate()` to ensure path components in `default_store_path` do not contain `..` (parent directory traversal).

### Finding CFG-SEC-2: Unvalidated State in `HardwareConfig::from_env()`
- **Vulnerability**: `from_env()` applies values from `AIOSH_HARDWARE_SYSFS`, `AIOSH_HARDWARE_PROCFS`, and `AIOSH_HARDWARE_STORE` directly without invoking `validate()`. If an environment variable provides an invalid path (e.g. control characters or excessive length), the resulting `HardwareConfig` violates invariants HCFG1..HCFG4.
- **Threat Vector**: Injected or malformed environment variables create unvalidated `HardwareConfig` instances that may cause unexpected errors downstream.
- **Severity**: **MEDIUM** (CWE-20)
- **Remediation**: In `T-01748`, ensure `from_env()` validates the resulting configuration, sanitizing or rejecting invalid environment variable overrides.

### Finding CFG-SEC-3: Non-Atomic File Writes in `save_to_path()`
- **Vulnerability**: `save_to_path()` directly writes to the destination path using `fs::write()`. If the process is interrupted or terminated midway, a truncated or corrupted JSON file may remain on disk.
- **Threat Vector**: Loss of configuration integrity upon system reboot or unexpected power loss.
- **Severity**: **LOW** (CWE-377)
- **Remediation**: In `T-01748`, implement atomic write semantics (write to temporary sibling file followed by atomic rename).

---

## 3. Hardening Recommendations for T-01748
1. Add `Component::ParentDir` traversal checks to `HardwareConfig::validate()`.
2. Invoke `self.validate()` inside `HardwareConfig::from_env()`, falling back to safe defaults if environment overrides violate invariants.
3. Utilize atomic replace (`.tmp` + rename) in `save_to_path()`.
