# T-02448: Sandbox Enforcement Configuration Hardening

## 1. Hardening Overview
This document records hardening controls applied to the Sandbox Enforcement Configuration subsystem.

## 2. Implemented Hardening Measures

### A. Pre-Execution File Bounds
- `SandboxConfig::load_from_path` interrogates file metadata prior to allocation. Files exceeding `MAX_CONFIG_FILE_BYTES` (64 KiB) are rejected fail-closed with `SANDBOXCONF_ERR_BOUNDS`.

### B. Pre-Persistence Invariant Enforcement
- `SandboxConfig::save_to_path` invokes `self.validate()` before touching the disk. Malformed or invalid configurations cannot be persisted.

### C. Safe Directory & Traversal Guards
- `custom_profiles_dir` rejects directory traversal components (`..`), mitigating escape from the designated profiles root.

### D. Bounded Parsing & Deserialization
- Numerical fields are clamped against strict lower and upper bounds, protecting against integer underflow, overflow, and resource starvation.

### E. Auditable Non-Silent Failures
- All errors across CLI (`aiosh sandbox config`) and MCP (`aios.sandbox.config`) emit structured error envelopes and write to `audit_ring`.
