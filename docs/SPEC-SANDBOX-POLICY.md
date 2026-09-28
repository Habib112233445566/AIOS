# Specification: Sandbox Enforcement Security Policy Subsystem

## 1. Overview
The Sandbox Enforcement Security Policy Subsystem (`SandboxSecurityPolicy`) enforces declarative, system-wide constraints on all sandboxed execution requests across AIOS. It operates as an overarching gate evaluating commands, environment variables, capabilities, and resource ceilings before process creation.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `SANDBOXPOL1` | **Tri-Mode Evaluation** | Supports `Enforcing` (fail-closed rejections), `Permissive` (audit warning emitted), and `Disabled` (bypass). |
| `SANDBOXPOL2` | **Prohibited Commands** | Enforces an immutable denylist of destructive utilities (`rm`, `dd`, `mkfs`, `format`, `fdisk`, `shutdown`, `reboot`, `poweroff`). Matches command basename and strips `.exe`. |
| `SANDBOXPOL3` | **Environment Confinement** | Blocks loader injection and library hijacking variables (`LD_PRELOAD`, `DYLD_INSERT_LIBRARIES`, `PYTHONPATH`, `NODE_OPTIONS`) using case-insensitive comparison. |
| `SANDBOXPOL4` | **PEP Grant Mandate** | Configurable profiles can be designated as requiring explicit PEP capability grants. |
| `SANDBOXPOL5` | **Global Resource Ceiling** | Binds requested wall-clock execution time and maximum memory limits to global policy ceilings. |
| `SANDBOXPOL6` | **Storage Hardening** | JSON policy configuration files are clamped to 64 KiB, reject symlinks, and forbid directory traversal (`..`). |

---

## 3. Interfaces & Usage

### CLI Commands
```bash
# View active sandbox security policy
aiosh sandbox policy

# View policy in structured JSON envelope
aiosh sandbox policy --json

# Load and inspect custom security policy file
aiosh sandbox policy --path /etc/aiosh/sandbox_policy.json --json
```

### MCP Tool Surface
- Tool: `aios.sandbox.policy`
- Arguments: `{"policy_path": "<optional-path>"}`
- Response:
```json
{
  "ok": true,
  "policy": {
    "version": "1.0.0",
    "mode": "enforcing",
    "prohibited_commands": ["rm", "dd", "mkfs", "format", "fdisk", "shutdown", "reboot", "poweroff"],
    "prohibited_env_vars": ["LD_PRELOAD", "LD_LIBRARY_PATH", "DYLD_INSERT_LIBRARIES", "PYTHONPATH", "NODE_OPTIONS"],
    "require_pep_grant_for_profiles": [],
    "max_permissible_wall_time_ms": 300000,
    "max_permissible_memory_bytes": 8589934592
  }
}
```
