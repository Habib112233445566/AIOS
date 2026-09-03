# T-01032 — Distro Selection & Justification / MCP/API Surface: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / MCP/API Surface

## 1. Tool Signatures & Input Schemas

### 1. `aios.distro.list`
- **Description**: Returns an array of registered distribution profiles.
- **Parameters**:
  - `store_path` (optional, string): Path to custom store file.
- **Response**:
  ```json
  {
    "count": 2,
    "profiles": [
      {
        "id": "debian-12-minimal-x86_64",
        "name": "Debian GNU/Linux 12 (Bookworm) Minimal",
        "family": "Debian",
        "arch": "X86_64",
        "c_lib": "Glibc",
        "init_system": "Systemd",
        "min_kernel_version": "6.1.0",
        "recommended": true,
        "default_packages": ["systemd", "udev", "iproute2", "ca-certificates", "curl", "python3-minimal"],
        "justification": "Primary tier-1 base for AIOS userspace..."
      }
    ]
  }
  ```

### 2. `aios.distro.show`
- **Description**: Returns detailed specification of a single profile.
- **Parameters**:
  - `id` (required, string): Profile ID.
  - `store_path` (optional, string).
- **Validation**: If `id` is null or empty, returns JSON-RPC `-32602` error (`Invalid params: missing 'id'`).
- **Response**: Full `DistroProfile` object. If profile does not exist, returns `ok: false, error: "Distro profile '...' not found"`.

### 3. `aios.distro.evaluate`
- **Description**: Evaluates single profile or all registered profiles against AIOS evaluation formulas.
- **Parameters**:
  - `id` (optional, string): Profile ID.
  - `store_path` (optional, string).
- **Response**: If `id` provided: single `DistroEvaluation` object. If omitted: `{ "count": N, "evaluations": [...] }` sorted by `overall_score` descending.

### 4. `aios.distro.recommend`
- **Description**: Returns recommended reference profile.
- **Parameters**:
  - `store_path` (optional, string).
- **Response**: `DistroProfile` object with `recommended: true`.

## 2. Dispatch Engine & Policy Gate
All tools route through:
```rust
dispatch::recorded_call(
    &mut self.audit_db,
    &mut self.classifier,
    &self.constitution_rev,
    tool_name,
    args,
    "agent",
    "agent:mcp@aiosh-mcp",
    None,
    |args| { ... }
)
```
Ensuring:
1. Classification verdict checked against `AI_CONSTITUTION.md`.
2. Consequential action logged to `AuditRing`.
3. Standard error envelope on failure.
