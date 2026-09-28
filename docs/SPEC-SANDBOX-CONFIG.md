# Sandbox Enforcement Configuration Subsystem Specification

## 1. Overview
The Sandbox Enforcement Configuration subsystem manages runtime parameters, resource limits, default profiles, and environment overrides for process sandboxing.

---

## 2. Configuration Parameters

| Parameter | Type | Default | Bounds | Description |
|:---|:---:|:---:|:---:|:---|
| `version` | String | `"1.0.0"` | `"1.x"` | Configuration schema version |
| `default_profile_name` | String | `"standard"` | Non-empty | Default containment profile |
| `max_output_capture_bytes` | usize | `10485760` (10 MiB) | `1 KiB .. 64 MiB` | Stdout/stderr capture ceiling |
| `execution_timeout_seconds` | u64 | `300` | `1s .. 86400s` | Process execution timeout |
| `max_registered_profiles` | usize | `256` | `1 .. 1024` | Registry profile capacity |
| `enforce_pep_grants` | bool | `false` | Boolean | Require PEP authorization |
| `audit_enabled` | bool | `true` | Boolean | Audit ring logging toggle |
| `custom_profiles_dir` | Option<Path> | `None` | No traversal (`..`) | Custom profiles directory |

---

## 3. Invocations

### CLI Invocation
```bash
# View active configuration
aiosh sandbox config

# View custom configuration file in JSON format
aiosh sandbox config --path /etc/aios/sandbox.json --json
```

### MCP Tool Invocation
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.sandbox.config",
    "arguments": {
      "config_path": "/etc/aios/sandbox.json"
    }
  }
}
```

---

## 4. Environment Overrides
- `AIOS_SANDBOX_DEFAULT_PROFILE`
- `AIOS_SANDBOX_ENFORCE_PEP`
- `AIOS_SANDBOX_MAX_OUTPUT_BYTES`
- `AIOS_SANDBOX_TIMEOUT_SECS`
- `AIOS_SANDBOX_PROFILES_DIR`
